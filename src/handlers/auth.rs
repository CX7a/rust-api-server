use axum::{extract::State, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::Database,
    error::{AppError, AppResult},
    models::{AuthResponse, LoginRequest, RegisterRequest, User},
    utils::jwt,
};

pub async fn register(
    State(db): State<Arc<Database>>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    // Validate email
    if !payload.email.contains('@') {
        return Err(AppError::ValidationError("Invalid email format".to_string()));
    }

    // Hash password
    let password_hash = bcrypt::hash(&payload.password, 12)
        .map_err(|_| AppError::InternalServerError("Failed to hash password".to_string()))?;

    // Insert user into database
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, first_name, last_name) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&user_id)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .execute(db.pool())
    .await?;

    // Generate tokens
    let access_token = jwt::generate_token(&user_id.to_string(), 3600)?;
    let refresh_token = jwt::generate_token(&user_id.to_string(), 86400 * 7)?;

    let user = User {
        id: user_id,
        email: payload.email,
        first_name: payload.first_name,
        last_name: payload.last_name,
        created_at: chrono::Utc::now(),
    };

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user,
    }))
}

pub async fn login(
    State(db): State<Arc<Database>>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    // Query user from database
    let row = sqlx::query("SELECT id, email, password_hash, first_name, last_name, created_at FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(db.pool())
        .await?;

    let row = row.ok_or(AppError::AuthenticationError("Invalid credentials".to_string()))?;

    let user_id: Uuid = row.get("id");
    let stored_hash: String = row.get("password_hash");

    // Verify password
    if !bcrypt::verify(&payload.password, &stored_hash)
        .map_err(|_| AppError::InternalServerError("Password verification failed".to_string()))?
    {
        return Err(AppError::AuthenticationError("Invalid credentials".to_string()));
    }

    // Generate tokens
    let access_token = jwt::generate_token(&user_id.to_string(), 3600)?;
    let refresh_token = jwt::generate_token(&user_id.to_string(), 86400 * 7)?;

    let user = User {
        id: user_id,
        email: row.get("email"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        created_at: row.get("created_at"),
    };

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user,
    }))
}

pub async fn refresh_token(
    State(db): State<Arc<Database>>,
    Json(payload): Json<crate::models::TokenRefreshRequest>,
) -> AppResult<Json<AuthResponse>> {
    // Verify refresh token
    let claims = jwt::verify_token(&payload.refresh_token)?;

    // Fetch user from database
    let row = sqlx::query("SELECT id, email, first_name, last_name, created_at FROM users WHERE id = $1")
        .bind(&claims.sub)
        .fetch_optional(db.pool())
        .await?;

    let row = row.ok_or(AppError::AuthenticationError("User not found".to_string()))?;

    let user_id: Uuid = row.get("id");

    // Generate new tokens
    let access_token = jwt::generate_token(&user_id.to_string(), 3600)?;
    let new_refresh_token = jwt::generate_token(&user_id.to_string(), 86400 * 7)?;

    let user = User {
        id: user_id,
        email: row.get("email"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        created_at: row.get("created_at"),
    };

    Ok(Json(AuthResponse {
        access_token,
        refresh_token: new_refresh_token,
        user,
    }))
}

pub async fn logout() -> &'static str {
    "Logged out successfully"
}
