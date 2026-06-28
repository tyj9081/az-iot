//! JSON-file local storage for offline buffering.
//!
//! When the MQTT connection is unavailable, aggregated readings are persisted
//! to a local file (newline-delimited JSON). On reconnection, buffered data is
//! flushed to the broker. A periodic cleanup task removes old entries.
//!
//! # Retention
//!
//! Default: 7 days (configurable). Cleanup runs every hour.
//!
//! # Why JSON file instead of SQLite?
//!
//! This sandbox environment has no C compiler available, which is required for
//! compiling SQLite from source (the `bundled` feature of rusqlite). A JSON-based
//! approach uses only pure Rust crates (serde_json) and is sufficient for the
//! offline buffering use case in an MVP.

use collector_model::{AggregatedReading, Device};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing;

/// A reading persisted to the local file store.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredReading {
    id: u64,
    device_id: i64,
    sensor_code: String,
    avg: f64,
    max: f64,
    min: f64,
    sample_count: u32,
    window_start: String,
    window_end: String,
    created_at: String,
}

pub struct LocalStorage {
    file_path: PathBuf,
    retention_days: u32,
    /// Monotonically increasing ID for ordering.
    next_id: Mutex<u64>,
    /// Write handle (appended only).
    writer: Mutex<File>,
}

impl LocalStorage {
    /// Open (or create) the JSON storage file.
    pub fn open(db_path: &str, retention_days: u32) -> anyhow::Result<Self> {
        let path = Path::new(db_path);
        let exists = path.exists();

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path)?;

        let next_id = if exists {
            // Scan existing file for the max ID
            let reader = BufReader::new(File::open(path)?);
            let mut max_id = 0u64;
            for line in reader.lines().flatten() {
                if let Ok(entry) = serde_json::from_str::<serde_json::Value>(&line) {
                    if let Some(id) = entry.get("id").and_then(|v| v.as_u64()) {
                        max_id = max_id.max(id);
                    }
                }
            }
            max_id + 1
        } else {
            tracing::info!("Creating storage file at {}", db_path);
            1
        };

        let count = Self::count_lines(&path)?;
        tracing::info!(
            "LocalStorage opened: {} ({} entries, {}d retention)",
            db_path,
            count,
            retention_days
        );

