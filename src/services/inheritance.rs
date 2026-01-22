use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;
use std::collections::HashMap;
use crate::models::inheritance::{
    ResolvedPermissions, InheritedPermissionInfo, HierarchyTree, InheritanceConfig,
};
use std::sync::Arc;

pub struct InheritanceEngine {
    pool: Arc<Pool<Postgres>>,
    config: InheritanceConfig,
    cache: std::sync::Mutex<HashMap<(Uuid, Uuid), ResolvedPermissions>>,
}

impl InheritanceEngine {
    pub fn new(pool: Arc<Pool<Postgres>>, config: Option<InheritanceConfig>) -> Self {
        Self {
            pool,
            config: config.unwrap_or_default(),
            cache: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Resolve effective permissions for a user on a resource
    pub async fn resolve_permissions(
        &self,
        user_id: Uuid,
        resource_id: Uuid,
        resource_type: &str,
    ) -> Result<ResolvedPermissions, String> {
        // Check cache first
        let cache_key = (user_id, resource_id);
        if let Ok(cache) = self.cache.lock() {
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        // Get direct permissions
        let direct_perms = self
            .get_direct_permissions(user_id, resource_id, resource_type)
            .await?;

        // Get inherited permissions
        let inherited_perms = self
            .get_inherited_permissions(user_id, resource_id, resource_type)
            .await?;

        // Merge and resolve effective permissions
        let effective_perms = Self::merge_permissions(&direct_perms, &inherited_perms);
        let role = self
            .get_user_role(user_id, resource_id, resource_type)
            .await?;

        let resolved = ResolvedPermissions {
            user_id,
            resource_id,
            resource_type: resource_type.to_string(),
            direct_permissions: direct_perms,
            inherited_permissions: inherited_perms,
            effective_permissions: effective_perms,
            role,
        };

        // Cache result
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(cache_key, resolved.clone());
        }

        Ok(resolved)
    }

    /// Get direct permissions assigned to user on resource
    async fn get_direct_permissions(
        &self,
        user_id: Uuid,
        resource_id: Uuid,
        resource_type: &str,
    ) -> Result<Vec<String>, String> {
        let table = if resource_type == "team" {
            "team_members"
        } else if resource_type == "project" {
            "project_members"
        } else {
            return Err("Invalid resource type".to_string());
        };

        let id_col = if resource_type == "team" {
            "team_id"
        } else {
            "project_id"
        };

        let query = format!(
            r#"
            SELECT permissions FROM {} 
            WHERE {} = $1 AND user_id = $2
            "#,
            table, id_col
        );

        let result = sqlx::query_scalar::<_, Option<serde_json::Value>>(&query)
            .bind(resource_id)
            .bind(user_id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| e.to_string())?;

        match result {
            Some(Some(perms)) => {
                let perms: Vec<String> = serde_json::from_value(perms).unwrap_or_default();
                Ok(perms)
            }
            _ => Ok(vec![]),
        }
    }

    /// Get inherited permissions from parent resources
    async fn get_inherited_permissions(
        &self,
        user_id: Uuid,
        resource_id: Uuid,
        resource_type: &str,
    ) -> Result<Vec<InheritedPermissionInfo>, String> {
        if !self.config.enabled {
            return Ok(vec![]);
        }

        let mut inherited = Vec::new();
        let mut to_process = vec![(resource_id, 0)];
        let mut processed = std::collections::HashSet::new();

        while let Some((current_id, depth)) = to_process.pop() {
            if depth > self.config.max_depth || processed.contains(&current_id) {
                continue;
            }
            processed.insert(current_id);

            // Get parents
            let parents = self.get_parents(current_id, resource_type).await?;

            for parent_id in parents {
                // Get parent permissions for user
                let parent_perms = self
                    .get_direct_permissions(user_id, parent_id, resource_type)
                    .await?;

                if !parent_perms.is_empty() {
                    let role = self
                        .get_user_role(user_id, parent_id, resource_type)
                        .await?;

                    inherited.push(InheritedPermissionInfo {
                        source_id: parent_id,
                        source_type: resource_type.to_string(),
                        permissions: parent_perms,
                        depth: depth + 1,
                        from_role: role,
                    });

                    // Continue traversal
                    if depth < self.config.max_depth {
                        to_process.push((parent_id, depth + 1));
                    }
                }
            }
        }

        Ok(inherited)
    }

    /// Get parent resources
    async fn get_parents(&self, resource_id: Uuid, resource_type: &str) -> Result<Vec<Uuid>, String> {
        let table = if resource_type == "team" {
            "team_hierarchy"
        } else if resource_type == "project" {
            "project_hierarchy"
        } else {
            return Err("Invalid resource type".to_string());
        };

        let child_col = if resource_type == "team" {
            "child_team_id"
        } else {
            "child_project_id"
        };

        let parent_col = if resource_type == "team" {
            "parent_team_id"
        } else {
            "parent_project_id"
        };

        let query = format!(
            r#"
            SELECT {} FROM {} 
            WHERE {} = $1 AND inheritance_enabled = TRUE
            "#,
            parent_col, table, child_col
        );

        let parents = sqlx::query_scalar::<_, Option<Uuid>>(&query)
            .bind(resource_id)
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter_map(|p| p)
            .collect();

        Ok(parents)
    }

    /// Get user's role on resource
    async fn get_user_role(
        &self,
        user_id: Uuid,
        resource_id: Uuid,
        resource_type: &str,
    ) -> Result<String, String> {
        let table = if resource_type == "team" {
            "team_members"
        } else if resource_type == "project" {
            "project_members"
        } else {
            return Err("Invalid resource type".to_string());
        };

        let id_col = if resource_type == "team" {
            "team_id"
        } else {
            "project_id"
        };

        let query = format!(
            r#"
            SELECT role FROM {} 
            WHERE {} = $1 AND user_id = $2
            "#,
            table, id_col
        );

        let role = sqlx::query_scalar::<_, Option<String>>(&query)
            .bind(resource_id)
            .bind(user_id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| e.to_string())?
            .flatten()
            .unwrap_or_else(|| "viewer".to_string());

        Ok(role)
    }

    /// Merge direct and inherited permissions
    fn merge_permissions(
        direct: &[String],
        inherited: &[InheritedPermissionInfo],
    ) -> Vec<String> {
        let mut merged = direct.to_vec();

        for inherited_info in inherited {
            for perm in &inherited_info.permissions {
                if !merged.contains(perm) {
                    merged.push(perm.clone());
                }
            }
        }

        merged.sort();
        merged.dedup();
        merged
    }

    /// Build hierarchy tree for visualization
    pub async fn build_hierarchy_tree(
        &self,
        resource_id: Uuid,
        resource_type: &str,
        name: &str,
    ) -> Result<HierarchyTree, String> {
        let children = self
            .get_children(resource_id, resource_type)
            .await?;

        let mut tree_children = Vec::new();
        for child_id in children {
            let child_tree = self
                .build_hierarchy_tree(child_id, resource_type, "child")
                .await?;
            tree_children.push(child_tree);
        }

        Ok(HierarchyTree {
            id: resource_id,
            name: name.to_string(),
            resource_type: resource_type.to_string(),
            children: tree_children,
            permissions_inherited: !tree_children.is_empty(),
        })
    }

    /// Get child resources
    async fn get_children(&self, resource_id: Uuid, resource_type: &str) -> Result<Vec<Uuid>, String> {
        let table = if resource_type == "team" {
            "team_hierarchy"
        } else if resource_type == "project" {
            "project_hierarchy"
        } else {
            return Err("Invalid resource type".to_string());
        };

        let parent_col = if resource_type == "team" {
            "parent_team_id"
        } else {
            "parent_project_id"
        };

        let child_col = if resource_type == "team" {
            "child_team_id"
        } else {
            "child_project_id"
        };

        let query = format!(
            r#"
            SELECT {} FROM {} 
            WHERE {} = $1 AND inheritance_enabled = TRUE
            "#,
            child_col, table, parent_col
        );

        let children = sqlx::query_scalar::<_, Uuid>(&query)
            .bind(resource_id)
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(children)
    }

    /// Clear permission cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    /// Clear cache for specific resource
    pub fn clear_cache_for_resource(&self, user_id: Uuid, resource_id: Uuid) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.remove(&(user_id, resource_id));
        }
    }

    /// Check if user has permission
    pub async fn has_permission(
        &self,
        user_id: Uuid,
        resource_id: Uuid,
        resource_type: &str,
        permission: &str,
    ) -> Result<bool, String> {
        let resolved = self
            .resolve_permissions(user_id, resource_id, resource_type)
            .await?;

        Ok(resolved.effective_permissions.contains(&permission.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_permissions() {
        let direct = vec!["read".to_string(), "write".to_string()];
        let inherited = vec![InheritedPermissionInfo {
            source_id: Uuid::new_v4(),
            source_type: "team".to_string(),
            permissions: vec!["admin".to_string(), "read".to_string()],
            depth: 1,
            from_role: "admin".to_string(),
        }];

        let merged = InheritanceEngine::merge_permissions(&direct, &inherited);
        assert_eq!(merged.len(), 3);
        assert!(merged.contains(&"read".to_string()));
        assert!(merged.contains(&"write".to_string()));
        assert!(merged.contains(&"admin".to_string()));
    }
}
