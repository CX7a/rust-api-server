use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    // Skip authentication for public endpoints
    let path = request.uri().path();
    if is_public_route(path) {
        return next.run(request).await;
    }

    // Extract authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            // Validate token (simplified - should use actual JWT secret)
            if validate_token(token).is_ok() {
                return next.run(request).await;
            }
        }
    }

    Response::builder()
        .status(401)
        .body(Body::from("Unauthorized"))
        .unwrap()
}

fn is_public_route(path: &str) -> bool {
    matches!(
        path,
        "/health" | "/auth/register" | "/auth/login" | "/auth/refresh"
    )
}

fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    // This is a simplified version - in production, use your actual JWT secret
    let secret = std::env::var("JWT_SECRET").unwrap_or_default();
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}
