use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// User Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: User,
}

// Project Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub repository_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub repository_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
}

// Code File Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFile {
    pub id: Uuid,
    pub project_id: Uuid,
    pub file_path: String,
    pub content: String,
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFileRequest {
    pub file_path: String,
    pub content: String,
    pub language: Option<String>,
}

// Analysis Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisTask {
    pub id: Uuid,
    pub project_id: Uuid,
    pub task_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct OptimizeCodeRequest {
    pub code: String,
    pub language: String,
    pub file_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewCodeRequest {
    pub code: String,
    pub language: String,
    pub file_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RefactorCodeRequest {
    pub code: String,
    pub language: String,
    pub target_pattern: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CodeAnalysisResponse {
    pub task_id: Uuid,
    pub suggestions: Vec<String>,
    pub optimized_code: Option<String>,
    pub metrics: AnalysisMetrics,
}

#[derive(Debug, Serialize)]
pub struct AnalysisMetrics {
    pub complexity_reduction: f64,
    pub performance_gain: f64,
    pub maintainability_score: f64,
}

// Agent Models
#[derive(Debug, Deserialize)]
pub struct AgentRequest {
    pub project_id: Uuid,
    pub task_description: String,
    pub context: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AgentTaskResponse {
    pub task_id: Uuid,
    pub agent_type: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct AgentTaskStatus {
    pub task_id: Uuid,
    pub status: String,
    pub progress: f64,
    pub result: Option<serde_json::Value>,
}

// Analytics Models
#[derive(Debug, Serialize)]
pub struct DashboardMetrics {
    pub total_projects: i64,
    pub active_agents: i64,
    pub code_quality_score: f64,
    pub recent_analyses: Vec<AnalysisTask>,
}

#[derive(Debug, Serialize)]
pub struct Metric {
    pub metric_type: String,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TokenRefreshRequest {
    pub refresh_token: String,
}
