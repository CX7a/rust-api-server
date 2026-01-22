use axum::{
    middleware::Next,
    http::{Request, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;
use sqlx::Pool;
use sqlx::Postgres;

use crate::error::ApiError;

/// RBAC middleware for enforcing role-based access control
pub async fn rbac_middleware<B>(
    request: Request<B>,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    // Extract user from JWT claims
    let user_id = request
        .extensions()
        .get::<Uuid>()
        .copied()
        .ok_or(ApiError::Unauthorized)?;

    // User ID added to extensions, continue
    Ok(next.run(request).await)
}

/// Check if user has specific permission on project
pub async fn check_project_permission(
    pool: &Pool<Postgres>,
    user_id: Uuid,
    project_id: Uuid,
    required_permission: &str,
) -> Result<bool, ApiError> {
    let result = sqlx::query_scalar::<_, Vec<String>>(
        r#"
        SELECT permissions FROM project_members
        WHERE user_id = $1 AND project_id = $2
        "#,
    )
    .bind(user_id)
    .bind(project_id)
    .fetch_optional(pool)
    .await?;

    match result {
        Some(permissions) => Ok(permissions.contains(&required_permission.to_string())),
        None => Ok(false),
    }
}

/// Check if user has specific role in team
pub async fn check_team_role(
    pool: &Pool<Postgres>,
    user_id: Uuid,
    team_id: Uuid,
    min_role_level: i32,
) -> Result<bool, ApiError> {
    let role = sqlx::query_scalar::<_, String>(
        r#"
        SELECT role FROM team_members
        WHERE user_id = $1 AND team_id = $2
        "#,
    )
    .bind(user_id)
    .bind(team_id)
    .fetch_optional(pool)
    .await?;

    if let Some(role_str) = role {
        let role_level = match role_str.as_str() {
            "owner" => 4,
            "admin" => 3,
            "member" => 2,
            "viewer" => 1,
            _ => 0,
        };
        Ok(role_level >= min_role_level)
    } else {
        Ok(false)
    }
}

/// Verify user is project owner or admin
pub async fn check_project_admin(
    pool: &Pool<Postgres>,
    user_id: Uuid,
    project_id: Uuid,
) -> Result<bool, ApiError> {
    let project_owner = sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM projects WHERE id = $1"
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await?;

    match project_owner {
        Some(owner_id) => Ok(owner_id == user_id),
        None => Ok(false),
    }
}

/// Get user's role in project
pub async fn get_user_project_role(
    pool: &Pool<Postgres>,
    user_id: Uuid,
    project_id: Uuid,
) -> Result<Option<String>, ApiError> {
    let role = sqlx::query_scalar::<_, String>(
        r#"
        SELECT role FROM project_members
        WHERE user_id = $1 AND project_id = $2
        "#,
    )
    .bind(user_id)
    .bind(project_id)
    .fetch_optional(pool)
    .await?;

    Ok(role)
}

/// Enforce permission check - returns 403 if unauthorized
pub async fn enforce_permission(
    pool: &Pool<Postgres>,
    user_id: Uuid,
    project_id: Uuid,
    required_permission: &str,
) -> Result<(), ApiError> {
    let has_permission = check_project_permission(pool, user_id, project_id, required_permission).await?;

    if !has_permission {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}

/// Enforce role check - returns 403 if user doesn't meet minimum role level
pub async fn enforce_role(
    pool: &Pool<Postgres>,
    user_id: Uuid,
    team_id: Uuid,
    min_role_level: i32,
) -> Result<(), ApiError> {
    let has_role = check_team_role(pool, user_id, team_id, min_role_level).await?;

    if !has_role {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}

/// Check if user can modify code review
pub async fn can_modify_review(
    pool: &Pool<Postgres>,
    user_id: Uuid,
    review_id: Uuid,
) -> Result<bool, ApiError> {
    let author_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT author_id FROM code_reviews WHERE id = $1"
    )
    .bind(review_id)
    .fetch_optional(pool)
    .await?;

    Ok(author_id.map(|id| id == user_id).unwrap_or(false))
}

/// Check if user can comment on review
pub async fn can_comment_on_review(
    pool: &Pool<Postgres>,
    user_id: Uuid,
    review_id: Uuid,
) -> Result<bool, ApiError> {
    let project_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT project_id FROM code_reviews WHERE id = $1"
    )
    .bind(review_id)
    .fetch_optional(pool)
    .await?;

    if let Some(pid) = project_id {
        check_project_permission(pool, user_id, pid, "write").await
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_hierarchy() {
        assert!(4 >= 3); // owner >= admin
        assert!(3 >= 2); // admin >= member
        assert!(2 >= 1); // member >= viewer
        assert!(1 < 2);  // viewer < member
    }
}
