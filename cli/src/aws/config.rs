use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    pub region: String,
    pub account_id: String,
    pub ecr_repository: String,
    pub ecs_cluster: String,
    pub ecs_service: String,
    pub task_family: String,
    pub task_cpu: String,
    pub task_memory: String,
    pub container_port: u16,
    pub log_group: String,
}

impl AwsConfig {
    pub fn from_env() -> Result<Self, String> {
        Ok(AwsConfig {
            region: env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            account_id: env::var("AWS_ACCOUNT_ID").map_err(|_| "AWS_ACCOUNT_ID not set")?,
            ecr_repository: env::var("ECR_REPOSITORY").map_err(|_| "ECR_REPOSITORY not set")?,
            ecs_cluster: env::var("ECS_CLUSTER").map_err(|_| "ECS_CLUSTER not set")?,
            ecs_service: env::var("ECS_SERVICE").map_err(|_| "ECS_SERVICE not set")?,
            task_family: env::var("TASK_FAMILY").map_err(|_| "TASK_FAMILY not set")?,
            task_cpu: env::var("TASK_CPU").unwrap_or_else(|_| "256".to_string()),
            task_memory: env::var("TASK_MEMORY").unwrap_or_else(|_| "512".to_string()),
            container_port: env::var("CONTAINER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            log_group: env::var("LOG_GROUP").unwrap_or_else(|_| "/ecs/compilex7".to_string()),
        })
    }

    pub fn ecr_image_uri(&self, tag: &str) -> String {
        format!(
            "{}.dkr.ecr.{}.amazonaws.com/{}:{}",
            self.account_id, self.region, self.ecr_repository, tag
        )
    }
}
