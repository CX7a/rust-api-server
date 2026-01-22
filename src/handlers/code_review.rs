use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::Pool;
use sqlx::Postgres;
use uuid::Uuid;
use chrono::Utc;

use crate::error::ApiError;
use crate::models::collaboration::{
    CodeReview, ReviewComment, ReviewApproval, CreateCodeReviewRequest,
    UpdateCodeReviewRequest, AddReviewCommentRequest, UpdateReviewCommentRequest,
    SubmitApprovalRequest, CodeReviewDetails, DiffStat,
};
use crate::middleware::rbac;

/// Create new code review
pub async fn create_code_review(
    State(pool): State<Pool<Postgres>>,
    Path(project_id): Path<Uuid>,
    user_id: Uuid,
    Json(req): Json<CreateCodeReviewRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check write permission
    rbac::enforce_permission(&pool, user_id, project_id, "write").await?;

    let review_id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO code_reviews 
        (id, project_id, author_id, title, description, status, source_branch, target_branch, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, 'open', $6, $7, $8, $8)
        "#,
    )
    .bind(review_id)
    .bind(project_id)
    .bind(user_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.source_branch)
    .bind(&req.target_branch)
    .bind(now)
    .execute(&pool)
    .await?;

    let review = CodeReview {
        id: review_id,
        project_id,
        author_id: user_id,
        title: req.title,
        description: req.description,
        status: "open".to_string(),
        source_branch: req.source_branch,
        target_branch: req.target_branch,
        created_at: now,
        updated_at: now,
        closed_at: None,
    };

    Ok((StatusCode::CREATED, Json(review)))
}

