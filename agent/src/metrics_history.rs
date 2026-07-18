use crate::metrics::{self, RawMetrics};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

const SAMPLE_INTERVAL: Duration = Duration::from_secs(10);
const MAX_SAMPLES: usize = 360;

pub struct HistoryEntry {
    pub timestamp_unix_ms: i64,
    pub raw: RawMetrics,
}

pub type MetricsHistory = Arc<RwLock<VecDeque<HistoryEntry>>>;

pub fn start() -> MetricsHistory {
    let history: MetricsHistory = Arc::new(RwLock::new(VecDeque::with_capacity(MAX_SAMPLES)));
    let history_task = history.clone();

    tokio::spawn(async move {
        let mut ticker = interval(SAMPLE_INTERVAL);
        loop {
            ticker.tick().await;

            let Ok(Ok(raw)) = tokio::task::spawn_blocking(metrics::snapshot).await else {
                continue;
            };
            let timestamp_unix_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);

            let mut guard = history_task.write().await;
            if guard.len() >= MAX_SAMPLES {
                guard.pop_front();
            }
            guard.push_back(HistoryEntry {
                timestamp_unix_ms,
                raw,
            });
        }
    });

    history
}
