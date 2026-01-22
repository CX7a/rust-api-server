use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;
use crate::models::collaboration::Role;

#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub role: Role,
}

pub async fn rbac_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, String> {
    // Extract user context from request headers/JWT
    // This will be populated by the JWT middleware first
    if let Some(user_context) = request.extensions().get::<UserContext>() {
        Ok(next.run(request).await)
    } else {
        Err("Unauthorized".to_string())
    }
}

pub fn check_permission(user_role: Role, required_role: Role) -> bool {
    user_role.hierarchy_level() >= required_role.hierarchy_level()
}
