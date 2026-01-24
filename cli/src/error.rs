use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
