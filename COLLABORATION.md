# CompileX7 Collaboration Features

This document describes the collaboration and team management features integrated into CompileX7 backend.

## Overview

CompileX7 now includes enterprise-grade collaboration capabilities:
- Real-time code editing with WebSocket support
- Role-Based Access Control (RBAC) for teams and projects
- Code review workflows with approval systems
- Conflict resolution using Operational Transformation (OT)
- Team and project member management

## Architecture

### Database Schema

#### Teams & Members
```sql
teams - Team organizations with owner and metadata
team_members - Team membership with role-based access
```

#### Projects & Access
```sql
project_members - Project-level access control
collaborative_sessions - Active editing sessions
session_participants - Users in collaboration sessions
document_versions - Version history for collaborative edits
```

#### Code Reviews
```sql
code_reviews - Review metadata and status
review_comments - Line-specific and general comments
review_approvals - Reviewer approvals/rejections
```

### Core Modules

#### 1. RBAC Middleware (`src/middleware/rbac.rs`)
Enforces role-based access control for all operations.

**Role Hierarchy:**
- `owner` (4): Full control, can manage team
- `admin` (3): Can modify project, manage members
- `member` (2): Can read/write code, comment on reviews
- `viewer` (1): Read-only access

**Key Functions:**
```rust
// Check if user has specific permission on project
check_project_permission(pool, user_id, project_id, "write").await?

// Check if user meets minimum team role
check_team_role(pool, user_id, team_id, min_role_level).await?

// Verify user is project admin
check_project_admin(pool, user_id, project_id).await?

// Enforce permission check (returns 403 if unauthorized)
enforce_permission(pool, user_id, project_id, "admin").await?
```

#### 2. Real-Time Collaboration (`src/services/collaboration.rs`)
Manages live collaboration sessions and operations.

**Features:**
- Session creation and participant management
- Cursor position tracking
- Operation broadcasting via channels
- Conflict detection
- Operational Transformation

**Key API:**
```rust
let manager = CollaborationManager::new();

// Create and manage sessions
manager.create_session(session_id, file_id)?
manager.join_session(session_id, user_id)?
manager.leave_session(session_id, user_id)?

// Track cursors
let cursor_update = CursorUpdate { user_id, session_id, cursor_position: 42, ... };
manager.update_cursor(session_id, cursor_update)?

// Apply and transform operations
let version = manager.apply_operation(session_id, operation)?
let conflicts = manager.detect_conflicts(session_id, incoming_op.version)?
let participants = manager.get_participants(session_id)?
```

#### 3. Operational Transformation Engine (`src/services/ot_engine.rs`)
Implements conflict-free collaborative editing.

**Algorithm:**
- Transforms client operations against concurrent server operations
- Supports Insert, Delete, and Replace operations
- Maintains document consistency without explicit locking
- Validates operations before application

**Usage:**
```rust
use crate::services::OTEngine;

// Transform operation against concurrent ops
let transformed = OTEngine::transform(&client_op, &server_ops);

// Detect conflicts
if let Some(conflict) = OTEngine::detect_conflicts(client_version, server_ops) {
    // Handle conflict
}

// Resolve conflicting operations
let resolution = OTEngine::resolve_conflicts(&original_content, &conflicting_ops);

// Validate operation
OTEngine::validate_operation(&op, content_length)?
```

#### 4. Code Review System (`src/handlers/code_review.rs`)
Implements comprehensive code review workflows.

**Endpoints:**
```
POST   /projects/:id/reviews              - Create code review
GET    /projects/:id/reviews/:review_id   - Get review with comments
PUT    /projects/:id/reviews/:review_id   - Update review status

POST   /projects/:id/reviews/:id/comments - Add comment
PUT    /projects/:id/reviews/:id/comments/:cid - Update comment

POST   /projects/:id/reviews/:id/approve  - Submit approval
GET    /projects/:id/reviews/:id/approvals - Get all approvals
```

**Approval States:**
- `open` - Under review, waiting for feedback
- `approved` - Reviewer approved the changes
- `changes_requested` - Reviewer requested modifications
- `merged` - Changes merged into main branch
- `closed` - Review closed without merging

#### 5. Team Management (`src/handlers/teams.rs`)
Manages teams, team members, and project access.

**Team Operations:**
```rust
// Create team
POST /teams
{
    "name": "Engineering Team",
    "description": "Backend development team"
}

// Add team member
POST /teams/:id/members
{
    "user_id": "uuid",
    "role": "member|viewer|admin|owner"
}

// Update team member role
PUT /teams/:id/members/:member_id
{
    "role": "admin"
}

// Remove team member
DELETE /teams/:id/members/:member_id
```

**Project Member Operations:**
```rust
// Add project member
POST /projects/:id/members
{
    "user_id": "uuid",
    "role": "viewer|editor|admin",
    "permissions": ["read", "write", "admin"]
}

// Update project member permissions
PUT /projects/:id/members/:member_id
{
    "permissions": ["read", "write"]
}

// Check user permissions
GET /projects/:id/members/:user_id/permissions
```

## Permissions System

### Permission Levels

