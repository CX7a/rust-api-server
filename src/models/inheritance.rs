use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ============ Hierarchy Models ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamHierarchy {
    pub id: Uuid,
    pub parent_team_id: Option<Uuid>,
    pub child_team_id: Uuid,
    pub inheritance_enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHierarchy {
    pub id: Uuid,
    pub parent_project_id: Option<Uuid>,
    pub child_project_id: Uuid,
    pub inheritance_enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamHierarchyRequest {
    pub parent_team_id: Uuid,
    pub child_team_id: Uuid,
    pub inheritance_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectHierarchyRequest {
    pub parent_project_id: Uuid,
    pub child_project_id: Uuid,
    pub inheritance_enabled: Option<bool>,
}

// ============ Inherited Permissions Models ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritedPermission {
    pub id: Uuid,
    pub source_id: Uuid,
    pub source_type: String, // "team" or "project"
    pub target_id: Uuid,
    pub target_type: String,
    pub user_id: Uuid,
    pub role: String,
    pub permissions: Vec<String>,
    pub inheritance_depth: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    pub id: Uuid,
    pub team_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub role: String,
    pub permissions: Vec<String>,
    pub description: Option<String>,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePermissionRuleRequest {
    pub team_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub role: String,
    pub permissions: Vec<String>,
    pub description: Option<String>,
    pub priority: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePermissionRuleRequest {
    pub permissions: Option<Vec<String>>,
    pub description: Option<String>,
    pub priority: Option<i32>,
}

// ============ Inheritance Resolution Models ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedPermissions {
    pub user_id: Uuid,
    pub resource_id: Uuid,
    pub resource_type: String,
    pub direct_permissions: Vec<String>,
    pub inherited_permissions: Vec<InheritedPermissionInfo>,
    pub effective_permissions: Vec<String>,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritedPermissionInfo {
    pub source_id: Uuid,
    pub source_type: String,
    pub permissions: Vec<String>,
    pub depth: i32,
    pub from_role: String,
}

#[derive(Debug, Serialize)]
pub struct HierarchyTree {
    pub id: Uuid,
    pub name: String,
    pub resource_type: String,
    pub children: Vec<HierarchyTree>,
    pub permissions_inherited: bool,
}

// ============ Audit Log Models ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub actor_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub actor_id: Option<Uuid>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ============ Permission Models ============

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Permission {
    pub name: String,
    pub description: String,
    pub category: String,
}

impl Permission {
    pub fn read() -> Self {
        Permission {
            name: "read".to_string(),
            description: "Read access to resource".to_string(),
            category: "basic".to_string(),
        }
    }

    pub fn write() -> Self {
        Permission {
            name: "write".to_string(),
            description: "Write access to resource".to_string(),
            category: "basic".to_string(),
        }
    }

    pub fn admin() -> Self {
        Permission {
            name: "admin".to_string(),
            description: "Admin access to resource".to_string(),
            category: "admin".to_string(),
        }
    }

    pub fn delete() -> Self {
        Permission {
            name: "delete".to_string(),
            description: "Delete access to resource".to_string(),
            category: "destructive".to_string(),
        }
    }

    pub fn invite() -> Self {
        Permission {
            name: "invite".to_string(),
            description: "Invite members".to_string(),
            category: "management".to_string(),
        }
    }

    pub fn manage_roles() -> Self {
        Permission {
            name: "manage_roles".to_string(),
            description: "Manage user roles".to_string(),
            category: "management".to_string(),
        }
    }

    pub fn view_audit() -> Self {
        Permission {
            name: "view_audit".to_string(),
            description: "View audit logs".to_string(),
            category: "audit".to_string(),
        }
    }

    pub fn all() -> Vec<Permission> {
        vec![
            Self::read(),
            Self::write(),
            Self::admin(),
            Self::delete(),
            Self::invite(),
            Self::manage_roles(),
            Self::view_audit(),
        ]
    }
}

// ============ Inheritance Configuration ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceConfig {
    pub enabled: bool,
    pub max_depth: i32,
    pub cascading_updates: bool,
    pub override_allowed: bool,
}

impl Default for InheritanceConfig {
    fn default() -> Self {
        InheritanceConfig {
            enabled: true,
            max_depth: 5,
            cascading_updates: true,
            override_allowed: true,
        }
    }
}
