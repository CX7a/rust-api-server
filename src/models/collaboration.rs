use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ============ Team & RBAC Models ============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TeamRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

impl TeamRole {
    pub fn as_str(&self) -> &str {
        match self {
            TeamRole::Owner => "owner",
            TeamRole::Admin => "admin",
            TeamRole::Member => "member",
            TeamRole::Viewer => "viewer",
        }
    }

    pub fn hierarchy_level(&self) -> i32 {
        match self {
            TeamRole::Owner => 4,
            TeamRole::Admin => 3,
            TeamRole::Member => 2,
            TeamRole::Viewer => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTeamRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddTeamMemberRequest {
    pub user_id: Uuid,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTeamMemberRequest {
    pub role: String,
}

// ============ Project RBAC Models ============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectPermission {
    Read,
    Write,
    Admin,
    Delete,
}

impl ProjectPermission {
    pub fn as_str(&self) -> &str {
        match self {
            ProjectPermission::Read => "read",
            ProjectPermission::Write => "write",
            ProjectPermission::Admin => "admin",
            ProjectPermission::Delete => "delete",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMember {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub permissions: Vec<String>,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddProjectMemberRequest {
    pub user_id: Uuid,
    pub role: String,
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectMemberRequest {
    pub role: Option<String>,
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct PermissionCheck {
    pub user_id: Uuid,
    pub project_id: Uuid,
    pub has_permission: bool,
    pub permissions: Vec<String>,
}

// ============ Code Review Models ============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewStatus {
    Open,
    Approved,
    ChangesRequested,
    Merged,
    Closed,
}

impl ReviewStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ReviewStatus::Open => "open",
            ReviewStatus::Approved => "approved",
            ReviewStatus::ChangesRequested => "changes_requested",
            ReviewStatus::Merged => "merged",
            ReviewStatus::Closed => "closed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReview {
    pub id: Uuid,
    pub project_id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub source_branch: Option<String>,
    pub target_branch: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub id: Uuid,
    pub review_id: Uuid,
    pub author_id: Uuid,
    pub file_path: Option<String>,
    pub line_number: Option<i32>,
    pub content: String,
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalStatus {
    Approved,
    ChangesRequested,
    Commented,
}

impl ApprovalStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ApprovalStatus::Approved => "approved",
            ApprovalStatus::ChangesRequested => "changes_requested",
            ApprovalStatus::Commented => "commented",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewApproval {
    pub id: Uuid,
    pub review_id: Uuid,
    pub reviewer_id: Uuid,
    pub status: String,
    pub comments: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCodeReviewRequest {
    pub title: String,
    pub description: Option<String>,
    pub source_branch: Option<String>,
    pub target_branch: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCodeReviewRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddReviewCommentRequest {
    pub file_path: Option<String>,
    pub line_number: Option<i32>,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReviewCommentRequest {
    pub content: Option<String>,
    pub resolved: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitApprovalRequest {
    pub status: String,
    pub comments: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DiffStat {
    pub file_path: String,
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Serialize)]
pub struct CodeReviewDetails {
    pub review: CodeReview,
    pub comments: Vec<ReviewComment>,
    pub approvals: Vec<ReviewApproval>,
    pub diff_stats: Vec<DiffStat>,
}

// ============ Real-Time Collaboration Models ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborativeSession {
    pub id: Uuid,
    pub project_id: Uuid,
    pub file_id: Uuid,
    pub session_token: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionParticipant {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub cursor_position: Option<i32>,
    pub selection_start: Option<i32>,
    pub selection_end: Option<i32>,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVersion {
    pub id: Uuid,
    pub file_id: Uuid,
    pub version_number: i32,
    pub content: String,
    pub author_id: Uuid,
    pub change_description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCollaborativeSessionRequest {
    pub file_id: Uuid,
    pub expires_in_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorUpdate {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub cursor_position: i32,
    pub selection_start: Option<i32>,
    pub selection_end: Option<i32>,
}

// ============ Operational Transformation Models ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOperation {
    pub id: String,
    pub version: u32,
    pub timestamp: DateTime<Utc>,
    pub user_id: Uuid,
    pub operation: OperationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OperationType {
    #[serde(rename = "insert")]
    Insert { position: usize, content: String },
    #[serde(rename = "delete")]
    Delete { position: usize, length: usize },
    #[serde(rename = "replace")]
    Replace {
        position: usize,
        old_content: String,
        new_content: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetection {
    pub session_id: Uuid,
    pub conflicting_operations: Vec<DocumentOperation>,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ConflictResolution {
    pub version: u32,
    pub resolved_content: String,
    pub conflicting_operations: Vec<DocumentOperation>,
    pub resolution_strategy: String,
}

// ============ WebSocket Message Models ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub event_type: String,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationEvent {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
}
