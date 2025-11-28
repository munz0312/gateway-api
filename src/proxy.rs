use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_client_ip::ClientIp;
use chrono::Utc;
use http::{HeaderMap, Method, Request, StatusCode};
use reqwest::RequestBuilder;
use serde_json::json;
use tracing::info;

use crate::{error::ProxyError, models::RequestLog, router::match_route, state::AppState};

pub async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    ClientIp(ip): ClientIp,
    method: Method,
    headers: HeaderMap,
    req: Request<Body>,
) -> Result<Response, ProxyError> {
    let start_time = Utc::now();
    let uri = req.uri().clone();
    let path = uri.path();
    let query = uri.query().map(|q| format!("?{}", q)).unwrap_or_default();

    // println!("Client IP: {}", ip);
    // println!("X-Forwarded-For: {:?}", headers.get("x-forwarded-for"));
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

    //    info!("Headers sent by client:");
    //    for (key, value) in headers.iter() {
    //        info!("  {}: {:?}", key, value);
    //    }

    let response = RequestBuilder::send(client_req).await.unwrap();

    // info!("Version: {:?}", response.version());

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

    // Record metrics
    let end_time = Utc::now();
    let response_time = end_time.signed_duration_since(start_time);
    let status_code = status.as_u16();

    let log = RequestLog::new(
        method.to_string(),
        path.to_string(),
        status_code,
        response_time,
        ip.to_string(),
    );

    state.metrics_store.add_request(log);

    Ok(response)
}

pub async fn health_check(ClientIp(_ip): ClientIp) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "service": "gateway-api"
        })),
    )
}
