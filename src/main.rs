use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};

use std::{fs};
use serde_json::{self, Value};

use http::{HeaderMap, Method};
use reqwest::Client;
use std::sync::Arc;
use tower::ServiceBuilder;
use tracing::{error, info};
use tracing_subscriber;

#[derive(Clone)]
struct AppState {
    client: Client,
    backend_url: String,
}

struct Route {
    path: String,
    backend_url: String,
}

fn get_routes() -> Vec<Route> {
    let config = fs::read_to_string("config.json").expect("Couldn't read config file");
    let config: Value = serde_json::from_str(config.as_str()).unwrap();    
    let routes = config["routes"].as_array().unwrap();

    routes.iter().map(|v| {
        Route {
            path: v["path"].as_str().unwrap().to_string(),
            backend_url: v["backend_url"].as_str().unwrap().to_string(),
        }
    }).collect()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let client = Client::builder().use_rustls_tls().build().unwrap();

    let routes = get_routes();
    
    let backend_url = routes[0].backend_url.clone();
    
    let state = Arc::new(AppState {
        client,
        backend_url,
    });

    let app = Router::new()
        .fallback(proxy_handler)
        .layer(ServiceBuilder::new().layer(tower_http::trace::TraceLayer::new_for_http()))
        .with_state(state);

    let addr = "127.0.0.1:3000";

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    info!("Proxy server listening on {}", addr);
    info!("Forwarding requests to backend");

    axum::serve(listener, app).await.unwrap();
}

async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    method: Method,
    headers: HeaderMap,
    req: Request<Body>,
) -> Result<Response, ProxyError> {
    let path = req.uri().path();
    let query = req
        .uri()
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();

    let backend_uri = format!("{}{}{}", state.backend_url, path, query);

    info!("Proxying {} {} -> {}", req.method(), req.uri(), backend_uri);

    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|e| ProxyError::BodyError(e.to_string()))?;

    let mut client_req = state.client.request(method.clone(), backend_uri);

    if !body_bytes.is_empty() {
        client_req = client_req.body(body_bytes);
    }

    for (key, value) in headers.iter() {
        let key_str = key.as_str();
        if key_str == "host"
            || key_str == "connection"
            || key_str == "keep-alive"
            || key_str == "proxy-authenticate"
            || key_str == "proxy-authorization"
            || key_str == "te"
            || key_str == "trailers"
            || key_str == "transfer-encoding"
            || key_str == "upgrade"
        {
            continue;
        }

        client_req = client_req.header(key, value);
    }

    let request = client_req.build().unwrap();

    info!("Headers sent by client:");
    for (key, value) in headers.iter() {
        info!("  {}: {:?}", key, value);
    }

    info!("Headers sent to backend:");
    for (key, value) in request.headers().iter() {
        info!("  {}: {:?}", key, value);
    }

    let response = state.client.execute(request).await.unwrap();

    info!("Version: {:?}", response.version());

    let status = response.status();
    let response_headers = response.headers().clone();
    let body_bytes = response
        .bytes()
        .await
        .map_err(|e| ProxyError::BackendError(e.to_string()))?;

    let mut axum_response = Response::builder().status(status);

    for (key, value) in response_headers.iter() {
        axum_response = axum_response.header(key, value);
    }

    let response = axum_response
        .body(Body::from(body_bytes))
        .map_err(|e| ProxyError::ResponseError(e.to_string()))?;
    Ok(response)
}

#[derive(Debug)]
enum ProxyError {
    BackendError(String),
    BodyError(String),
    ResponseError(String),
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ProxyError::BackendError(msg) => {
                error!("Backend error: {}", msg);
                (StatusCode::BAD_GATEWAY, format!("Backend error: {}", msg))
            }
            ProxyError::BodyError(msg) => {
                error!("Body error: {}", msg);
                (StatusCode::BAD_REQUEST, format!("Body error: {}", msg))
            }
            ProxyError::ResponseError(msg) => {
                error!("Response error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Response error: {}", msg),
                )
            }
        };

        (status, message).into_response()
    }
}
