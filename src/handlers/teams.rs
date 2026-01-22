use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::Pool;
use sqlx::Postgres;
use uuid::Uuid;
use chrono::Utc;
use regex::Regex;

use crate::error::ApiError;
use crate::models::collaboration::{
    Team, TeamMember, CreateTeamRequest, UpdateTeamRequest,
    AddTeamMemberRequest, UpdateTeamMemberRequest, ProjectMember,
    AddProjectMemberRequest, UpdateProjectMemberRequest, PermissionCheck,
};
use crate::middleware::rbac;

/// Create new team
pub async fn create_team(
    State(pool): State<Pool<Postgres>>,
    user_id: Uuid,
    Json(req): Json<CreateTeamRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let team_id = Uuid::new_v4();
    let now = Utc::now();

    // Generate slug from team name
    let slug = generate_slug(&req.name);

    sqlx::query(
        r#"
        INSERT INTO teams (id, owner_id, name, description, slug, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $6)
        "#,
    )
    .bind(team_id)
    .bind(user_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&slug)
    .bind(now)
    .execute(&pool)
    .await?;

    // Add creator as owner
    sqlx::query(
        r#"
        INSERT INTO team_members (id, team_id, user_id, role, joined_at)
        VALUES ($1, $2, $3, 'owner', $4)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(team_id)
    .bind(user_id)
    .bind(now)
    .execute(&pool)
    .await?;

    let team = Team {
        id: team_id,
        owner_id: user_id,
        name: req.name,
        description: req.description,
        slug,
        created_at: now,
        updated_at: now,
    };

    Ok((StatusCode::CREATED, Json(team)))
}

/// Get team details
pub async fn get_team(
    State(pool): State<Pool<Postgres>>,
    Path(team_id): Path<Uuid>,
    user_id: Uuid,
) -> Result<impl IntoResponse, ApiError> {
    // Verify user is team member
    let is_member = sqlx::query_scalar::<_, bool>(
        "SELECT COUNT(*) > 0 FROM team_members WHERE team_id = $1 AND user_id = $2"
    )
    .bind(team_id)
    .bind(user_id)
    .fetch_one(&pool)
    .await?;

    if !is_member {
        return Err(ApiError::Forbidden);
    }

    let team = sqlx::query_as::<_, Team>("SELECT * FROM teams WHERE id = $1")
        .bind(team_id)
        .fetch_optional(&pool)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(team))
}

/// Update team
pub async fn update_team(
    State(pool): State<Pool<Postgres>>,
    Path(team_id): Path<Uuid>,
    user_id: Uuid,
    Json(req): Json<UpdateTeamRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check if user is owner or admin
    rbac::enforce_role(&pool, user_id, team_id, 3).await?;

    let now = Utc::now();

    sqlx::query(
        r#"
        UPDATE teams 
        SET 
            name = COALESCE($1, name),
            description = COALESCE($2, description),
            updated_at = $3
        WHERE id = $4
        "#,
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(now)
    .bind(team_id)
    .execute(&pool)
    .await?;

    Ok(StatusCode::OK)
}

/// List team members
pub async fn list_team_members(
    State(pool): State<Pool<Postgres>>,
    Path(team_id): Path<Uuid>,
    user_id: Uuid,
) -> Result<impl IntoResponse, ApiError> {
    // Verify user is team member
    let is_member = sqlx::query_scalar::<_, bool>(
        "SELECT COUNT(*) > 0 FROM team_members WHERE team_id = $1 AND user_id = $2"
    )
    .bind(team_id)
    .bind(user_id)
    .fetch_one(&pool)
    .await?;

    if !is_member {
        return Err(ApiError::Forbidden);
    }

    let members = sqlx::query_as::<_, TeamMember>(
        "SELECT * FROM team_members WHERE team_id = $1 ORDER BY joined_at DESC"
    )
    .bind(team_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(members))
}

/// Add team member
pub async fn add_team_member(
    State(pool): State<Pool<Postgres>>,
    Path(team_id): Path<Uuid>,
    user_id: Uuid,
    Json(req): Json<AddTeamMemberRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check if user is owner or admin
    rbac::enforce_role(&pool, user_id, team_id, 3).await?;

    // Validate role
    if !["owner", "admin", "member", "viewer"].contains(&req.role.as_str()) {
        return Err(ApiError::BadRequest);
    }

    let member_id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO team_members (id, team_id, user_id, role, joined_at)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (team_id, user_id) DO NOTHING
        "#,
    )
    .bind(member_id)
    .bind(team_id)
    .bind(req.user_id)
    .bind(&req.role)
    .bind(now)
    .execute(&pool)
    .await?;

    let member = TeamMember {
        id: member_id,
        team_id,
        user_id: req.user_id,
        role: req.role,
        joined_at: now,
    };

    Ok((StatusCode::CREATED, Json(member)))
}

