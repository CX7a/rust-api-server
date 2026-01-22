use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_addr: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiry: u64,
    pub ai_api_key: String,
    pub ai_api_url: String,
    pub log_level: String,
    pub environment: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            server_addr: env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            database_url: env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL not set"))?,
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|_| anyhow::anyhow!("JWT_SECRET not set"))?,
            jwt_expiry: env::var("JWT_EXPIRY")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()?,
            ai_api_key: env::var("AI_API_KEY")
                .map_err(|_| anyhow::anyhow!("AI_API_KEY not set"))?,
            ai_api_url: env::var("AI_API_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
        })
    }
}
