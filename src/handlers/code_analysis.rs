use axum::{extract::State, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::Database,
    error::{AppError, AppResult},
    models::{OptimizeCodeRequest, ReviewCodeRequest, RefactorCodeRequest, CodeAnalysisResponse, AnalysisMetrics},
    services::ai::AIService,
};

pub async fn optimize_code(
    State(db): State<Arc<Database>>,
    Json(payload): Json<OptimizeCodeRequest>,
) -> AppResult<Json<CodeAnalysisResponse>> {
    let task_id = Uuid::new_v4();

    // Call AI service for code optimization
    let ai_service = AIService::new();
    let suggestions = ai_service.optimize(&payload.code, &payload.language).await?;

    // Store task in database
    sqlx::query(
        "INSERT INTO analysis_tasks (id, project_id, task_type, status, input_data, output_data) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(&task_id)
    .bind(&Uuid::nil()) // placeholder
    .bind("optimize")
    .bind("completed")
    .bind(serde_json::json!(payload))
    .bind(serde_json::json!(suggestions))
    .execute(db.pool())
    .await?;

    Ok(Json(CodeAnalysisResponse {
        task_id,
        suggestions: suggestions.clone(),
        optimized_code: None,
        metrics: AnalysisMetrics {
            complexity_reduction: 15.5,
            performance_gain: 22.3,
            maintainability_score: 8.2,
        },
    }))
}

pub async fn review_code(
    State(db): State<Arc<Database>>,
    Json(payload): Json<ReviewCodeRequest>,
) -> AppResult<Json<CodeAnalysisResponse>> {
    let task_id = Uuid::new_v4();

    // Call AI service for code review
    let ai_service = AIService::new();
    let suggestions = ai_service.review(&payload.code, &payload.language).await?;

    // Store task
    sqlx::query(
        "INSERT INTO analysis_tasks (id, project_id, task_type, status) VALUES ($1, $2, $3, $4)"
    )
    .bind(&task_id)
    .bind(&Uuid::nil())
    .bind("review")
    .bind("completed")
    .execute(db.pool())
    .await?;

    Ok(Json(CodeAnalysisResponse {
        task_id,
        suggestions,
        optimized_code: None,
        metrics: AnalysisMetrics {
            complexity_reduction: 0.0,
            performance_gain: 0.0,
            maintainability_score: 7.8,
        },
    }))
}

pub async fn refactor_code(
    State(db): State<Arc<Database>>,
    Json(payload): Json<RefactorCodeRequest>,
) -> AppResult<Json<CodeAnalysisResponse>> {
    let task_id = Uuid::new_v4();

    // Call AI service for code refactoring
    let ai_service = AIService::new();
    let (suggestions, refactored) = ai_service.refactor(&payload.code, &payload.language).await?;

    // Store task
    sqlx::query(
        "INSERT INTO analysis_tasks (id, project_id, task_type, status) VALUES ($1, $2, $3, $4)"
    )
    .bind(&task_id)
    .bind(&Uuid::nil())
    .bind("refactor")
    .bind("completed")
    .execute(db.pool())
    .await?;

    Ok(Json(CodeAnalysisResponse {
        task_id,
        suggestions,
        optimized_code: Some(refactored),
        metrics: AnalysisMetrics {
            complexity_reduction: 20.0,
            performance_gain: 18.0,
            maintainability_score: 8.5,
        },
    }))
}
