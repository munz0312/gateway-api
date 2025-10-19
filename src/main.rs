use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Request, StatusCode, Uri},
    response::{IntoResponse, Response},
};

use hyper_util::{
    client::legacy::{
        connect::HttpConnector, Client
    }, rt::TokioExecutor
};
use std::{sync::Arc};
use tower::ServiceBuilder;
use tracing::{info, error};
use tracing_subscriber;

#[derive(Clone)]
struct AppState {
    client: Client<HttpConnector, Body>,
    backend_url: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let client: Client<HttpConnector, Body> = Client::builder(TokioExecutor::new()).build_http();

    let backend_url = "http://httpbin.org".to_string();

    let state = Arc::new(AppState {
        client,
        backend_url,
    });

    let app = Router::new()
        .fallback(proxy_handler)
        .layer(
            ServiceBuilder::new()
                .layer(tower_http::trace::TraceLayer::new_for_http())
        )
        .with_state(state);

    let addr = "127.0.0.1:3000";

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    info!("Proxy server listening on {}", addr);
    info!("Forwarding requests to backend");
    
    axum::serve(listener, app).await.unwrap();
}

async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>
) -> Result<Response, ProxyError> {
    let path = req.uri().path();
    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();

    let backend_uri = format!("{}{}{}", state.backend_url, path, query);

    info!("Proxying {} {} -> {}",
        req.method(),
        req.uri(),
        backend_uri);

    let uri = backend_uri.parse::<Uri>()
    .map_err(|e| ProxyError::InvalidUri(e.to_string()))?;

    *req.uri_mut() = uri;

    let headers = req.headers_mut();
    headers.remove("connection");
    headers.remove("keep-alive");
    headers.remove("proxy-authenticate");
    headers.remove("proxy-authorization");
    headers.remove("te");
    headers.remove("trailers");
    headers.remove("transfer-encoding");
    headers.remove("upgrade");

    let response = state.client
    .request(req)
    .await
    .map_err(|e| ProxyError::BackendError(e.to_string()))?;

    Ok(response.into_response())
}

#[derive(Debug)]
enum ProxyError {
    InvalidUri(String),
    BackendError(String),
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ProxyError::InvalidUri(msg) => {
                error!("Invalid URI: {}", msg);
                (StatusCode::BAD_REQUEST, format!("Invalid URI: {}", msg))
            }
            ProxyError::BackendError(msg) => {
                error!("Backend error: {}", msg);
                (StatusCode::BAD_GATEWAY, format!("Backend error: {}", msg))
            }
        };

        (status, message).into_response()
    }
}