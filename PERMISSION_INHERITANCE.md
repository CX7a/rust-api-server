# Permission Inheritance System

## Overview

The Permission Inheritance System enables hierarchical permission management for teams and projects in CompileX7. This allows organizations to create nested team/project structures where child resources automatically inherit permissions from their parents, reducing permission management overhead.

## Table of Contents

1. [Core Concepts](#core-concepts)
2. [Architecture](#architecture)
3. [API Reference](#api-reference)
4. [Configuration](#configuration)
5. [Examples](#examples)
6. [Best Practices](#best-practices)
7. [Troubleshooting](#troubleshooting)

## Core Concepts

### Hierarchy Types

#### Team Hierarchy
- Teams can be organized in parent-child relationships
- Child teams inherit permissions from parent teams
- Useful for organizational structure (e.g., Company → Department → Team)

#### Project Hierarchy
- Projects can be organized in parent-child relationships
- Child projects inherit permissions from parent projects
- Useful for project organization (e.g., Product → Feature → Sprint)

### Permission Resolution

Permission resolution follows a specific precedence:

1. **Direct Permissions** - Permissions explicitly assigned to user on resource
2. **Inherited Permissions** - Permissions inherited from parent resources
3. **Effective Permissions** - Union of direct and inherited permissions

### Inheritance Depth

By default, permissions traverse up to 5 levels in the hierarchy. This prevents infinite loops and maintains performance.

```
Level 0: Resource (child)
Level 1: Parent
Level 2: Grandparent
Level 3: Great-grandparent
Level 4: Great-great-grandparent
Level 5: (max depth reached)
```

### Permission Caching

Resolved permissions are cached to optimize performance. The cache is automatically invalidated when:
- A user's role changes
- A hierarchy relationship is modified
- A permission rule is updated

## Architecture

### Components

#### InheritanceEngine
Core service responsible for permission resolution and hierarchy management.

```rust
pub struct InheritanceEngine {
    pool: Arc<Pool<Postgres>>,
    config: InheritanceConfig,
    cache: std::sync::Mutex<HashMap<(Uuid, Uuid), ResolvedPermissions>>,
}
```

**Key Methods:**
- `resolve_permissions()` - Get effective permissions for user on resource
- `get_inherited_permissions()` - Get permissions from parent hierarchy
- `build_hierarchy_tree()` - Visualize hierarchy structure
- `has_permission()` - Check if user has specific permission
- `clear_cache()` - Invalidate permission cache

#### RBAC Middleware Extensions
New middleware functions support inheritance-aware permission checks.

**New Functions:**
- `enforce_permission_with_inheritance()` - Check permissions with inheritance
- `get_resolved_permissions()` - Get detailed permission breakdown

#### Handlers
New endpoints for managing hierarchies and permissions.

**Endpoints:**
- `POST /api/hierarchies/teams` - Create team hierarchy
- `POST /api/hierarchies/projects` - Create project hierarchy
- `GET /api/permissions/{resource_id}/{resource_type}` - Get resolved permissions
- `POST /api/permission-rules` - Create permission rules
- `GET /api/audit-logs` - View audit trail

### Database Schema

#### New Tables

```sql
-- Team/Project hierarchies
team_hierarchy
project_hierarchy

-- Permission rules by role
permission_rules

-- Inherited permissions cache
inherited_permissions

-- Audit trail
audit_logs
```

## API Reference

### Create Team Hierarchy

```bash
POST /api/hierarchies/teams
Content-Type: application/json
Authorization: Bearer <token>

{
  "parent_team_id": "550e8400-e29b-41d4-a716-446655440000",
  "child_team_id": "550e8400-e29b-41d4-a716-446655440001",
  "inheritance_enabled": true
}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440002",
  "parent_team_id": "550e8400-e29b-41d4-a716-446655440000",
  "child_team_id": "550e8400-e29b-41d4-a716-446655440001",
  "inheritance_enabled": true,
  "created_at": "2024-01-22T10:00:00Z"
}
```

### Get Resolved Permissions

```bash
GET /api/permissions/{resource_id}/team
Authorization: Bearer <token>
```

**Response:**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440003",
  "resource_id": "550e8400-e29b-41d4-a716-446655440000",
  "resource_type": "team",
  "direct_permissions": ["read", "write"],
  "inherited_permissions": [
    {
      "source_id": "550e8400-e29b-41d4-a716-446655440001",
      "source_type": "team",
      "permissions": ["admin"],
      "depth": 1,
      "from_role": "admin"
    }
  ],
  "effective_permissions": ["read", "write", "admin"],
  "role": "member"
}
```

### Create Permission Rule

```bash
POST /api/permission-rules
Content-Type: application/json
Authorization: Bearer <token>

{
  "team_id": "550e8400-e29b-41d4-a716-446655440000",
  "role": "member",
  "permissions": ["read", "write"],
  "description": "Default member permissions",
  "priority": 0
}
```

### Get Audit Logs

```bash
GET /api/audit-logs?resource_type=team&resource_id=550e8400-e29b-41d4-a716-446655440000
Authorization: Bearer <token>
```

## Configuration

### InheritanceConfig

```rust
pub struct InheritanceConfig {
    pub enabled: bool,              // Enable/disable inheritance
    pub max_depth: i32,             // Maximum hierarchy depth (default: 5)
    pub cascading_updates: bool,    // Propagate changes downward
    pub override_allowed: bool,     // Allow child overrides
}
```

### Default Configuration

```rust
InheritanceConfig {
    enabled: true,
    max_depth: 5,
    cascading_updates: true,
    override_allowed: true,
}
```

## Examples

### Example 1: Corporate Structure

```
Company (Parent Team)
├── Engineering Department
│   ├── Backend Team
│   └── Frontend Team
└── Sales Department
    └── Account Management Team
```

**Permission Flow:**
1. User assigned "read" on Company team
2. User automatically gets "read" on all child teams
3. Backend Team can override with "admin" for team members

### Example 2: Multi-level Project

```
Product (Parent Project)
├── Feature A
│   ├── Sprint 1
│   └── Sprint 2
└── Feature B
    ├── Sprint 1
    └── Sprint 2
```

**Permission Flow:**
1. Developer assigned "write" on Feature A
2. Developer inherits "write" on Sprint 1 and Sprint 2
3. Sprint leads get "admin" on their specific sprints

### Usage Example

```rust
// Create inheritance engine
let engine = InheritanceEngine::new(
    Arc::new(pool),
    Some(InheritanceConfig::default())
);

// Resolve permissions for user
let permissions = engine.resolve_permissions(
    user_id,
    team_id,
    "team"
).await?;

// Check specific permission
let can_write = engine.has_permission(
    user_id,
    team_id,
    "team",
    "write"
).await?;

// Build hierarchy tree
let tree = engine.build_hierarchy_tree(
    team_id,
    "team",
    "Engineering"
).await?;
```

## Best Practices

### 1. Hierarchy Design

- Keep hierarchy depth under 5 levels for performance
- Design hierarchies that match organizational structure
- Avoid circular references (enforced by database constraints)

### 2. Permission Rules

- Define role-based permission rules at each level
- Use consistent role names across hierarchy
- Document permission inheritance flow

### 3. Performance

- Use permission caching in high-volume scenarios
- Invalidate cache strategically to avoid stale permissions
- Monitor query performance on large hierarchies

### 4. Audit Trail

- Enable audit logging for compliance requirements
- Review audit logs regularly for permission changes
- Archive old logs for archival

### 5. Security

- Always verify user permissions via inheritance engine
- Don't trust cached permissions in sensitive operations
- Use role hierarchy to enforce least privilege

## Troubleshooting

### Issue: Permissions Not Inherited

**Symptoms:** User doesn't have expected inherited permissions

**Solutions:**
1. Verify hierarchy relationship exists: Check `team_hierarchy` or `project_hierarchy` tables
2. Check inheritance enabled: Ensure `inheritance_enabled = true`
3. Clear cache: Call `engine.clear_cache()` to refresh
4. Verify role assignment: Ensure user has role on parent resource

### Issue: Circular Hierarchy

**Symptoms:** Database constraint error on hierarchy creation

**Solutions:**
1. Review existing hierarchies to identify loop
2. Remove circular relationship
3. Use `build_hierarchy_tree()` to visualize structure

### Issue: Performance Degradation

**Symptoms:** Slow permission resolution

**Solutions:**
1. Check hierarchy depth: Limit to reasonable depth
2. Monitor cache hit rate: Review cache statistics
3. Analyze database queries: Use EXPLAIN ANALYZE
4. Consider flattening very deep hierarchies

### Issue: Stale Permissions After Update

**Symptoms:** Permission changes not reflected immediately

**Solutions:**
1. Clear relevant cache entries: `engine.clear_cache_for_resource(user_id, resource_id)`
2. Wait for cache expiration (if TTL configured)
3. Restart service (full cache clear)

## Migration Guide

### From Flat Permissions to Hierarchical

```rust
// Step 1: Create hierarchy relationship
POST /api/hierarchies/teams {
  "parent_team_id": "...",
  "child_team_id": "...",
  "inheritance_enabled": true
}

// Step 2: Create permission rules
POST /api/permission-rules {
  "team_id": "...",
  "role": "member",
  "permissions": ["read", "write"],
  ...
}

// Step 3: Verify permissions resolve correctly
GET /api/permissions/{resource_id}/team

// Step 4: Remove redundant direct permissions if appropriate
DELETE old direct permissions

// Step 5: Monitor via audit logs
GET /api/audit-logs?action=*
```

## Performance Metrics

### Typical Resolution Times

- **Simple hierarchy** (1-2 levels): < 5ms
- **Medium hierarchy** (3-4 levels): 10-50ms
- **Deep hierarchy** (5+ levels): 50-200ms

### Cache Impact

- **Cache hit**: < 1ms
- **Cache miss**: 10-200ms (depending on hierarchy depth)
- **Cache hit rate goal**: > 90% for production

## Support

For issues or questions:
1. Check this documentation
2. Review audit logs for diagnostics
3. Check database schema constraints
4. Test with `InheritanceEngine` unit tests
