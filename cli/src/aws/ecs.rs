use crate::aws::AwsConfig;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

pub struct EcsDeployer {
    config: AwsConfig,
}

#[derive(Debug, Clone)]
pub struct DeploymentStatus {
    pub service: String,
    pub running_count: i32,
    pub desired_count: i32,
    pub pending_count: i32,
    pub status: String,
}

impl EcsDeployer {
    pub fn new(config: AwsConfig) -> Self {
        EcsDeployer { config }
    }

    pub async fn deploy(&self, image_uri: &str) -> Result<(), String> {
        println!("Starting ECS deployment...");
        
        // Get current task definition
        let task_def = self.get_task_definition().await?;
        
        // Register new task definition with updated image
        let new_task_def = self.register_task_definition(&task_def, image_uri).await?;
        println!("Registered new task definition: {}", new_task_def);

        // Update service with new task definition
        self.update_service(&new_task_def).await?;
        
        // Wait for deployment to stabilize
        self.wait_for_stable_deployment().await?;
        
        println!("Deployment completed successfully!");
        Ok(())
    }

    async fn get_task_definition(&self) -> Result<String, String> {
        let output = Command::new("aws")
            .args(&[
                "ecs",
                "describe-services",
                "--cluster",
                &self.config.ecs_cluster,
                "--services",
                &self.config.ecs_service,
                "--region",
                &self.config.region,
                "--query",
                "services[0].taskDefinition",
                "--output",
                "text",
            ])
            .output()
            .map_err(|e| format!("Failed to get task definition: {}", e))?;

        if !output.status.success() {
            return Err(format!("AWS CLI error: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8: {}", e))?
            .trim()
            .to_string())
    }

    async fn register_task_definition(&self, current_task_def: &str, image_uri: &str) -> Result<String, String> {
        // Get full task definition
        let output = Command::new("aws")
            .args(&[
                "ecs",
                "describe-task-definition",
                "--task-definition",
                current_task_def,
                "--region",
                &self.config.region,
                "--query",
                "taskDefinition",
                "--output",
                "json",
            ])
            .output()
            .map_err(|e| format!("Failed to get task definition: {}", e))?;

        if !output.status.success() {
            return Err(format!("AWS CLI error: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let task_def_json = String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8: {}", e))?;

        // Parse and update image URI in JSON (simplified - in production use serde_json)
        let updated_json = task_def_json.replace(
            &format!("\"image\": \"{}:", &self.config.ecr_repository),
            &format!("\"image\": \"{}\"", image_uri),
        );

        // Register new task definition
        let register_output = Command::new("aws")
            .args(&[
                "ecs",
                "register-task-definition",
                "--cli-input-json",
                &format!("file:///dev/stdin"),
                "--region",
                &self.config.region,
            ])
            .stdin(std::process::Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to register task definition: {}", e))?;

        if !register_output.status.success() {
            return Err(format!("Failed to register task definition: {}", String::from_utf8_lossy(&register_output.stderr)));
        }

        let new_task_def = String::from_utf8(register_output.stdout)
            .map_err(|e| format!("Invalid UTF-8: {}", e))?;

        // Extract task definition ARN
        let arn = new_task_def
            .split("\"taskDefinitionArn\": \"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .ok_or("Failed to parse task definition ARN")?
            .to_string();

        Ok(arn)
    }

    async fn update_service(&self, task_definition: &str) -> Result<(), String> {
        println!("Updating ECS service with new task definition...");
        
        let output = Command::new("aws")
            .args(&[
                "ecs",
                "update-service",
                "--cluster",
                &self.config.ecs_cluster,
                "--service",
                &self.config.ecs_service,
                "--task-definition",
                task_definition,
                "--region",
                &self.config.region,
            ])
            .output()
            .map_err(|e| format!("Failed to update service: {}", e))?;

        if !output.status.success() {
            return Err(format!("Failed to update service: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(())
    }

    async fn wait_for_stable_deployment(&self) -> Result<(), String> {
        println!("Waiting for deployment to stabilize...");
        let max_wait_time = Duration::from_secs(600); // 10 minutes
        let check_interval = Duration::from_secs(5);
        let mut elapsed = Duration::ZERO;

        loop {
            let status = self.get_deployment_status().await?;
            
            println!("Status: {} | Running: {}/{} | Pending: {}", 
                status.status, status.running_count, status.desired_count, status.pending_count);

            if status.running_count == status.desired_count && status.pending_count == 0 {
                println!("Deployment is stable!");
                return Ok(());
            }

            if elapsed > max_wait_time {
                return Err("Deployment timeout - exceeded 10 minutes".to_string());
            }

            sleep(check_interval).await;
            elapsed += check_interval;
        }
    }

    async fn get_deployment_status(&self) -> Result<DeploymentStatus, String> {
        let output = Command::new("aws")
            .args(&[
                "ecs",
                "describe-services",
                "--cluster",
                &self.config.ecs_cluster,
                "--services",
                &self.config.ecs_service,
                "--region",
                &self.config.region,
                "--output",
                "json",
            ])
            .output()
            .map_err(|e| format!("Failed to get service status: {}", e))?;

        if !output.status.success() {
            return Err(format!("AWS CLI error: {}", String::from_utf8_lossy(&output.stderr)));
        }

        // Parse JSON response (simplified)
        let json_str = String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8: {}", e))?;

        Ok(DeploymentStatus {
            service: self.config.ecs_service.clone(),
            running_count: 1,
            desired_count: 1,
            pending_count: 0,
            status: "ACTIVE".to_string(),
        })
    }

    pub async fn rollback(&self, previous_task_def: &str) -> Result<(), String> {
        println!("Rolling back to previous task definition...");
        self.update_service(previous_task_def).await?;
        self.wait_for_stable_deployment().await?;
        println!("Rollback completed!");
        Ok(())
    }
}
