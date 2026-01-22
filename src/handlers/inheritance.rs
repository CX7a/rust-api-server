use axum::{
    extract::{Path, State, Query, Json},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;

use crate::error::ApiError;
use crate::models::inheritance::{
    TeamHierarchy, ProjectHierarchy, CreateTeamHierarchyRequest,
    CreateProjectHierarchyRequest, PermissionRule, CreatePermissionRuleRequest,
    UpdatePermissionRuleRequest, AuditLog, AuditLogQuery, ResolvedPermissions,
};
use crate::middleware::rbac;
use crate::services::InheritanceEngine;
use crate::models::inheritance::InheritanceConfig;

/// Create team hierarchy relationship
pub async fn create_team_hierarchy(
    State(pool): State<Pool<Postgres>>,
    user_id: Uuid,
    Json(req): Json<CreateTeamHierarchyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify user is owner of parent team
    rbac::enforce_role(&pool, user_id, req.parent_team_id, 4).await?;

    let hierarchy_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO team_hierarchy (id, parent_team_id, child_team_id, inheritance_enabled)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(hierarchy_id)
    .bind(&req.parent_team_id)
    .bind(&req.child_team_id)
    .bind(req.inheritance_enabled.unwrap_or(true))
    .execute(&pool)
    .await?;

    log_audit(&pool, user_id, "create_team_hierarchy", "team_hierarchy", hierarchy_id, None, None).await?;

    let hierarchy = TeamHierarchy {
        id: hierarchy_id,
        parent_team_id: Some(req.parent_team_id),
        child_team_id: req.child_team_id,
        inheritance_enabled: req.inheritance_enabled.unwrap_or(true),
        created_at: Utc::now(),
    };

    Ok((StatusCode::CREATED, Json(hierarchy)))
}

/// Create project hierarchy relationship
pub async fn create_project_hierarchy(
    State(pool): State<Pool<Postgres>>,
    user_id: Uuid,
    Json(req): Json<CreateProjectHierarchyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify user has admin permission on parent project
    rbac::enforce_permission(&pool, user_id, req.parent_project_id, "admin").await?;

    let hierarchy_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO project_hierarchy (id, parent_project_id, child_project_id, inheritance_enabled)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(hierarchy_id)
    .bind(&req.parent_project_id)
    .bind(&req.child_project_id)
    .bind(req.inheritance_enabled.unwrap_or(true))
    .execute(&pool)
    .await?;

    log_audit(&pool, user_id, "create_project_hierarchy", "project_hierarchy", hierarchy_id, None, None).await?;

    let hierarchy = ProjectHierarchy {
        id: hierarchy_id,
        parent_project_id: Some(req.parent_project_id),
        child_project_id: req.child_project_id,
        inheritance_enabled: req.inheritance_enabled.unwrap_or(true),
        created_at: Utc::now(),
    };

    Ok((StatusCode::CREATED, Json(hierarchy)))
}

/// Get resolved permissions for user on resource
pub async fn get_resolved_permissions(
    State(pool): State<Pool<Postgres>>,
    Path((resource_id, resource_type)): Path<(Uuid, String)>,
    user_id: Uuid,
) -> Result<impl IntoResponse, ApiError> {
    // Verify user has access
    rbac::enforce_permission_with_inheritance(&pool, user_id, resource_id, &resource_type, "read")
        .await?;

    let resolved = rbac::get_resolved_permissions(&pool, user_id, resource_id, &resource_type).await?;

    Ok(Json(resolved))
}

/// Create permission rule for role
pub async fn create_permission_rule(
    State(pool): State<Pool<Postgres>>,
    user_id: Uuid,
    Json(req): Json<CreatePermissionRuleRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify user can manage permissions
    if let Some(team_id) = req.team_id {
        rbac::enforce_role(&pool, user_id, team_id, 3).await?; // Admin level
    } else if let Some(project_id) = req.project_id {
        rbac::enforce_permission(&pool, user_id, project_id, "admin").await?;
    } else {
        return Err(ApiError::BadRequest("Team or Project ID required".to_string()));
    }

    let rule_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO permission_rules (id, team_id, project_id, role, permissions, description, priority)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(rule_id)
    .bind(req.team_id)
    .bind(req.project_id)
    .bind(&req.role)
    .bind(serde_json::to_value(&req.permissions).unwrap())
    .bind(&req.description)
    .bind(req.priority.unwrap_or(0))
    .execute(&pool)
    .await?;

    let rule = PermissionRule {
        id: rule_id,
        team_id: req.team_id,
        project_id: req.project_id,
        role: req.role.clone(),
        permissions: req.permissions,
        description: req.description,
        priority: req.priority.unwrap_or(0),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    Ok((StatusCode::CREATED, Json(rule)))
}

/// Update permission rule
pub async fn update_permission_rule(
    State(pool): State<Pool<Postgres>>,
    Path(rule_id): Path<Uuid>,
    user_id: Uuid,
    Json(req): Json<UpdatePermissionRuleRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Get rule to verify access
    let rule = sqlx::query_as::<_, PermissionRule>(
        "SELECT * FROM permission_rules WHERE id = $1"
    )
    .bind(rule_id)
    .fetch_optional(&pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    if let Some(team_id) = rule.team_id {
        rbac::enforce_role(&pool, user_id, team_id, 3).await?;
    } else if let Some(project_id) = rule.project_id {
        rbac::enforce_permission(&pool, user_id, project_id, "admin").await?;
    }

    sqlx::query(
        r#"
        UPDATE permission_rules
        SET permissions = COALESCE($1, permissions),
            description = COALESCE($2, description),
            priority = COALESCE($3, priority),
            updated_at = $4
        WHERE id = $5
        "#,
    )
    .bind(req.permissions.as_ref().map(|p| serde_json::to_value(p).ok()).flatten())
    .bind(&req.description)
    .bind(req.priority)
    .bind(Utc::now())
    .bind(rule_id)
    .execute(&pool)
    .await?;

    Ok(StatusCode::OK)
}

/// Delete permission rule
pub async fn delete_permission_rule(
    State(pool): State<Pool<Postgres>>,
    Path(rule_id): Path<Uuid>,
    user_id: Uuid,
) -> Result<impl IntoResponse, ApiError> {
    // Get rule to verify access
    let rule = sqlx::query_as::<_, PermissionRule>(
        "SELECT * FROM permission_rules WHERE id = $1"
    )
    .bind(rule_id)
    .fetch_optional(&pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    if let Some(team_id) = rule.team_id {
        rbac::enforce_role(&pool, user_id, team_id, 4).await?; // Owner level
    } else if let Some(project_id) = rule.project_id {
        rbac::enforce_permission(&pool, user_id, project_id, "admin").await?;
    }

    sqlx::query("DELETE FROM permission_rules WHERE id = $1")
        .bind(rule_id)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get audit logs
pub async fn get_audit_logs(
    State(pool): State<Pool<Postgres>>,
    user_id: Uuid,
    Query(query): Query<AuditLogQuery>,
) -> Result<impl IntoResponse, ApiError> {
    // User can only view their own actions or if they have audit permission
    let mut sql = "SELECT * FROM audit_logs WHERE 1=1".to_string();
    let mut conditions = vec![];

    if let Some(actor_id) = query.actor_id {
        if actor_id != user_id {
            // Check if current user has view_audit permission
            // For now, restrict to own actions
            return Err(ApiError::Forbidden);
        }
        conditions.push(format!("actor_id = '{}'", actor_id));
    }

    if let Some(resource_type) = query.resource_type {
        conditions.push(format!("resource_type = '{}'", resource_type));
    }

    if let Some(resource_id) = query.resource_id {
        conditions.push(format!("resource_id = '{}'", resource_id));
    }

    for condition in conditions {
        sql.push_str(&format!(" AND {}", condition));
    }

    sql.push_str(" ORDER BY created_at DESC LIMIT 100");

    let logs = sqlx::query_as::<_, AuditLog>(&sql)
        .fetch_all(&pool)
        .await?;

    Ok(Json(logs))
}

/// Get hierarchy tree
pub async fn get_hierarchy_tree(
    State(pool): State<Pool<Postgres>>,
    Path((resource_id, resource_type)): Path<(Uuid, String)>,
    user_id: Uuid,
) -> Result<impl IntoResponse, ApiError> {
    // Verify access
    rbac::enforce_permission_with_inheritance(&pool, user_id, resource_id, &resource_type, "read")
        .await?;

    let engine = InheritanceEngine::new(
        Arc::new(pool.clone()),
        Some(InheritanceConfig::default()),
    );

    let tree = engine
        .build_hierarchy_tree(resource_id, &resource_type, "root")
        .await
        .map_err(|_| ApiError::BadRequest("Failed to build hierarchy".to_string()))?;

    Ok(Json(tree))
}

/// Log audit event
async fn log_audit(
    pool: &Pool<Postgres>,
    actor_id: Uuid,
    action: &str,
    resource_type: &str,
    resource_id: Uuid,
    old_value: Option<serde_json::Value>,
    new_value: Option<serde_json::Value>,
) -> Result<(), ApiError> {
    sqlx::query(
        r#"
        INSERT INTO audit_logs (id, actor_id, action, resource_type, resource_id, old_value, new_value, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(actor_id)
    .bind(action)
    .bind(resource_type)
    .bind(resource_id)
    .bind(old_value)
    .bind(new_value)
    .bind(Utc::now())
    .execute(pool)
    .await?;

    Ok(())
}
