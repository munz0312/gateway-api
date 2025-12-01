use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State, WebSocketUpgrade, ws::Message},
};

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
    ws.on_upgrade(|mut socket| async move {
        let mut rx = state.metrics_store.get_broadcaster().subscribe();

        loop {
            tokio::select! {
                // Handle incoming WebSocket messages (client -> server)
                msg = socket.recv() => {
                    match msg {
                        Some(Ok(Message::Close(_))) | None => break,
                        Some(Ok(Message::Ping(data))) => {
                            if socket.send(Message::Pong(data)).await.is_err() {
                                break;
                            }
                        }
                        _ => {} // Ignore others
                    }
                }
                // Handle internal broadcast messages (server -> client)
                msg = rx.recv() => {
                    if let Ok(msg) = msg {
                        if let Ok(json) = serde_json::to_string(&msg) {
                            if socket.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }
        }
    })
}
