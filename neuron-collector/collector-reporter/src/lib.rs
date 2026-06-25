//! Reporter: 使用 Uploader 双通道上报采集读数
//!
//! Channel A: 实时通道 — LatestReading → MQTT (优先) / WS fallback
//! Channel B: 聚合通道 — AggregatedReading (Phase 3 实现)

use collector_model::LatestReading;
use collector_uploader::Uploader;
use std::sync::Arc;

/// 实时上报器 — 封装 Uploader
pub struct RealtimePublisher {
    uploader: Arc<Uploader>,
}

impl RealtimePublisher {
    pub fn new(uploader: Arc<Uploader>) -> Self {
        Self { uploader }
    }

    /// Channel A: 上报实时读数
    ///
    /// 自动选择 MQTT → WebSocket fallback
    pub async fn publish(&self, reading: &LatestReading) {
        self.uploader.publish(reading).await;
    }

    /// 当前活跃通道名称
    pub async fn active_channel(&self) -> &'static str {
        self.uploader.active_channel().await
    }
}

// ─── Aggregator 保持原有实现 ──────────────────────────

use std::collections::HashMap;

pub struct Aggregator {
    window_secs: u64,
    buffers: HashMap<String, Vec<f64>>,
    last_flush: HashMap<String, std::time::Instant>,
}

impl Aggregator {
    pub fn new(window_secs: u64) -> Self {
        Self {
            window_secs,
            buffers: HashMap::new(),
            last_flush: HashMap::new(),
        }
    }

    pub fn push(
        &mut self,
        key: &str,
        value: f64,
    ) -> Option<collector_model::AggregatedReading> {
        let now = std::time::Instant::now();
        let buffer = self.buffers.entry(key.to_string()).or_default();
        buffer.push(value);

        let last = self.last_flush.entry(key.to_string()).or_insert(now);
        if last.elapsed().as_secs() >= self.window_secs && !buffer.is_empty() {
            let n = buffer.len() as f64;
            let sum: f64 = buffer.iter().sum();
            let avg = sum / n;
            let max = buffer.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let min = buffer.iter().cloned().fold(f64::INFINITY, f64::min);
            buffer.clear();
            *last = now;
            return Some(collector_model::AggregatedReading {
                device_id: 0,
                sensor_code: key.to_string(),
                avg,
                max,
                min,
                sample_count: n as u32,
                window_start: "".into(),
                window_end: "".into(),
            });
        }
        None
    }
}
