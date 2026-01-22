use axum::{extract::State, Json, Path};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::Database,
    error::{AppError, AppResult},
    models::{CreateProjectRequest, Project, UpdateProjectRequest},
};

pub async fn create_project(
    State(db): State<Arc<Database>>,
    Json(payload): Json<CreateProjectRequest>,
) -> AppResult<Json<Project>> {
    let project_id = Uuid::new_v4();
    let user_id = Uuid::new_v4(); // Should extract from JWT token in production

    sqlx::query(
        "INSERT INTO projects (id, user_id, name, description, language, repository_url) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(&project_id)
    .bind(&user_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.language)
    .bind(&payload.repository_url)
    .execute(db.pool())
    .await?;

    Ok(Json(Project {
        id: project_id,
        user_id,
        name: payload.name,
        description: payload.description,
        language: payload.language,
        repository_url: payload.repository_url,
        created_at: chrono::Utc::now(),
    }))
}

pub async fn list_projects(
    State(db): State<Arc<Database>>,
) -> AppResult<Json<Vec<Project>>> {
    let rows = sqlx::query("SELECT id, user_id, name, description, language, repository_url, created_at FROM projects LIMIT 50")
        .fetch_all(db.pool())
        .await?;

    let projects = rows
        .iter()
        .map(|row| Project {
            id: row.get("id"),
            user_id: row.get("user_id"),
            name: row.get("name"),
            description: row.get("description"),
            language: row.get("language"),
            repository_url: row.get("repository_url"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(Json(projects))
}

pub async fn get_project(
    State(db): State<Arc<Database>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Project>> {
    let row = sqlx::query("SELECT id, user_id, name, description, language, repository_url, created_at FROM projects WHERE id = $1")
        .bind(&id)
        .fetch_optional(db.pool())
        .await?;

    let row = row.ok_or(AppError::NotFoundError("Project not found".to_string()))?;

    Ok(Json(Project {
        id: row.get("id"),
        user_id: row.get("user_id"),
        name: row.get("name"),
        description: row.get("description"),
        language: row.get("language"),
        repository_url: row.get("repository_url"),
        created_at: row.get("created_at"),
    }))
}

pub async fn update_project(
    State(db): State<Arc<Database>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProjectRequest>,
) -> AppResult<Json<Project>> {
    // Get existing project
    let row = sqlx::query("SELECT id, user_id, name, description, language, repository_url, created_at FROM projects WHERE id = $1")
        .bind(&id)
        .fetch_optional(db.pool())
        .await?;

    let row = row.ok_or(AppError::NotFoundError("Project not found".to_string()))?;

    let name = payload.name.unwrap_or_else(|| row.get("name"));
    let description = payload.description.or_else(|| row.get("description"));
    let language = payload.language.or_else(|| row.get("language"));

    sqlx::query("UPDATE projects SET name = $1, description = $2, language = $3, updated_at = CURRENT_TIMESTAMP WHERE id = $4")
        .bind(&name)
        .bind(&description)
        .bind(&language)
        .bind(&id)
        .execute(db.pool())
        .await?;

    Ok(Json(Project {
        id,
        user_id: row.get("user_id"),
        name,
        description,
        language,
        repository_url: row.get("repository_url"),
        created_at: row.get("created_at"),
    }))
}

pub async fn delete_project(
    State(db): State<Arc<Database>>,
    Path(id): Path<Uuid>,
) -> AppResult<&'static str> {
    sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(&id)
        .execute(db.pool())
        .await?;

    Ok("Project deleted successfully")
}

pub async fn list_files(
    State(db): State<Arc<Database>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<crate::models::CodeFile>>> {
    let rows = sqlx::query("SELECT id, project_id, file_path, content, language FROM code_files WHERE project_id = $1")
        .bind(&id)
        .fetch_all(db.pool())
        .await?;

    let files = rows
        .iter()
        .map(|row| crate::models::CodeFile {
            id: row.get("id"),
            project_id: row.get("project_id"),
            file_path: row.get("file_path"),
            content: row.get("content"),
            language: row.get("language"),
        })
        .collect();

    Ok(Json(files))
}
