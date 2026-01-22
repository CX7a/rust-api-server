use bcrypt::{hash, verify};
use crate::error::{AppError, AppResult};

pub fn hash_password(password: &str) -> AppResult<String> {
    hash(password, 12).map_err(|e| {
        tracing::error!("Password hashing error: {:?}", e);
        AppError::InternalServerError("Failed to hash password".to_string())
    })
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    verify(password, hash).map_err(|e| {
        tracing::error!("Password verification error: {:?}", e);
        AppError::InternalServerError("Password verification failed".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing_and_verification() {
        let password = "TestPassword123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("WrongPassword", &hash).unwrap());
    }
}
