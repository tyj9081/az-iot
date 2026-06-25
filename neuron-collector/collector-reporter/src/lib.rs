use collector_model::{LatestReading, AggregatedReading, DeviceAlarmThreshold};
use std::collections::HashMap;

/// Channel A: real-time latest value (fire and forget)
pub struct RealtimePublisher {
    // In production: MQTT client handle
}

impl RealtimePublisher {
    pub fn new() -> Self { Self {} }

    pub fn publish(&self, reading: &LatestReading) {
        tracing::info!(
            "CHANNEL_A device={} sensor={} value={:.3} {} read_at={}",
            reading.device_id, reading.sensor_code, reading.value, reading.unit, reading.read_at
        );
        // P5: publish to MQTT topic neuron/{id}/latest
    }
}

/// Channel B: aggregation window → avg/max/min
pub struct Aggregator {
    window_secs: u64,
    buffers: HashMap<String, Vec<f64>>,
    last_flush: HashMap<String, std::time::Instant>,
}

impl Aggregator {
    pub fn new(window_secs: u64) -> Self { 
        Self { window_secs, buffers: HashMap::new(), last_flush: HashMap::new() }
    }

    pub fn push(&mut self, key: &str, value: f64) -> Option<AggregatedReading> {
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
            return Some(AggregatedReading {
                device_id: 0,
                sensor_code: key.to_string(),
                avg, max, min,
                sample_count: n as u32,
                window_start: "".into(),
                window_end: "".into(),
            });
        }
        None
    }
}

/// Edge alarm engine
pub struct AlarmEngine {
    thresholds: HashMap<i64, Vec<DeviceAlarmThreshold>>,
}

impl AlarmEngine {
    pub fn new() -> Self { Self { thresholds: HashMap::new() } }

    pub fn load_from_device(&mut self, device_id: i64, configs: Vec<DeviceAlarmThreshold>) {
        self.thresholds.insert(device_id, configs);
    }

    pub fn check(&self, device_id: i64, sensor_code: &str, current_value: f64, previous_value: Option<f64>) -> Vec<String> {
        let configs = match self.thresholds.get(&device_id) {
            Some(c) => c,
            None => return vec![],
        };
        let mut alarms = Vec::new();
        for cfg in configs {
            if !cfg.enabled || cfg.sensor_code != sensor_code { continue; }
            if let Some(msg) = self.evaluate(cfg, current_value, previous_value) {
                alarms.push(msg);
            }
        }
        alarms
    }