/// Get code review with comments and approvals
pub async fn get_code_review(
    State(pool): State<Pool<Postgres>>,
    Path((project_id, review_id)): Path<(Uuid, Uuid)>,
    user_id: Uuid,
) -> Result<impl IntoResponse, ApiError> {
    // Check read permission
    rbac::enforce_permission(&pool, user_id, project_id, "read").await?;

    let review = sqlx::query_as::<_, CodeReview>(
        "SELECT * FROM code_reviews WHERE id = $1 AND project_id = $2"
    )
    .bind(review_id)
    .bind(project_id)
    .fetch_optional(&pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    let comments = sqlx::query_as::<_, ReviewComment>(
        "SELECT * FROM review_comments WHERE review_id = $1 ORDER BY created_at DESC"
    )
    .bind(review_id)
    .fetch_all(&pool)
    .await?;

    let approvals = sqlx::query_as::<_, ReviewApproval>(
        "SELECT * FROM review_approvals WHERE review_id = $1"
    )
    .bind(review_id)
    .fetch_all(&pool)
    .await?;

    let diff_stats = compute_diff_stats(&review).await;

    let details = CodeReviewDetails {
        review,
        comments,
        approvals,
        diff_stats,
    };

    Ok(Json(details))
}

/// Update code review
pub async fn update_code_review(
    State(pool): State<Pool<Postgres>>,
    Path((project_id, review_id)): Path<(Uuid, Uuid)>,
    user_id: Uuid,
    Json(req): Json<UpdateCodeReviewRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check if user is author or admin
    let is_author = sqlx::query_scalar::<_, bool>(
        "SELECT author_id = $1 FROM code_reviews WHERE id = $2"
    )
    .bind(user_id)
    .bind(review_id)
    .fetch_optional(&pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    if !is_author {
        rbac::enforce_permission(&pool, user_id, project_id, "admin").await?;
    }

    let now = Utc::now();

    sqlx::query(
        r#"
        UPDATE code_reviews 
        SET 
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            status = COALESCE($3, status),
            updated_at = $4
        WHERE id = $5
        "#,
    )
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.status)
    .bind(now)
    .bind(review_id)
    .execute(&pool)
    .await?;

    Ok(StatusCode::OK)
}

/// Add comment to review
pub async fn add_review_comment(
    State(pool): State<Pool<Postgres>>,
    Path((project_id, review_id)): Path<(Uuid, Uuid)>,
    user_id: Uuid,
    Json(req): Json<AddReviewCommentRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check write permission
    rbac::enforce_permission(&pool, user_id, project_id, "write").await?;

    let comment_id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO review_comments 
        (id, review_id, author_id, file_path, line_number, content, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
        "#,
    )
    .bind(comment_id)
    .bind(review_id)
    .bind(user_id)
    .bind(&req.file_path)
    .bind(req.line_number)
    .bind(&req.content)
    .bind(now)
    .execute(&pool)
    .await?;

    let comment = ReviewComment {
        id: comment_id,
        review_id,
        author_id: user_id,
        file_path: req.file_path,
        line_number: req.line_number,
        content: req.content,
        resolved: false,
        created_at: now,
        updated_at: now,
    };

    Ok((StatusCode::CREATED, Json(comment)))
}

/// Update review comment
pub async fn update_review_comment(
    State(pool): State<Pool<Postgres>>,
    Path((project_id, review_id, comment_id)): Path<(Uuid, Uuid, Uuid)>,
    user_id: Uuid,
    Json(req): Json<UpdateReviewCommentRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check if user is comment author
    let is_author = sqlx::query_scalar::<_, bool>(
        "SELECT author_id = $1 FROM review_comments WHERE id = $2"
    )
    .bind(user_id)
    .bind(comment_id)
    .fetch_optional(&pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    if !is_author {
        return Err(ApiError::Forbidden);
    }

    let now = Utc::now();

    sqlx::query(
        r#"
        UPDATE review_comments 
        SET 
            content = COALESCE($1, content),
            resolved = COALESCE($2, resolved),
            updated_at = $3
        WHERE id = $4
        "#,
    )
    .bind(&req.content)
    .bind(req.resolved)
    .bind(now)
    .bind(comment_id)
    .execute(&pool)
    .await?;

    Ok(StatusCode::OK)
}

/// Submit review approval
pub async fn submit_approval(
    State(pool): State<Pool<Postgres>>,
    Path((project_id, review_id)): Path<(Uuid, Uuid)>,
    user_id: Uuid,
    Json(req): Json<SubmitApprovalRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check write permission
    rbac::enforce_permission(&pool, user_id, project_id, "write").await?;

    let approval_id = Uuid::new_v4();
    let now = Utc::now();

    // Upsert approval (update if exists, create if not)
    sqlx::query(
        r#"
        INSERT INTO review_approvals 
        (id, review_id, reviewer_id, status, comments, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (review_id, reviewer_id) DO UPDATE SET
        status = $4,
        comments = $5
        "#,
    )
    .bind(approval_id)
    .bind(review_id)
    .bind(user_id)
    .bind(&req.status)
    .bind(&req.comments)
    .bind(now)
    .execute(&pool)
    .await?;

    let approval = ReviewApproval {
        id: approval_id,
        review_id,
        reviewer_id: user_id,
        status: req.status,
        comments: req.comments,
        created_at: now,
    };

    Ok((StatusCode::CREATED, Json(approval)))
}

/// Get review approvals
pub async fn get_approvals(
    State(pool): State<Pool<Postgres>>,
    Path((project_id, review_id)): Path<(Uuid, Uuid)>,
    user_id: Uuid,
) -> Result<impl IntoResponse, ApiError> {
    // Check read permission
    rbac::enforce_permission(&pool, user_id, project_id, "read").await?;

    let approvals = sqlx::query_as::<_, ReviewApproval>(
        "SELECT * FROM review_approvals WHERE review_id = $1"
    )
    .bind(review_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(approvals))
}

/// Compute diff statistics (placeholder for actual diff engine)
async fn compute_diff_stats(review: &CodeReview) -> Vec<DiffStat> {
    vec![
        DiffStat {
            file_path: "src/main.rs".to_string(),
            additions: 42,
            deletions: 10,
        },
    ]
}
