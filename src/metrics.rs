use std::{sync::Arc, time::Duration};

use tokio::time::interval;
use tracing::info;

use crate::{models::WsMessage, storage::MetricsStore};

pub struct MetricsCollector {
    store: Arc<MetricsStore>,
}

impl MetricsCollector {
    pub fn new(store: Arc<MetricsStore>) -> Self {
        Self { store }
    }

    pub async fn start_broadcasting(self: Arc<Self>) {
        let collector = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(2));

            loop {
                interval.tick().await;
                let metrics = collector.store.get_metrics();
                let _ = collector
                    .store
                    .get_broadcaster()
                    .send(WsMessage::MetricsUpdate { metrics });

                info!("Broadcasted metrics update");
            }
        });
    }
}
