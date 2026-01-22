use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Database { pool })
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                email VARCHAR(255) UNIQUE NOT NULL,
                password_hash VARCHAR(255) NOT NULL,
                first_name VARCHAR(100),
                last_name VARCHAR(100),
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS projects (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                language VARCHAR(50),
                repository_url VARCHAR(255),
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS code_files (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                file_path VARCHAR(500) NOT NULL,
                content TEXT NOT NULL,
                language VARCHAR(50),
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS analysis_tasks (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                task_type VARCHAR(50) NOT NULL,
                status VARCHAR(50) DEFAULT 'pending',
                input_data JSONB,
                output_data JSONB,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                completed_at TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS agent_tasks (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                agent_type VARCHAR(50) NOT NULL,
                status VARCHAR(50) DEFAULT 'pending',
                request_data JSONB,
                result_data JSONB,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                completed_at TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS analytics_metrics (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                metric_type VARCHAR(100) NOT NULL,
                value NUMERIC,
                metadata JSONB,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS teams (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                slug VARCHAR(255) UNIQUE NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS team_members (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                role VARCHAR(50) NOT NULL DEFAULT 'member',
                joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(team_id, user_id)
            );
            
            CREATE TABLE IF NOT EXISTS project_members (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                role VARCHAR(50) NOT NULL DEFAULT 'viewer',
                permissions JSONB DEFAULT '[]',
                joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(project_id, user_id)
            );
            
            CREATE TABLE IF NOT EXISTS code_reviews (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                title VARCHAR(255) NOT NULL,
                description TEXT,
                status VARCHAR(50) DEFAULT 'open',
                source_branch VARCHAR(255),
                target_branch VARCHAR(255),
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                closed_at TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS review_comments (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                review_id UUID NOT NULL REFERENCES code_reviews(id) ON DELETE CASCADE,
                author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                file_path VARCHAR(500),
                line_number INT,
                content TEXT NOT NULL,
                resolved BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS review_approvals (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                review_id UUID NOT NULL REFERENCES code_reviews(id) ON DELETE CASCADE,
                reviewer_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                status VARCHAR(50) NOT NULL,
                comments TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(review_id, reviewer_id)
            );
            
            CREATE TABLE IF NOT EXISTS collaborative_sessions (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                file_id UUID NOT NULL REFERENCES code_files(id) ON DELETE CASCADE,
                session_token VARCHAR(255) UNIQUE NOT NULL,
                status VARCHAR(50) DEFAULT 'active',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                expires_at TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS session_participants (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                session_id UUID NOT NULL REFERENCES collaborative_sessions(id) ON DELETE CASCADE,
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                cursor_position INT,
                selection_start INT,
                selection_end INT,
                joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                left_at TIMESTAMP,
                UNIQUE(session_id, user_id)
            );
            
            CREATE TABLE IF NOT EXISTS document_versions (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                file_id UUID NOT NULL REFERENCES code_files(id) ON DELETE CASCADE,
                version_number INT NOT NULL,
                content TEXT NOT NULL,
                author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                change_description TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(file_id, version_number)
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
}