    fn evaluate(&self, cfg: &DeviceAlarmThreshold, value: f64, _prev: Option<f64>) -> Option<String> {
        let p = &cfg.params;
        match cfg.alarm_type.as_str() {
            "limit_upper" => {
                let max = p["max"].as_f64()?;
                let hyst = p.get("hysteresis").and_then(|v| v.as_f64()).unwrap_or(0.0);
                if value > max + hyst { Some(format!("HIGH: {:.2}>{:.2}", value, max)) } else { None }
            },
            "limit_lower" => {
                let min = p["min"].as_f64()?;
                let hyst = p.get("hysteresis").and_then(|v| v.as_f64()).unwrap_or(0.0);
                if value < min - hyst { Some(format!("LOW: {:.2}<{:.2}", value, min)) } else { None }
            },
            "limit_both" => {
                let min = p["min"].as_f64()?;
                let max = p["max"].as_f64()?;
                let hyst = p.get("hysteresis").and_then(|v| v.as_f64()).unwrap_or(0.0);
                if value > max + hyst { Some(format!("HIGH: {:.2}>{:.2}", value, max)) }
                else if value < min - hyst { Some(format!("LOW: {:.2}<{:.2}", value, min)) }
                else { None }
            },
            "di_change" => {
                let trigger_on = p["trigger_on"].as_i64()?;
                let int_val = value as i64;
                if int_val == trigger_on {
                    Some(format!("DI_CHANGE: triggered {}", trigger_on))
                } else { None }
            },
            "timeout" => {
                let timeout = p["timeout_sec"].as_f64()?;
                if value <= 0.0 { Some(format!("TIMEOUT: no response for {}s", timeout)) } else { None }
            },
            _ => None, // rate_rise/fall/deviation/deadband: Phase 2 实现
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_aggregator_flush() {
        // 0-second window triggers immediate flush
        let mut agg = Aggregator::new(0);
        let result = agg.push("voltage", 220.0);
        assert!(result.is_some());
        let reading = result.unwrap();
        assert_eq!(reading.sample_count, 1);
        assert_eq!(reading.sensor_code, "voltage");
    }

    #[test]
    fn test_aggregator_no_flush() {
        // large window — no flush during same test
        let mut agg = Aggregator::new(3600);
        let result = agg.push("voltage", 220.0);
        assert!(result.is_none());
        let result = agg.push("voltage", 221.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_alarm_engine_limit_upper() {
        let mut engine = AlarmEngine::new();
        let configs = vec![DeviceAlarmThreshold {
            sensor_code: "voltage".into(),
            alarm_type: "limit_upper".into(),
            enabled: true,
            level: "warning".into(),
            params: json!({"max": 240.0, "hysteresis": 0.0, "delay_count": 3}),
        }];
        engine.load_from_device(1, configs);
        let alarms = engine.check(1, "voltage", 250.0, None);
        assert_eq!(alarms.len(), 1);
        assert!(alarms[0].contains("HIGH"));
    }

    #[test]
    fn test_alarm_engine_limit_lower() {
        let mut engine = AlarmEngine::new();
        let configs = vec![DeviceAlarmThreshold {
            sensor_code: "voltage".into(),
            alarm_type: "limit_lower".into(),
            enabled: true,
            level: "warning".into(),
            params: json!({"min": 200.0, "hysteresis": 0.0, "delay_count": 3}),
        }];
        engine.load_from_device(1, configs);
        let alarms = engine.check(1, "voltage", 190.0, None);
        assert_eq!(alarms.len(), 1);
        assert!(alarms[0].contains("LOW"));
    }

    #[test]
    fn test_alarm_engine_limit_both_high() {
        let mut engine = AlarmEngine::new();
        let configs = vec![DeviceAlarmThreshold {
            sensor_code: "voltage".into(),
            alarm_type: "limit_both".into(),
            enabled: true,
            level: "warning".into(),
            params: json!({"min": 200.0, "max": 240.0, "hysteresis": 0.0, "delay_count": 3}),
        }];
        engine.load_from_device(1, configs);
        let alarms = engine.check(1, "voltage", 250.0, None);
        assert_eq!(alarms.len(), 1);
        assert!(alarms[0].contains("HIGH"));
    }

    #[test]
    fn test_alarm_engine_limit_both_low() {
        let mut engine = AlarmEngine::new();
        let configs = vec![DeviceAlarmThreshold {
            sensor_code: "voltage".into(),
            alarm_type: "limit_both".into(),
            enabled: true,
            level: "warning".into(),
            params: json!({"min": 200.0, "max": 240.0, "hysteresis": 0.0, "delay_count": 3}),
        }];
        engine.load_from_device(1, configs);
        let alarms = engine.check(1, "voltage", 190.0, None);
        assert_eq!(alarms.len(), 1);
        assert!(alarms[0].contains("LOW"));
    }

    #[test]
    fn test_alarm_engine_ok() {
        let mut engine = AlarmEngine::new();
        let configs = vec![DeviceAlarmThreshold {
            sensor_code: "voltage".into(),
            alarm_type: "limit_both".into(),
            enabled: true,
            level: "warning".into(),
            params: json!({"min": 200.0, "max": 240.0, "hysteresis": 0.0, "delay_count": 3}),
        }];
        engine.load_from_device(1, configs);
        let alarms = engine.check(1, "voltage", 220.0, None);
        assert!(alarms.is_empty());
    }

    #[test]
    fn test_alarm_engine_di_change() {
        let mut engine = AlarmEngine::new();
        let configs = vec![DeviceAlarmThreshold {
            sensor_code: "smoke".into(),
            alarm_type: "di_change".into(),
            enabled: true,
            level: "critical".into(),
            params: json!({"trigger_on": 1}),
        }];
        engine.load_from_device(1, configs);
        let alarms = engine.check(1, "smoke", 1.0, None);
        assert_eq!(alarms.len(), 1);
        assert!(alarms[0].contains("DI_CHANGE"));
    }

    #[test]
    fn test_alarm_engine_timeout() {
        let mut engine = AlarmEngine::new();
        let configs = vec![DeviceAlarmThreshold {
            sensor_code: "comm".into(),
            alarm_type: "timeout".into(),
            enabled: true,
            level: "warning".into(),
            params: json!({"timeout_sec": 30}),
        }];
        engine.load_from_device(1, configs);
        let alarms = engine.check(1, "comm", 0.0, None);
        assert_eq!(alarms.len(), 1);
        assert!(alarms[0].contains("TIMEOUT"));
    }
}
