use serde::{Deserialize, Serialize};

pub struct ApiClient {
    base_url: String,
    token: Option<String>,
    http_client: reqwest::Client,
}

impl ApiClient {
    pub fn new(base_url: &str, token: Option<&str>) -> Self {
        Self {
            base_url: base_url.to_string(),
            token: token.map(|t| t.to_string()),
            http_client: reqwest::Client::new(),
        }
    }

    async fn request(&self, method: &str, endpoint: &str) -> anyhow::Result<reqwest::RequestBuilder> {
        let url = format!("{}{}", self.base_url, endpoint);
        let builder = match method {
            "GET" => self.http_client.get(&url),
            "POST" => self.http_client.post(&url),
            "PUT" => self.http_client.put(&url),
            "DELETE" => self.http_client.delete(&url),
            _ => return Err(anyhow::anyhow!("Unknown HTTP method")),
        };

        let builder = if let Some(token) = &self.token {
            builder.header("Authorization", format!("Bearer {}", token))
        } else {
            builder
        };

        Ok(builder)
    }

    pub async fn login(&self, email: &str, password: &str) -> anyhow::Result<LoginResponse> {
        let req = self.request("POST", "/api/auth/login").await?;
        let response = req
            .json(&serde_json::json!({ "email": email, "password": password }))
            .send()
            .await?;
        
        response.json().await.map_err(Into::into)
    }

    pub async fn refresh_token(&self) -> anyhow::Result<LoginResponse> {
        let req = self.request("POST", "/api/auth/refresh").await?;
        let response = req.send().await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn get_user_info(&self) -> anyhow::Result<UserInfo> {
        let req = self.request("GET", "/api/auth/me").await?;
        let response = req.send().await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn list_projects(&self) -> anyhow::Result<Vec<ProjectInfo>> {
        let req = self.request("GET", "/api/projects").await?;
        let response = req.send().await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn get_project(&self, id: &str) -> anyhow::Result<ProjectInfo> {
        let req = self.request("GET", &format!("/api/projects/{}", id)).await?;
        let response = req.send().await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn create_project(&self, name: &str, description: Option<&str>) -> anyhow::Result<ProjectInfo> {
        let req = self.request("POST", "/api/projects").await?;
        let response = req
            .json(&serde_json::json!({ "name": name, "description": description }))
            .send()
            .await?;
        
        response.json().await.map_err(Into::into)
    }

    pub async fn delete_project(&self, id: &str) -> anyhow::Result<()> {
        let req = self.request("DELETE", &format!("/api/projects/{}", id)).await?;
        req.send().await?;
        Ok(())
    }

    pub async fn deploy_code(&self, project: &str, files: &[String], message: &str) -> anyhow::Result<DeploymentResponse> {
        let req = self.request("POST", &format!("/api/projects/{}/deploy", project)).await?;
        let response = req
            .json(&serde_json::json!({ "files": files, "message": message }))
            .send()
            .await?;
        
        response.json().await.map_err(Into::into)
    }

    pub async fn pull_code(&self, project: &str) -> anyhow::Result<Vec<FileContent>> {
        let req = self.request("GET", &format!("/api/projects/{}/code", project)).await?;
        let response = req.send().await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn analyze_code(&self, project: &str) -> anyhow::Result<CodeAnalysis> {
        let req = self.request("POST", &format!("/api/projects/{}/analyze", project)).await?;
        let response = req.send().await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn get_deployment_history(&self, project: &str, limit: usize) -> anyhow::Result<Vec<DeploymentInfo>> {
        let req = self.request("GET", &format!("/api/projects/{}/deployments?limit={}", project, limit)).await?;
        let response = req.send().await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn list_agents(&self) -> anyhow::Result<Vec<AgentInfo>> {
        let req = self.request("GET", "/api/agents").await?;
        let response = req.send().await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn run_agent(&self, project: &str, agent: &str) -> anyhow::Result<AgentResult> {
        let req = self.request("POST", &format!("/api/agents/{}/run", agent)).await?;
        let response = req
            .json(&serde_json::json!({ "project_id": project }))
            .send()
            .await?;
        
        response.json().await.map_err(Into::into)
    }

    pub async fn get_agent_status(&self, agent: &str) -> anyhow::Result<AgentStatus> {
        let req = self.request("GET", &format!("/api/agents/{}/status", agent)).await?;
        let response = req.send().await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn health_check(&self) -> anyhow::Result<HealthStatus> {
        let req = self.request("GET", "/api/health").await?;
        let response = req.send().await?;
        response.json().await.map_err(Into::into)
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct DeploymentResponse {
    pub id: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct FileContent {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CodeAnalysis {
    pub lines_of_code: usize,
    pub complexity: f32,
    pub issues: usize,
}

#[derive(Debug, Deserialize)]
pub struct DeploymentInfo {
    pub id: String,
    pub status: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct AgentResult {
    pub id: String,
    pub status: String,
    pub output: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AgentStatus {
    pub status: String,
    pub last_run: String,
}

#[derive(Debug, Deserialize)]
pub struct HealthStatus {
    pub ok: bool,
    pub database_ok: bool,
    pub cache_ok: bool,
    pub agents_running: usize,
}
