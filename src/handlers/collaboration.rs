use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, Path, State},
    http::StatusCode,
    Json, response::IntoResponse,
};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::broadcast;
use futures::{sink::SinkExt, stream::StreamExt};
use crate::db::Database;
use crate::services::collaboration::CollaborationManager;
use crate::models::collaboration::{
    WebSocketMessage, CursorPosition, CollaborationEvent, CodeChangeEvent,
};
use crate::error::AppError;

pub async fn join_collaboration(
    State(db): State<Arc<Database>>,
    Path(project_id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let collab_manager = CollaborationManager::new();
    
    ws.on_upgrade(move |socket| {
        handle_websocket(socket, project_id, db, collab_manager)
    })
}

async fn handle_websocket(
    socket: WebSocket,
    project_id: Uuid,
    _db: Arc<Database>,
    collab_manager: Arc<CollaborationManager>,
) {
    let (mut sender, mut receiver) = socket.split();
    let user_id = Uuid::new_v4(); // In production, extract from JWT
    
    collab_manager.add_session(project_id, user_id);
    let mut rx = collab_manager.get_or_create_channel(project_id).subscribe();

    tracing::info!("User {} joined project {}", user_id, project_id);

    // Spawn a task to forward broadcast messages to the WebSocket
    let collab_clone = collab_manager.clone();
    let project_clone = project_id;
    let user_clone = user_id;

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if msg.user_id != user_clone {
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = sender.send(axum::extract::ws::Message::Text(json)).await;
                }
            }
        }
    });

    // Handle incoming WebSocket messages
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            axum::extract::ws::Message::Text(text) => {
                if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                    let channel = collab_manager.get_or_create_channel(project_id);
                    let _ = channel.send(ws_msg);
                }
            }
            axum::extract::ws::Message::Close(_) => {
                tracing::info!("User {} left project {}", user_id, project_id);
                collab_manager.remove_session(project_id, user_id);
                break;
            }
            _ => {}
        }
    }
}

pub async fn get_active_collaborators(
    State(_db): State<Arc<Database>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<Uuid>>, AppError> {
    let collab_manager = CollaborationManager::new();
    let users = collab_manager.get_active_users(project_id);
    Ok(Json(users))
}

pub async fn get_cursor_positions(
    State(_db): State<Arc<Database>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<CursorPosition>>, AppError> {
    let collab_manager = CollaborationManager::new();
    let cursors = collab_manager.get_cursors(project_id);
    Ok(Json(cursors))
}

pub async fn sync_code_state(
    State(_db): State<Arc<Database>>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CodeChangeEvent>,
) -> Result<StatusCode, AppError> {
    tracing::info!(
        "Syncing code change for project {}, file: {}",
        project_id,
        payload.file_id
    );

    Ok(StatusCode::ACCEPTED)
}

pub async fn detect_conflicts(
    State(_db): State<Arc<Database>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<String>>, AppError> {
    // Implement conflict detection logic
    Ok(Json(vec![]))
}
