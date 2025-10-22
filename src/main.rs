mod error;
mod state;
mod config;
mod router;
mod proxy;

use crate::proxy::proxy_handler;
use crate::state::AppState;

use axum::Router;
use std::sync::Arc;
use tower::ServiceBuilder;
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let state = Arc::new(AppState::new());

    let app = Router::new()
        .fallback(proxy_handler)
        .layer(ServiceBuilder::new()
            .layer(tower_http::trace::TraceLayer::new_for_http()))
        .with_state(state);

    let addr = "127.0.0.1:3000";

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    info!("Proxy server listening on {}", addr);
    info!("Forwarding requests to backend");

    axum::serve(listener, app).await.unwrap();
}

