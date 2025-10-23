mod config;
mod error;
mod proxy;
mod router;
mod state;

use crate::proxy::proxy_handler;
use crate::state::AppState;

use axum::Router;
use axum::routing::get;
use axum_client_ip::ClientIpSource;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/health", get(proxy::health_check))
        .fallback(proxy_handler)
        .layer(
            ServiceBuilder::new()
                .layer(ClientIpSource::ConnectInfo.into_extension())
                .layer(tower_http::trace::TraceLayer::new_for_http()),
        )
        .with_state(state);

    let addr = "127.0.0.1:3000";

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    info!("Proxy server listening on {}", addr);
    info!("Forwarding requests to backend");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
