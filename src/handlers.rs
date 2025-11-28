use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State, WebSocketUpgrade, ws::Message},
};
use futures_util::{SinkExt, StreamExt};

use crate::{
    models::{LogQuery, RequestLog, SummaryMetrics},
    state::AppState,
};

// GET /api/metrics - Returns current metrics
pub async fn get_metrics(State(state): State<Arc<AppState>>) -> Json<SummaryMetrics> {
    Json(state.metrics_store.get_metrics())
}

// GET /api/logs?limit=50 - Returns recent logs
pub async fn get_logs(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LogQuery>,
) -> Json<Vec<RequestLog>> {
    let limit = params.limit.unwrap_or(50);
    let metrics = state.metrics_store.get_metrics();
    Json(metrics.recent_logs.into_iter().take(limit).collect())
}

// GET /api/routes - Returns current routing configuration
#[axum::debug_handler]
pub async fn get_routes(State(state): State<Arc<AppState>>) -> Json<Vec<crate::state::Route>> {
    let routes = state.routes.clone();
    Json(routes)
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> axum::response::Response {
    ws.on_upgrade(|socket| async move {
        let mut rx = state.metrics_store.get_broadcaster().subscribe();

        let (mut sender, mut receiver) = socket.split();

        tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                if let Ok(msg) = msg {
                    match msg {
                        Message::Close(_) => break, // Close message
                        Message::Ping(data) => {
                            // Respond to ping with pong
                            let _ = sender.send(Message::Pong(data)).await;
                        }
                        _ => {
                            // Handle other message types or ignore
                        }
                    }
                }
            }
        });
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    // Note: We can't use sender here as it was moved
                    // For now, just log the message
                    tracing::debug!("Broadcasting message: {}", json);
                }
            }
        });
    })
}
