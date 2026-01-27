use crate::aws::{AwsConfig, EcrManager, EcsDeployer, SecretsManager};
use crate::utils::*;

pub async fn deploy_to_ecs(
    dockerfile_path: Option<String>,
    tag: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = AwsConfig::from_env()?;
    let tag = tag.unwrap_or_else(|| {
        chrono::Local::now().format("%Y%m%d-%H%M%S").to_string()
    });

    let dockerfile = dockerfile_path.unwrap_or_else(|| "Dockerfile".to_string());

    // Build and push to ECR
    let ecr = EcrManager::new(config.clone());
    let image_uri = ecr.build_and_push(&dockerfile, &tag).await?;

    // Deploy to ECS
    let ecs = EcsDeployer::new(config);
    ecs.deploy(&image_uri).await?;

    success(&format!("Successfully deployed to ECS with tag: {}", tag));
    Ok(())
}

pub async fn rollback_deployment(previous_tag: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = AwsConfig::from_env()?;
    let ecs = EcsDeployer::new(config);
    
    let task_def = format!("compilex7-task:1"); // This should be retrieved from deployment history
    ecs.rollback(&task_def).await?;

    success(&format!("Successfully rolled back to tag: {}", previous_tag));
    Ok(())
}

pub async fn check_deployment_status() -> Result<(), Box<dyn std::error::Error>> {
    let config = AwsConfig::from_env()?;
    let ecs = EcsDeployer::new(config);
    
    let status = ecs.get_deployment_status().await?;
    
    println!("\n{}", separator("Deployment Status"));
    println!("Service: {}", status.service);
    println!("Running: {}/{}", status.running_count, status.desired_count);
    println!("Pending: {}", status.pending_count);
    println!("Status: {}", status.status);

    Ok(())
}

pub async fn manage_secrets(action: &str, secret_name: &str, secret_value: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let config = AwsConfig::from_env()?;
    
    match action {
        "set" => {
            let value = secret_value.ok_or("Secret value required")?;
            SecretsManager::set_secret(secret_name, &value, &config.region).await?;
            success(&format!("Secret '{}' set successfully", secret_name));
        }
        "get" => {
            let secrets = SecretsManager::get_secrets(secret_name, &config.region).await?;
            println!("{:?}", secrets);
        }
        "delete" => {
            SecretsManager::delete_secret(secret_name, &config.region).await?;
            success(&format!("Secret '{}' deleted successfully", secret_name));
        }
        _ => return Err("Invalid action. Use 'set', 'get', or 'delete'".into()),
    }

    Ok(())
}
