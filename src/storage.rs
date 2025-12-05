use std::{
    collections::{HashMap, VecDeque},
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use tokio::sync::broadcast;

use crate::models::{RequestLog, SummaryMetrics, WsMessage};

#[derive(Clone)]
pub struct MetricsStore {
    request_logs: Arc<RwLock<VecDeque<RequestLog>>>,
    total_requests: Arc<AtomicU64>,
    total_errors: Arc<AtomicU64>,
    active_connections: Arc<AtomicU64>,

    route_stats: Arc<RwLock<HashMap<String, u64>>>,

    broadcaster: broadcast::Sender<WsMessage>,
}

impl MetricsStore {
    pub fn new() -> (Self, broadcast::Receiver<WsMessage>) {
        let (tx, rx) = broadcast::channel(100);
        let store = Self {
            request_logs: Arc::new(RwLock::new(VecDeque::new())),
            total_requests: Arc::new(AtomicU64::new(0)),
            total_errors: Arc::new(AtomicU64::new(0)),
            active_connections: Arc::new(AtomicU64::new(0)),
            route_stats: Arc::new(RwLock::new(HashMap::new())),
            broadcaster: tx,
        };
        (store, rx)
    }

    pub fn add_request(&self, log: RequestLog) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if log.status >= 400 {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
        }

        {
            let mut logs = self.request_logs.write().unwrap();
            if logs.len() >= 100 {
                logs.pop_front();
            }
            logs.push_back(log.clone());
        }

        {
            let mut stats = self.route_stats.write().unwrap();
            let route_path = extract_route_path(&log.path)
                .unwrap_or(&log.path)
                .to_string();
            *stats.entry(route_path).or_insert(0) += 1;
        }

        let _ = self.broadcaster.send(WsMessage::NewLog { log });
    }

    pub fn get_metrics(&self) -> SummaryMetrics {
        let recent_logs = {
            let logs = self.request_logs.read().unwrap();
            logs.iter().rev().take(50).cloned().collect()
        };

        let route_stats = self.route_stats.read().unwrap().clone();

        SummaryMetrics {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_errors: self.total_errors.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            recent_logs,
            route_stats,
        }
    }

    pub fn get_broadcaster(&self) -> broadcast::Sender<WsMessage> {
        self.broadcaster.clone()
    }

    pub fn _increment_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn _decrement_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
}

fn extract_route_path(path: &str) -> Option<&str> {
    let mut slashes = 0;
    for (i, c) in path.char_indices() {
        if c == '/' {
            slashes += 1;
            if slashes == 2 {
                return Some(&path[..i]);
            }
        }
    }
    None
}
