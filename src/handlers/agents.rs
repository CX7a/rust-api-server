use axum::{extract::State, Json, Path};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::Database,
    error::AppResult,
    models::{AgentRequest, AgentTaskResponse, AgentTaskStatus},
    services::agent::{Agent, FrontendAgent, BackendAgent, QAAgent},
};

pub async fn frontend_agent(
    State(db): State<Arc<Database>>,
    Json(payload): Json<AgentRequest>,
) -> AppResult<Json<AgentTaskResponse>> {
    let task_id = Uuid::new_v4();

    // Store agent task in database
    sqlx::query(
        "INSERT INTO agent_tasks (id, project_id, agent_type, status, request_data) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&task_id)
    .bind(&payload.project_id)
    .bind("frontend")
    .bind("processing")
    .bind(serde_json::json!(payload))
    .execute(db.pool())
    .await?;

    // Execute agent (non-blocking)
    let agent = FrontendAgent::new();
    tokio::spawn(async move {
        match agent.execute(&payload.task_description, payload.context).await {
            Ok(result) => {
                tracing::info!("Frontend agent task {} completed", task_id);
            }
            Err(e) => {
                tracing::error!("Frontend agent task {} failed: {:?}", task_id, e);
            }
        }
    });

    Ok(Json(AgentTaskResponse {
        task_id,
        agent_type: "frontend".to_string(),
        status: "processing".to_string(),
    }))
}

pub async fn backend_agent(
    State(db): State<Arc<Database>>,
    Json(payload): Json<AgentRequest>,
) -> AppResult<Json<AgentTaskResponse>> {
    let task_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO agent_tasks (id, project_id, agent_type, status, request_data) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&task_id)
    .bind(&payload.project_id)
    .bind("backend")
    .bind("processing")
    .bind(serde_json::json!(payload))
    .execute(db.pool())
    .await?;

    let agent = BackendAgent::new();
    tokio::spawn(async move {
        match agent.execute(&payload.task_description, payload.context).await {
            Ok(result) => {
                tracing::info!("Backend agent task {} completed", task_id);
            }
            Err(e) => {
                tracing::error!("Backend agent task {} failed: {:?}", task_id, e);
            }
        }
    });

    Ok(Json(AgentTaskResponse {
        task_id,
        agent_type: "backend".to_string(),
        status: "processing".to_string(),
    }))
}

pub async fn qa_agent(
    State(db): State<Arc<Database>>,
    Json(payload): Json<AgentRequest>,
) -> AppResult<Json<AgentTaskResponse>> {
    let task_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO agent_tasks (id, project_id, agent_type, status, request_data) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&task_id)
    .bind(&payload.project_id)
    .bind("qa")
    .bind("processing")
    .bind(serde_json::json!(payload))
    .execute(db.pool())
    .await?;

    let agent = QAAgent::new();
    tokio::spawn(async move {
        match agent.execute(&payload.task_description, payload.context).await {
            Ok(result) => {
                tracing::info!("QA agent task {} completed", task_id);
            }
            Err(e) => {
                tracing::error!("QA agent task {} failed: {:?}", task_id, e);
            }
        }
    });

    Ok(Json(AgentTaskResponse {
        task_id,
        agent_type: "qa".to_string(),
        status: "processing".to_string(),
    }))
}

pub async fn get_task_status(
    State(db): State<Arc<Database>>,
    Path(task_id): Path<Uuid>,
) -> AppResult<Json<AgentTaskStatus>> {
    let row = sqlx::query("SELECT id, agent_type, status, result_data FROM agent_tasks WHERE id = $1")
        .bind(&task_id)
        .fetch_optional(db.pool())
        .await?;

    let row = match row {
        Some(r) => r,
        None => {
            return Ok(Json(AgentTaskStatus {
                task_id,
                status: "not_found".to_string(),
                progress: 0.0,
                result: None,
            }))
        }
    };

    let status: String = row.get("status");
    let progress = match status.as_str() {
        "processing" => 50.0,
        "completed" => 100.0,
        "failed" => 0.0,
        _ => 25.0,
    };

    Ok(Json(AgentTaskStatus {
        task_id,
        status,
        progress,
        result: row.get("result_data"),
    }))
}