/// Update team member role
pub async fn update_team_member(
    State(pool): State<Pool<Postgres>>,
    Path((team_id, member_id)): Path<(Uuid, Uuid)>,
    user_id: Uuid,
    Json(req): Json<UpdateTeamMemberRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check if user is owner or admin
    rbac::enforce_role(&pool, user_id, team_id, 3).await?;

    // Validate role
    if !["owner", "admin", "member", "viewer"].contains(&req.role.as_str()) {
        return Err(ApiError::BadRequest);
    }

    sqlx::query(
        "UPDATE team_members SET role = $1 WHERE id = $2 AND team_id = $3"
    )
    .bind(&req.role)
    .bind(member_id)
    .bind(team_id)
    .execute(&pool)
    .await?;

    Ok(StatusCode::OK)
}

/// Remove team member
pub async fn remove_team_member(
    State(pool): State<Pool<Postgres>>,
    Path((team_id, member_id)): Path<(Uuid, Uuid)>,
    user_id: Uuid,
) -> Result<impl IntoResponse, ApiError> {
    // Check if user is owner or admin
    rbac::enforce_role(&pool, user_id, team_id, 3).await?;

    sqlx::query("DELETE FROM team_members WHERE id = $1 AND team_id = $2")
        .bind(member_id)
        .bind(team_id)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Add project member
pub async fn add_project_member(
    State(pool): State<Pool<Postgres>>,
    Path((project_id, user_id_to_add)): Path<(Uuid, Uuid)>,
    user_id: Uuid,
    Json(req): Json<AddProjectMemberRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check if user is project admin
    rbac::enforce_permission(&pool, user_id, project_id, "admin").await?;

    // Validate permissions
    let valid_perms = vec!["read", "write", "admin", "delete"];
    if let Some(ref perms) = req.permissions {
        for perm in perms {
            if !valid_perms.contains(&perm.as_str()) {
                return Err(ApiError::BadRequest);
            }
        }
    }

    let member_id = Uuid::new_v4();
    let now = Utc::now();
    let permissions = req.permissions.unwrap_or_default();

    sqlx::query(
        r#"
        INSERT INTO project_members (id, project_id, user_id, role, permissions, joined_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (project_id, user_id) DO NOTHING
        "#,
    )
    .bind(member_id)
    .bind(project_id)
    .bind(user_id_to_add)
    .bind(&req.role)
    .bind(&permissions)
    .bind(now)
    .execute(&pool)
    .await?;

    let member = ProjectMember {
        id: member_id,
        project_id,
        user_id: user_id_to_add,
        role: req.role,
        permissions,
        joined_at: now,
    };

    Ok((StatusCode::CREATED, Json(member)))
}

/// Update project member
pub async fn update_project_member(
    State(pool): State<Pool<Postgres>>,
    Path((project_id, member_id)): Path<(Uuid, Uuid)>,
    user_id: Uuid,
    Json(req): Json<UpdateProjectMemberRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check if user is project admin
    rbac::enforce_permission(&pool, user_id, project_id, "admin").await?;

    // Validate permissions
    let valid_perms = vec!["read", "write", "admin", "delete"];
    if let Some(ref perms) = req.permissions {
        for perm in perms {
            if !valid_perms.contains(&perm.as_str()) {
                return Err(ApiError::BadRequest);
            }
        }
    }

    sqlx::query(
        r#"
        UPDATE project_members 
        SET 
            role = COALESCE($1, role),
            permissions = COALESCE($2, permissions)
        WHERE id = $3 AND project_id = $4
        "#,
    )
    .bind(&req.role)
    .bind(&req.permissions)
    .bind(member_id)
    .bind(project_id)
    .execute(&pool)
    .await?;

    Ok(StatusCode::OK)
}

/// Remove project member
pub async fn remove_project_member(
    State(pool): State<Pool<Postgres>>,
    Path((project_id, member_id)): Path<(Uuid, Uuid)>,
    user_id: Uuid,
) -> Result<impl IntoResponse, ApiError> {
    // Check if user is project admin
    rbac::enforce_permission(&pool, user_id, project_id, "admin").await?;

    sqlx::query("DELETE FROM project_members WHERE id = $1 AND project_id = $2")
        .bind(member_id)
        .bind(project_id)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Check user permissions
pub async fn check_permissions(
    State(pool): State<Pool<Postgres>>,
    Path((project_id, check_user_id)): Path<(Uuid, Uuid)>,
    user_id: Uuid,
) -> Result<impl IntoResponse, ApiError> {
    // Check if requester has admin permission
    rbac::enforce_permission(&pool, user_id, project_id, "admin").await?;

    let permissions = sqlx::query_scalar::<_, Vec<String>>(
        r#"
        SELECT permissions FROM project_members
        WHERE project_id = $1 AND user_id = $2
        "#,
    )
    .bind(project_id)
    .bind(check_user_id)
    .fetch_optional(&pool)
    .await?
    .unwrap_or_default();

    let has_permission = !permissions.is_empty();

    let check = PermissionCheck {
        user_id: check_user_id,
        project_id,
        has_permission,
        permissions,
    };

    Ok(Json(check))
}

/// Generate URL-friendly slug from text
fn generate_slug(text: &str) -> String {
    let re = Regex::new(r"[^a-z0-9]+").unwrap();
    re.replace_all(&text.to_lowercase(), "-")
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slug_generation() {
        assert_eq!(generate_slug("My Team"), "my-team");
        assert_eq!(generate_slug("Awesome_Project"), "awesome-project");
        assert_eq!(generate_slug("---test---"), "test");
    }
}
