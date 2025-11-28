use std::sync::Arc;

use crate::{config::extract_routes, metrics::MetricsCollector, storage::MetricsStore};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub routes: Vec<Route>,
    pub metrics_store: Arc<MetricsStore>,
    pub metrics_collector: Arc<MetricsCollector>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Route {
    pub path: String,
    pub backend_url: String,
}

impl AppState {
    pub fn new() -> Self {
        let client = Client::builder().use_rustls_tls().build().unwrap();
        let routes = extract_routes();
        let (metrics_store, _ws_receiver) = MetricsStore::new();
        let metrics_collector = Arc::new(MetricsCollector::new(Arc::new(metrics_store.clone())));
        Self {
            client,
            routes,
            metrics_store: Arc::new(metrics_store),
            metrics_collector,
        }
    }
}
