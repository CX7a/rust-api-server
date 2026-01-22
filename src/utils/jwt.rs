use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn generate_token(user_id: &str, expires_in: i64) -> AppResult<String> {
    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::InternalServerError("JWT_SECRET not configured".to_string()))?;

    let now = Utc::now();
    let exp = (now + Duration::seconds(expires_in)).timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!("Token encoding error: {:?}", e);
        AppError::InternalServerError("Failed to generate token".to_string())
    })
}

pub fn verify_token(token: &str) -> AppResult<Claims> {
    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::InternalServerError("JWT_SECRET not configured".to_string()))?;

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| {
        tracing::error!("Token verification error: {:?}", e);
        AppError::AuthenticationError("Invalid token".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation_and_verification() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing");
        let user_id = "test_user";
        let token = generate_token(user_id, 3600).unwrap();
        let claims = verify_token(&token).unwrap();
        assert_eq!(claims.sub, user_id);
    }
}