**Project Permissions:**
- `read` - View code and reviews
- `write` - Edit code, comment on reviews
- `admin` - Manage project members, create reviews
- `delete` - Delete projects/files

**Team Roles:**
- Owner can manage entire team
- Admin can add/remove members and manage projects
- Member can participate in code review
- Viewer has read-only access

### Permission Checking

```rust
// Check single permission
rbac::check_project_permission(&pool, user_id, project_id, "write").await?

// Enforce permission (fails with 403)
rbac::enforce_permission(&pool, user_id, project_id, "admin").await?

// Check minimum role level (1-4)
rbac::check_team_role(&pool, user_id, team_id, 2).await?

// Verify project admin
rbac::check_project_admin(&pool, user_id, project_id).await?
```

## Collaboration Flow

### Real-Time Editing
1. User creates collaborative session
2. Multiple users join session
3. Users make edits (Insert/Delete/Replace operations)
4. Operations broadcast to all participants
5. OT engine transforms concurrent operations
6. Conflict detection alerts users
7. Conflict resolution merges changes

### Code Review Flow
1. Developer creates code review for branch
2. Adds title, description, and source/target branches
3. Reviewers add line-specific comments
4. Reviewers submit approval/change requests
5. Developer responds to comments
6. Once approved, code merged to target branch

## Data Models

### Team
```rust
pub struct Team {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### CollaborativeSession
```rust
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
```

### CodeReview
```rust
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
```

### DocumentOperation
```rust
pub struct DocumentOperation {
    pub id: String,
    pub version: u32,
    pub timestamp: DateTime<Utc>,
    pub user_id: Uuid,
    pub operation: OperationType,
}

pub enum OperationType {
    Insert { position: usize, content: String },
    Delete { position: usize, length: usize },
    Replace { position: usize, old_content: String, new_content: String },
}
```

## Operational Transformation Details

### Transform Rules

**Insert vs Insert:**
- If position1 < position2: position1 unchanged
- If position1 > position2: position1 += length of insert2
- If position1 == position2: tie-break by operation ID

**Insert vs Delete:**
- If delete overlaps with insert: insert moves to delete start
- If delete before insert: insert position -= delete length
- If delete after insert: insert position unchanged

**Delete vs Delete:**
- Overlapping deletes are resolved by operation version
- Earlier operations take precedence
- Delete lengths adjusted based on overlap

### Conflict Detection

Operations are considered conflicting if:
- Multiple users edit overlapping text ranges
- Delete operation removes text edited by another user
- Operations reach server out of causal order

### Conflict Resolution

When conflicts detected:
1. Collect all concurrent operations
2. Sort by version and timestamp
3. Apply OT transformation
4. Merge operations sequentially
5. Generate resolved document

## Integration Examples

### Creating a Team
```rust
let team_req = CreateTeamRequest {
    name: "Platform Team".to_string(),
    description: Some("Building the platform".to_string()),
};

let (status, team) = create_team(State(pool), user_id, Json(team_req)).await?;
// Returns 201 with Team data
```

### Starting Collaboration
```rust
let manager = CollaborationManager::new();

// Create session
manager.create_session(session_id, file_id)?;

// Users join
manager.join_session(session_id, user1_id)?;
manager.join_session(session_id, user2_id)?;

// User applies operation
let op = DocumentOperation { /* ... */ };
let version = manager.apply_operation(session_id, op)?;

// Get all participants
let participants = manager.get_participants(session_id)?;
```

### Submitting Code Review
```rust
let review_req = CreateCodeReviewRequest {
    title: "Add authentication".to_string(),
    description: Some("Implements JWT auth".to_string()),
    source_branch: Some("feature/auth".to_string()),
    target_branch: Some("main".to_string()),
};

let (status, review) = create_code_review(
    State(pool),
    Path(project_id),
    user_id,
    Json(review_req),
).await?;
```

### Adding Review Comment
```rust
let comment_req = AddReviewCommentRequest {
    file_path: Some("src/auth.rs".to_string()),
    line_number: Some(42),
    content: "Consider using constant-time comparison".to_string(),
};

let (status, comment) = add_review_comment(
    State(pool),
    Path((project_id, review_id)),
    user_id,
    Json(comment_req),
).await?;
```

## Security Considerations

1. **RBAC Enforcement:** All endpoints verify permissions before proceeding
2. **Operation Validation:** All document operations validated for bounds/feasibility
3. **Session Tokens:** Collaborative sessions use secure tokens
4. **Access Control:** Project members explicitly added, cascading deletes on removal
5. **Audit Trail:** All changes timestamped with user attribution
6. **Conflict Detection:** Prevents silent data loss from concurrent edits

## Testing

Included unit tests for:
- OT transformation rules
- Operation validation
- Session management
- Slug generation
- Role hierarchy

Run tests:
```bash
cargo test
```

## Future Enhancements

- WebSocket handlers for real-time communication
- Presence awareness (showing active users)
- Version branching for non-linear history
- Undo/redo with operation history
- Advanced diff algorithms (patience diff, histogram diff)
- Notification system for reviews
- Integration with Git providers
