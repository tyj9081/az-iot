//! Heartbeat and system metrics telemetry.
//!
//! Periodically collects host system metrics (CPU, memory, disk, uptime) and
//! publishes them to MQTT under `neuron/{client_id}/heartbeat`. Also tracks
//! collector-level metrics like device count and poll rate.

use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{Disks, System};
use tokio::sync::RwLock;

/// Collector telemetry snapshot.
#[derive(Debug, Clone, Serialize)]
pub struct TelemetrySnapshot {
    pub collector_id: String,
    pub timestamp: String,
    pub uptime_secs: u64,
    /// Number of devices in the collector registry.
    pub device_count: u64,
    /// CPU usage percentage (0.0–100.0).
    pub cpu_percent: f32,
    /// Used memory in MB.
    pub memory_used_mb: u64,
    /// Total memory in MB.
    pub memory_total_mb: u64,
    /// Used disk space on the main volume in MB.
    pub disk_used_mb: u64,
    /// Total disk space on the main volume in MB.
    pub disk_total_mb: u64,
}

/// Periodically collects system metrics and publishes heartbeat messages.
pub struct Telemetry {
    client_id: String,
    started_at: Instant,
    /// External callback to publish the snapshot (wired in collector-bin).
    /// Returns bytes to publish on MQTT.
    publisher: Arc<dyn Fn(String, String) + Send + Sync>,
    device_count: Arc<RwLock<u64>>,
}

impl Telemetry {
    /// Create a new telemetry instance.
    ///
    /// `publisher` is called with (topic, payload_json) when a heartbeat is ready.
    pub fn new(
        client_id: String,
        publisher: Arc<dyn Fn(String, String) + Send + Sync>,
        device_count: Arc<RwLock<u64>>,
    ) -> Self {
        Self {
            client_id,
            started_at: Instant::now(),
            publisher,
            device_count,
        }
    }

    /// Start the heartbeat loop (should be spawned via `tokio::spawn`).
    ///
    /// Publishes a heartbeat every 30 seconds.
    pub async fn run(&self, interval_secs: u64) {
        let mut sys = System::new_all();
        let disks = Disks::new_with_refreshed_list();

        loop {
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;

            sys.refresh_all();

            let snapshot = TelemetrySnapshot {
                collector_id: self.client_id.clone(),
                timestamp: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
                uptime_secs: self.started_at.elapsed().as_secs(),
                device_count: *self.device_count.read().await,
                cpu_percent: sys.global_cpu_usage(),
                memory_used_mb: sys.used_memory() / 1024 / 1024,
                memory_total_mb: sys.total_memory() / 1024 / 1024,
                disk_used_mb: disks.list().first().map(|d| d.total_space() - d.available_space()).unwrap_or(0) / 1024 / 1024,
                disk_total_mb: disks.list().first().map(|d| d.total_space()).unwrap_or(0) / 1024 / 1024,
            };

            let payload = match serde_json::to_string(&snapshot) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Failed to serialize telemetry: {}", e);
                    continue;
                }
            };

            let topic = format!("neuron/{}/heartbeat", self.client_id);
            (self.publisher)(topic, payload);

            tracing::debug!(
                "Heartbeat: uptime={}s devices={} cpu={:.1}% mem={}/{}MB",
                snapshot.uptime_secs,
                snapshot.device_count,
                snapshot.cpu_percent,
                snapshot.memory_used_mb,
                snapshot.memory_total_mb
            );
        }
    }
}
