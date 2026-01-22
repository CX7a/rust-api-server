use axum::{extract::State, Json};
use std::sync::Arc;

use crate::{
    db::Database,
    error::AppResult,
    models::{DashboardMetrics, Metric, AnalysisTask},
};
use uuid::Uuid;

pub async fn get_dashboard(
    State(db): State<Arc<Database>>,
) -> AppResult<Json<DashboardMetrics>> {
    // Query metrics
    let total_projects: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects")
        .fetch_one(db.pool())
        .await?;

    let active_agents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agent_tasks WHERE status = 'processing'")
        .fetch_one(db.pool())
        .await?;

    let recent_analyses = sqlx::query("SELECT id, project_id, task_type, status, created_at FROM analysis_tasks ORDER BY created_at DESC LIMIT 5")
        .fetch_all(db.pool())
        .await?;

    let analyses: Vec<AnalysisTask> = recent_analyses
        .iter()
        .map(|row| AnalysisTask {
            id: row.get("id"),
            project_id: row.get("project_id"),
            task_type: row.get("task_type"),
            status: row.get("status"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(Json(DashboardMetrics {
        total_projects,
        active_agents,
        code_quality_score: 8.3,
        recent_analyses: analyses,
    }))
}

pub async fn get_metrics(
    State(db): State<Arc<Database>>,
) -> AppResult<Json<Vec<Metric>>> {
    let rows = sqlx::query(
        "SELECT metric_type, value, created_at FROM analytics_metrics ORDER BY created_at DESC LIMIT 100"
    )
    .fetch_all(db.pool())
    .await?;

    let metrics: Vec<Metric> = rows
        .iter()
        .map(|row| Metric {
            metric_type: row.get("metric_type"),
            value: row.get::<f64, _>("value"),
            timestamp: row.get("created_at"),
        })
        .collect();

    Ok(Json(metrics))
}

pub async fn list_reports(
    State(db): State<Arc<Database>>,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    let rows = sqlx::query_as::<_, (serde_json::Value,)>(
        "SELECT metadata FROM analytics_metrics ORDER BY created_at DESC LIMIT 50"
    )
    .fetch_all(db.pool())
    .await?;

    let reports = rows.into_iter().map(|(metadata,)| metadata).collect();

    Ok(Json(reports))
}