        Ok(Self {
            file_path: path.to_path_buf(),
            retention_days,
            next_id: Mutex::new(next_id),
            writer: Mutex::new(file),
        })
    }

    /// Save a batch of aggregated readings to the file.
    pub fn save_batch(&self, readings: &[AggregatedReading]) -> anyhow::Result<usize> {
        let mut writer = self.writer.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();
        let mut count = 0;
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        for r in readings {
            let entry = StoredReading {
                id: *next_id,
                device_id: r.device_id,
                sensor_code: r.sensor_code.clone(),
                avg: r.avg,
                max: r.max,
                min: r.min,
                sample_count: r.sample_count,
                window_start: r.window_start.clone(),
                window_end: r.window_end.clone(),
                created_at: now.clone(),
            };
            let line = serde_json::to_string(&entry)?;
            writeln!(writer, "{}", line)?;
            *next_id += 1;
            count += 1;
        }

        if count > 0 {
            tracing::info!("LocalStorage saved {} readings", count);
        }
        Ok(count)
    }

    /// Retrieve all pending readings (not yet flushed to MQTT).
    pub fn pending_readings(&self) -> anyhow::Result<Vec<AggregatedReading>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut readings = Vec::new();

        for line in reader.lines().flatten() {
            if let Ok(entry) = serde_json::from_str::<StoredReading>(&line) {
                readings.push(AggregatedReading {
                    device_id: entry.device_id,
                    sensor_code: entry.sensor_code,
                    avg: entry.avg,
                    max: entry.max,
                    min: entry.min,
                    sample_count: entry.sample_count,
                    window_start: entry.window_start,
                    window_end: entry.window_end,
                });
            }
        }
        Ok(readings)
    }

    /// Delete entries up to `max_id` (inclusive), used after successful MQTT flush.
    pub fn delete_up_to(&self, max_id: u64) -> anyhow::Result<usize> {
        let content = fs::read_to_string(&self.file_path)?;
        let mut kept = Vec::new();
        let mut deleted = 0usize;

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(id) = entry.get("id").and_then(|v| v.as_u64()) {
                    if id <= max_id {
                        deleted += 1;
                        continue;
                    }
                }
            }
            kept.push(line.to_string());
        }

        // Rewrite file with only unflushed entries
        let mut file = File::create(&self.file_path)?;
        for line in &kept {
            writeln!(file, "{}", line)?;
        }

        if deleted > 0 {
            tracing::info!("LocalStorage deleted {} flushed readings", deleted);
        }
        Ok(deleted)
    }

    /// Remove entries older than the retention window.
    pub fn cleanup_expired(&self) -> anyhow::Result<usize> {
        let threshold = chrono::Local::now() - chrono::Duration::days(self.retention_days as i64);
        let threshold_str = threshold.format("%Y-%m-%dT%H:%M:%S").to_string();

        let content = fs::read_to_string(&self.file_path)?;
        let mut kept = Vec::new();
        let mut removed = 0usize;

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(created) = entry.get("created_at").and_then(|v| v.as_str()) {
                    if created < &threshold_str {
                        removed += 1;
                        continue;
                    }
                }
            }
            kept.push(line.to_string());
        }

        let mut file = File::create(&self.file_path)?;
        for line in &kept {
            writeln!(file, "{}", line)?;
        }

        if removed > 0 {
            tracing::info!(
                "LocalStorage cleaned up {} expired entries (>{}d)",
                removed,
                self.retention_days
            );
        }
        Ok(removed)
    }

    /// Total number of entries in the file.
    pub fn row_count(&self) -> anyhow::Result<usize> {
        Self::count_lines(&self.file_path)
    }

    // ─── Device Registry Persistence ─────────────────────

    /// Default path for device registry persistence.
    const REGISTRY_PATH: &str = "devices.json";

    /// Save devices to the default registry file.
    pub fn save_devices_static(devices: &[Device]) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(devices)?;
        fs::write(Self::REGISTRY_PATH, json)?;
        Ok(())
    }

    /// Load persisted devices from the default registry file.
    pub fn load_devices_static() -> Vec<Device> {
        let path = Path::new(Self::REGISTRY_PATH);
        if !path.exists() {
            tracing::info!("No persisted registry found, starting fresh");
            return vec![];
        }
        match fs::read_to_string(path) {
            Ok(json) => match serde_json::from_str::<Vec<Device>>(&json) {
                Ok(devices) => {
                    tracing::info!("Loaded {} devices from {}", devices.len(), REGISTRY_PATH);
                    devices
                }
                Err(e) => {
                    tracing::warn!("Failed to parse {}: {}", REGISTRY_PATH, e);
                    vec![]
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read {}: {}", REGISTRY_PATH, e);
                vec![]
            }
        }
    }

    fn count_lines(path: &Path) -> anyhow::Result<usize> {
        if !path.exists() {
            return Ok(0);
        }
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(reader.lines().filter(|l| l.is_ok()).count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_save_and_retrieve() {
        let db_path = "test_storage.jsonl";
        let _ = fs::remove_file(db_path);

        let storage = LocalStorage::open(db_path, 7).expect("open storage");
        let readings = vec![
            AggregatedReading {
                device_id: 1,
                sensor_code: "voltage_a".into(),
                avg: 220.5,
                max: 222.0,
                min: 219.0,
                sample_count: 60,
                window_start: "2026-01-01T00:00:00".into(),
                window_end: "2026-01-01T01:00:00".into(),
            },
            AggregatedReading {
                device_id: 1,
                sensor_code: "current_a".into(),
                avg: 10.2,
                max: 12.0,
                min: 9.5,
                sample_count: 60,
                window_start: "2026-01-01T00:00:00".into(),
                window_end: "2026-01-01T01:00:00".into(),
            },
        ];

        let saved = storage.save_batch(&readings).expect("save");
        assert_eq!(saved, 2);

        let pending = storage.pending_readings().expect("read");
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].sensor_code, "voltage_a");
        assert_eq!(pending[1].sensor_code, "current_a");

        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn test_delete_up_to() {
        let db_path = "test_storage_del.jsonl";
        let _ = fs::remove_file(db_path);

        let storage = LocalStorage::open(db_path, 7).expect("open");
        let readings = vec![
            AggregatedReading {
                device_id: 1, sensor_code: "t1".into(), avg: 1.0, max: 1.0, min: 1.0,
                sample_count: 1, window_start: "".into(), window_end: "".into(),
            },
        ];
        storage.save_batch(&readings).expect("save");
        storage.delete_up_to(1).expect("delete");
        assert_eq!(storage.pending_readings().expect("read").len(), 0);

        let _ = fs::remove_file(db_path);
    }
}
