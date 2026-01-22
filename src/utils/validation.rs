use crate::error::{AppError, AppResult};

pub fn validate_email(email: &str) -> AppResult<()> {
    if email.is_empty() || !email.contains('@') {
        return Err(AppError::ValidationError("Invalid email format".to_string()));
    }
    Ok(())
}

pub fn validate_password(password: &str) -> AppResult<()> {
    if password.len() < 8 {
        return Err(AppError::ValidationError(
            "Password must be at least 8 characters".to_string(),
        ));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(AppError::ValidationError(
            "Password must contain an uppercase letter".to_string(),
        ));
    }
    if !password.chars().any(|c| c.is_numeric()) {
        return Err(AppError::ValidationError(
            "Password must contain a number".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_project_name(name: &str) -> AppResult<()> {
    if name.is_empty() || name.len() > 255 {
        return Err(AppError::ValidationError(
            "Project name must be between 1 and 255 characters".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("").is_err());
    }

    #[test]
    fn test_password_validation() {
        assert!(validate_password("Secure123").is_ok());
        assert!(validate_password("short").is_err());
        assert!(validate_password("nouppercase123").is_err());
    }
}
