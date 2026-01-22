-- Organizations table
CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name)
);

-- Team members table
CREATE TABLE IF NOT EXISTS team_members (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL CHECK (role IN ('OWNER', 'ADMIN', 'EDITOR', 'REVIEWER', 'VIEWER')),
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, organization_id)
);

-- Project permissions table
CREATE TABLE IF NOT EXISTS project_permissions (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL CHECK (role IN ('OWNER', 'ADMIN', 'EDITOR', 'REVIEWER', 'VIEWER')),
    granted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(project_id, user_id)
);

-- Pull requests table
CREATE TABLE IF NOT EXISTS pull_requests (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    source_branch VARCHAR(255) NOT NULL,
    target_branch VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'PENDING', 'APPROVED', 'REQUESTED_CHANGES', 'CLOSED')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Code reviews table
CREATE TABLE IF NOT EXISTS code_reviews (
    id UUID PRIMARY KEY,
    pull_request_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    reviewer_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL CHECK (status IN ('DRAFT', 'PENDING', 'APPROVED', 'REQUESTED_CHANGES', 'CLOSED')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(pull_request_id, reviewer_id)
);

-- Review comments table
CREATE TABLE IF NOT EXISTS review_comments (
    id UUID PRIMARY KEY,
    pull_request_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    file_path VARCHAR(255) NOT NULL,
    line_number INTEGER NOT NULL,
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Collaboration sessions table
CREATE TABLE IF NOT EXISTS collaboration_sessions (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    left_at TIMESTAMP
);

-- Cursor positions table
CREATE TABLE IF NOT EXISTS cursor_positions (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_id UUID NOT NULL,
    line INTEGER NOT NULL,
    column INTEGER NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(project_id, user_id)
);

-- Conflict resolutions table
CREATE TABLE IF NOT EXISTS conflict_resolutions (
    id UUID PRIMARY KEY,
    file_id UUID NOT NULL,
    conflict_type VARCHAR(100) NOT NULL,
    user_version TEXT NOT NULL,
    server_version TEXT NOT NULL,
    resolved_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_team_members_org ON team_members(organization_id);
CREATE INDEX IF NOT EXISTS idx_team_members_user ON team_members(user_id);
CREATE INDEX IF NOT EXISTS idx_project_permissions_project ON project_permissions(project_id);
CREATE INDEX IF NOT EXISTS idx_project_permissions_user ON project_permissions(user_id);
CREATE INDEX IF NOT EXISTS idx_pull_requests_project ON pull_requests(project_id);
CREATE INDEX IF NOT EXISTS idx_pull_requests_author ON pull_requests(author_id);
CREATE INDEX IF NOT EXISTS idx_code_reviews_pr ON code_reviews(pull_request_id);
CREATE INDEX IF NOT EXISTS idx_review_comments_pr ON review_comments(pull_request_id);
CREATE INDEX IF NOT EXISTS idx_collaboration_sessions_project ON collaboration_sessions(project_id);
CREATE INDEX IF NOT EXISTS idx_cursor_positions_project ON cursor_positions(project_id);
