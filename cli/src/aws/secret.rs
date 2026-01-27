use std::collections::HashMap;
use std::process::Command;

pub struct SecretsManager;

impl SecretsManager {
    pub async fn get_secrets(secret_name: &str, region: &str) -> Result<HashMap<String, String>, String> {
        let output = Command::new("aws")
            .args(&[
                "secretsmanager",
                "get-secret-value",
                "--secret-id",
                secret_name,
                "--region",
                region,
                "--query",
                "SecretString",
                "--output",
                "text",
            ])
            .output()
            .map_err(|e| format!("Failed to get secrets: {}", e))?;

        if !output.status.success() {
            return Err(format!("Failed to retrieve secrets: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let secret_json = String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8: {}", e))?;

        // Parse JSON to HashMap (simplified)
        let mut secrets = HashMap::new();
        for line in secret_json.lines() {
            if let Some((key, val)) = line.split_once(':') {
                secrets.insert(
                    key.trim().trim_matches('"').to_string(),
                    val.trim().trim_matches('"').to_string(),
                );
            }
        }

        Ok(secrets)
    }

    pub async fn set_secret(secret_name: &str, secret_value: &str, region: &str) -> Result<(), String> {
        let output = Command::new("aws")
            .args(&[
                "secretsmanager",
                "create-secret",
                "--name",
                secret_name,
                "--secret-string",
                secret_value,
                "--region",
                region,
            ])
            .output()
            .map_err(|e| format!("Failed to create secret: {}", e))?;

        if !output.status.success() {
            // Try updating if it exists
            let update_output = Command::new("aws")
                .args(&[
                    "secretsmanager",
                    "update-secret",
                    "--secret-id",
                    secret_name,
                    "--secret-string",
                    secret_value,
                    "--region",
                    region,
                ])
                .output()
                .map_err(|e| format!("Failed to update secret: {}", e))?;

            if !update_output.status.success() {
                return Err(format!("Failed to update secret: {}", String::from_utf8_lossy(&update_output.stderr)));
            }
        }

        Ok(())
    }

    pub async fn delete_secret(secret_name: &str, region: &str) -> Result<(), String> {
        let output = Command::new("aws")
            .args(&[
                "secretsmanager",
                "delete-secret",
                "--secret-id",
                secret_name,
                "--force-delete-without-recovery",
                "--region",
                region,
            ])
            .output()
            .map_err(|e| format!("Failed to delete secret: {}", e))?;

        if !output.status.success() {
            return Err(format!("Failed to delete secret: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(())
    }
}
