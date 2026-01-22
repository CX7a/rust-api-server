use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(String),
    ValidationError(String),
    AuthenticationError(String),
    AuthorizationError(String),
    NotFoundError(String),
    ConflictError(String),
    ExternalApiError(String),
    InternalServerError(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, code) = match self {
            AppError::DatabaseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                msg,
                "DATABASE_ERROR".to_string(),
            ),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg, "VALIDATION_ERROR".to_string()),
            AppError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg, "AUTHENTICATION_ERROR".to_string()),
            AppError::AuthorizationError(msg) => (StatusCode::FORBIDDEN, msg, "AUTHORIZATION_ERROR".to_string()),
            AppError::NotFoundError(msg) => (StatusCode::NOT_FOUND, msg, "NOT_FOUND_ERROR".to_string()),
            AppError::ConflictError(msg) => (StatusCode::CONFLICT, msg, "CONFLICT_ERROR".to_string()),
            AppError::ExternalApiError(msg) => (
                StatusCode::BAD_GATEWAY,
                msg,
                "EXTERNAL_API_ERROR".to_string(),
            ),
            AppError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                msg,
                "INTERNAL_SERVER_ERROR".to_string(),
            ),
        };

        let body = Json(ErrorResponse {
            code,
            message: error_message,
        });

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        AppError::DatabaseError("Database operation failed".to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        tracing::error!("External API error: {:?}", err);
        AppError::ExternalApiError("External API call failed".to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
