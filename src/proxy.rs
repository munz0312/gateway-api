use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_client_ip::{ClientIp, ClientIpSource};
use http::{HeaderMap, Method, Request, StatusCode};
use reqwest::RequestBuilder;
use serde_json::json;
use tracing::info;

use crate::{error::ProxyError, router::match_route, state::AppState};

pub async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    ClientIp(ip): ClientIp,
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
    println!("{}", ip.to_string());
    let matched = match_route(&state.routes, path).unwrap();

    let backend_url = &matched.backend_url;
    let backend_path = path.strip_prefix(&matched.path).unwrap();

    let backend_uri = format!("{}{}{}", backend_url, backend_path, query);
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

    info!("Headers sent by client:");
    for (key, value) in headers.iter() {
        info!("  {}: {:?}", key, value);
    }

    let response = RequestBuilder::send(client_req).await.unwrap();

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

pub async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "service": "gateway-api"
        })),
    )
}
