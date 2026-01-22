use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod middleware_auth;
mod models;
mod services;
mod utils;

use config::Config;
use db::Database;
use handlers::{auth, code_analysis, agents, projects, analytics};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("compilex7=debug".parse()?),
        )
        .init();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded: {:?}", config);

    // Initialize database
    let db = Database::new(&config.database_url).await?;
    db.run_migrations().await?;
    let db = Arc::new(db);

    tracing::info!("Database migrations completed");

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        // Authentication routes
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh_token))
        .route("/auth/logout", post(auth::logout))
        // Project routes
        .route("/projects", get(projects::list_projects).post(projects::create_project))
        .route("/projects/:id", get(projects::get_project).put(projects::update_project).delete(projects::delete_project))
        .route("/projects/:id/files", get(projects::list_files))
        // Code analysis routes
        .route("/analysis/optimize", post(code_analysis::optimize_code))
        .route("/analysis/review", post(code_analysis::review_code))
        .route("/analysis/refactor", post(code_analysis::refactor_code))
        // Agent routes
        .route("/agents/frontend", post(agents::frontend_agent))
        .route("/agents/backend", post(agents::backend_agent))
        .route("/agents/qa", post(agents::qa_agent))
        .route("/agents/status/:task_id", get(agents::get_task_status))
        // Analytics routes
        .route("/analytics/dashboard", get(analytics::get_dashboard))
        .route("/analytics/metrics", get(analytics::get_metrics))
        .route("/analytics/reports", get(analytics::list_reports))
        // Protected routes middleware
        .layer(middleware::from_fn(middleware_auth::auth_middleware))
        // CORS layer
        .layer(CorsLayer::permissive())
        // Body limit
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB
        .with_state(db);

    // Start server
    let listener = TcpListener::bind(&config.server_addr).await?;
    tracing::info!("Server listening on {}", config.server_addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
