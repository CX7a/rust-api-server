use crate::aws::AwsConfig;
use std::process::Command;

pub struct EcrManager {
    config: AwsConfig,
}

impl EcrManager {
    pub fn new(config: AwsConfig) -> Self {
        EcrManager { config }
    }

    pub async fn build_and_push(&self, dockerfile_path: &str, tag: &str) -> Result<String, String> {
        println!("Building Docker image...");
        
        // Build image
        let build_output = Command::new("docker")
            .args(&["build", "-t", &format!("{}:{}", self.config.ecr_repository, tag), "-f", dockerfile_path, "."])
            .output()
            .map_err(|e| format!("Docker build failed: {}", e))?;

        if !build_output.status.success() {
            return Err(format!("Docker build failed: {}", String::from_utf8_lossy(&build_output.stderr)));
        }

        println!("Logging in to ECR...");
        self.ecr_login().await?;

        let image_uri = self.config.ecr_image_uri(tag);
        
        println!("Tagging image: {}", image_uri);
        let tag_output = Command::new("docker")
            .args(&["tag", &format!("{}:{}", self.config.ecr_repository, tag), &image_uri])
            .output()
            .map_err(|e| format!("Docker tag failed: {}", e))?;

        if !tag_output.status.success() {
            return Err(format!("Docker tag failed: {}", String::from_utf8_lossy(&tag_output.stderr)));
        }

        println!("Pushing to ECR: {}", image_uri);
        let push_output = Command::new("docker")
            .args(&["push", &image_uri])
            .output()
            .map_err(|e| format!("Docker push failed: {}", e))?;

        if !push_output.status.success() {
            return Err(format!("Docker push failed: {}", String::from_utf8_lossy(&push_output.stderr)));
        }

        println!("Successfully pushed image to ECR");
        Ok(image_uri)
    }

    async fn ecr_login(&self) -> Result<(), String> {
        let auth_output = Command::new("aws")
            .args(&[
                "ecr",
                "get-login-password",
                "--region",
                &self.config.region,
            ])
            .output()
            .map_err(|e| format!("ECR login failed: {}", e))?;

        if !auth_output.status.success() {
            return Err("Failed to get ECR login token".to_string());
        }

        let password = String::from_utf8(auth_output.stdout)
            .map_err(|e| format!("Invalid UTF-8: {}", e))?
            .trim()
            .to_string();

        let registry = format!("{}.dkr.ecr.{}.amazonaws.com", self.config.account_id, self.config.region);

        let login_output = Command::new("docker")
            .args(&["login", "--username", "AWS", "--password-stdin", &registry])
            .stdin(std::process::Stdio::piped())
            .output()
            .map_err(|e| format!("Docker login failed: {}", e))?;

        if !login_output.status.success() {
            return Err(format!("Docker login failed: {}", String::from_utf8_lossy(&login_output.stderr)));
        }

        Ok(())
    }
}
