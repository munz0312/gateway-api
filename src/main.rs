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
use std::time::Duration;
use tower::ServiceBuilder;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tracing::info;

struct Server {}

impl Server {
    async fn run(self) -> Result<(), std::io::Error> {
        tracing_subscriber::fmt::init();
        let state = Arc::new(AppState::new());

        let governor_conf = GovernorConfigBuilder::default()
            .per_second(5)
            .burst_size(1)
            .finish()
            .unwrap();

        let governor_limiter = governor_conf.limiter().clone();
        let interval = Duration::from_secs(60);
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(interval);
                tracing::info!("rate limiting storage size: {}", governor_limiter.len());
                governor_limiter.retain_recent();
            }
        });

        let app = Router::new()
            .route("/health", get(proxy::health_check))
            .fallback(proxy_handler)
            .layer(GovernorLayer::new(governor_conf))
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
    }
}

#[tokio::main]
async fn main() {
    let server = Server {};
    if let Err(e) = server.run().await {
        eprintln!("Server Error, {}", e);
    }
}
