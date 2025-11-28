use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Individual request information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestLog {
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub response_time: Duration,
    pub client_ip: String,
}

impl RequestLog {
    pub fn new(
        method: String,
        path: String,
        status: u16,
        response_time: Duration,
        client_ip: String,
    ) -> Self {
        RequestLog {
            timestamp: Utc::now(),
            method,
            path,
            status,
            response_time,
            client_ip,
        }
    }
}

/// Summary metrics of requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryMetrics {
    pub total_requests: u64,
    pub total_errors: u64,
    pub active_connections: u64,
    pub recent_logs: Vec<RequestLog>,
    pub route_stats: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    NewLog { log: RequestLog },
    MetricsUpdate { metrics: SummaryMetrics },
}

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    pub limit: Option<usize>,
}
