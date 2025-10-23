use axum::response::{IntoResponse, Response};
use http::StatusCode;
use tracing::error;

#[derive(Debug)]
pub enum ProxyError {
    BackendError(String),
    BodyError(String),
    ResponseError(String),
    RateLimitExceeded(String),
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
            ProxyError::RateLimitExceeded(msg) => {
                error!("Rate limit exceeded: {}", msg);
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    format!("Rate limit exceeded: {}", msg),
                )
            }
        };

        (status, message).into_response()
    }
}
