//! Reporter: 使用 Uploader 双通道上报采集读数
//!
//! Channel A: 实时通道 — LatestReading → MQTT (优先) / WS fallback
//! Channel B: 聚合通道 — AggregatedReading

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

// ─── Aggregator — 窗口聚合引擎 ──────────────────────────

use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub struct Aggregator {
    /// 聚合时间窗口（秒）
    window_secs: i64,
    /// 当前设备 ID（由 push 传入）
    device_id: i64,
    /// sensor_code → 累积的值序列
    buffers: HashMap<String, Vec<f64>>,
    /// sensor_code → 窗口开始时刻（用于判断窗口是否到期）
    window_starts: HashMap<String, DateTime<Utc>>,
}

impl Aggregator {
    /// 创建一个新的聚合器，绑定到指定设备
    pub fn new(device_id: i64, window_secs: u64) -> Self {
        Self {
            window_secs: window_secs as i64,
            device_id,
            buffers: HashMap::new(),
            window_starts: HashMap::new(),
        }
    }

    /// 向聚合窗口推入一个数据点
    ///
    /// * `device_id` - 设备 ID
    /// * `sensor_code` - 传感器编码
    /// * `value` - 采集值
    /// * `_timestamp` - 该数据点的采集时间戳（Unix 秒），预留扩展
    pub fn push(&mut self, device_id: i64, sensor_code: &str, value: f64, _timestamp: i64) {
        self.device_id = device_id;
        let now = Utc::now();
        let buffer = self.buffers.entry(sensor_code.to_string()).or_default();

        // 窗口开始时记录起始时间戳
        if buffer.is_empty() {
            self.window_starts
                .insert(sensor_code.to_string(), now);
        }

        buffer.push(value);
    }

    /// 是否存在至少一个传感器窗口已到期且有待计算的数据
    pub fn is_ready(&self) -> bool {
        let now = Utc::now();
        self.window_starts.iter().any(|(key, start)| {
            if let Some(buffer) = self.buffers.get(key) {
                if !buffer.is_empty() {
                    return (now - *start).num_seconds() >= self.window_secs;
                }
            }
            false
        })
    }

    /// 将所有到期的窗口数据计算聚合并返回
    ///
    /// 每次 flush 将清除已计算的缓冲区并复位对应窗口时间。
    pub fn flush(&mut self) -> Vec<collector_model::AggregatedReading> {
        let now = Utc::now();
        let mut results = Vec::new();

        // 收集所有到期的 key
        let ready_keys: Vec<String> = self
            .window_starts
            .iter()
            .filter(|(key, start)| {
                if let Some(buffer) = self.buffers.get(*key) {
                    !buffer.is_empty() && (now - *start).num_seconds() >= self.window_secs
                } else {
                    false
                }
            })
            .map(|(k, _)| k.clone())
            .collect();

        for key in ready_keys {
            if let Some(buffer) = self.buffers.get_mut(&key) {
                if buffer.is_empty() {
                    continue;
                }

                let n = buffer.len() as f64;
                let sum: f64 = buffer.iter().sum();
                let avg = sum / n;
                let max = buffer.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let min = buffer.iter().cloned().fold(f64::INFINITY, f64::min);

                // 窗口起止时间戳
                let window_start = self
                    .window_starts
                    .get(&key)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default();
                let window_end = now.to_rfc3339();

                results.push(collector_model::AggregatedReading {
                    device_id: self.device_id,
                    sensor_code: key.clone(),
                    avg,
                    max,
                    min,
                    sample_count: n as u32,
                    window_start,
                    window_end,
                });

                buffer.clear();
                // 复位窗口起始时间为当前时刻
                self.window_starts.insert(key.clone(), now);
            }
        }

        results
    }

    /// 清空所有缓冲区和窗口状态
    pub fn reset(&mut self) {
        self.buffers.clear();
        self.window_starts.clear();
    }
}

// ─── AlarmEngine — 告警检测引擎 ─────────────────────────

use collector_model::{Device, DeviceAlarmThreshold};
use serde::{Deserialize, Serialize};

/// 告警事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmEvent {
    pub device_id: i64,
    pub sensor_code: String,
    pub alarm_type: String,
    pub level: String,
    pub value: f64,
    pub threshold: serde_json::Value,
    pub message: String,
    pub timestamp: String,
}

/// 告警检测引擎
///
/// 根据设备配置的阈值规则检测传感器值是否触发告警。
pub struct AlarmEngine {
    /// sensor_code → 阈值规则列表
    thresholds: HashMap<String, Vec<DeviceAlarmThreshold>>,
}

impl AlarmEngine {
    /// 使用设备的告警配置初始化引擎
    ///
    /// 如果 `alarm_config` 为 `None` 或空列表，则引擎不检测任何告警。
    pub fn new(alarm_config: &Option<Vec<DeviceAlarmThreshold>>) -> Self {
        let mut thresholds: HashMap<String, Vec<DeviceAlarmThreshold>> = HashMap::new();
        if let Some(configs) = alarm_config {
            for t in configs {
                if t.enabled {
                    thresholds
                        .entry(t.sensor_code.clone())
                        .or_default()
                        .push(t.clone());
                }
            }
        }
        Self { thresholds }
    }

    /// 检查传感器值是否触发告警
    ///
    /// * `device` - 设备信息
    /// * `sensor_code` - 传感器编码
    /// * `value` - 当前采集值
    ///
    /// 返回 `Some(AlarmEvent)` 当触发告警时，否则 `None`。
    pub fn check(
        &self,
        device: &Device,
        sensor_code: &str,
        value: f64,
    ) -> Option<AlarmEvent> {
        let rules = self.thresholds.get(sensor_code)?;

        for rule in rules {
            let triggered = match rule.alarm_type.as_str() {
                "limit_upper" => {
                    rule.params.get("upper")
                        .and_then(|v| v.as_f64())
                        .map(|upper| value > upper)
                        .unwrap_or(false)
                }
                "limit_lower" => {
                    rule.params.get("lower")
                        .and_then(|v| v.as_f64())
                        .map(|lower| value < lower)
                        .unwrap_or(false)
                }
                "limit_both" => {
                    let upper_ok = rule.params.get("upper")
                        .and_then(|v| v.as_f64())
                        .map(|upper| value > upper)
                        .unwrap_or(false);
                    let lower_ok = rule.params.get("lower")
                        .and_then(|v| v.as_f64())
                        .map(|lower| value < lower)
                        .unwrap_or(false);
                    upper_ok || lower_ok
                }
                // 其他告警类型 (rate_rise, rate_fall, deviation, di_change, timeout,
                // deadband, custom) 暂未实现，可按需扩展
                _ => false,
            };

            if triggered {
                let message = format!(
                    "[{}] device={} sensor={} value={:.3} type={} level={}",
                    device.code, device.id, sensor_code, value, rule.alarm_type, rule.level
                );
                return Some(AlarmEvent {
                    device_id: device.id,
                    sensor_code: sensor_code.to_string(),
                    alarm_type: rule.alarm_type.clone(),
                    level: rule.level.clone(),
                    value,
                    threshold: rule.params.clone(),
                    message,
                    timestamp: Utc::now().to_rfc3339(),
                });
            }
        }

        None
    }
}
