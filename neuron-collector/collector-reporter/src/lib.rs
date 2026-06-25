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
    thresholds: HashMap<String, Vec<DeviceAlarmThreshold>>,
}

impl AlarmEngine {
    pub fn new() -> Self {
        Self { thresholds: HashMap::new() }
    }

    pub fn load_from_device(&mut self, device_id: i64, configs: &[DeviceAlarmThreshold]) {
        let key = format!("{}_{}", device_id, "alarms");
        self.thresholds.insert(key, configs.to_vec());
    }

    pub fn check(&self, device_id: i64, sensor_code: &str, value: f64) -> Option<Vec<String>> {
        let key = format!("{}_{}", device_id, "alarms");
        let configs = self.thresholds.get(&key)?;
        let mut alarms = Vec::new();
        for cfg in configs {
            if cfg.sensor_code != sensor_code || !cfg.enabled {
                continue;
            }
            if let Some(min) = cfg.min {
                if value < min - cfg.hysteresis {
                    alarms.push(format!("{}_LOW: {:.3}<{:.3}", sensor_code, value, min));
                }
            }
            if let Some(max) = cfg.max {
                if value > max + cfg.hysteresis {
                    alarms.push(format!("{}_HIGH: {:.3}>{:.3}", sensor_code, value, max));
                }
            }
        }
        if alarms.is_empty() { None } else { Some(alarms) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_alarm_engine_low() {
        let mut engine = AlarmEngine::new();
        let configs = vec![DeviceAlarmThreshold {
            sensor_code: "voltage".into(),
            enabled: true,
            min: Some(200.0),
            max: Some(240.0),
            hysteresis: 0.0,
            delay_count: 1,
            level: "warning".into(),
        }];
        engine.load_from_device(1, &configs);
        let alarms = engine.check(1, "voltage", 190.0);
        assert!(alarms.is_some());
        let alarm_list = alarms.unwrap();
        assert!(!alarm_list.is_empty());
        assert!(alarm_list[0].contains("LOW"));
    }

    #[test]
    fn test_alarm_engine_high() {
        let mut engine = AlarmEngine::new();
        let configs = vec![DeviceAlarmThreshold {
            sensor_code: "voltage".into(),
            enabled: true,
            min: Some(200.0),
            max: Some(240.0),
            hysteresis: 0.0,
            delay_count: 1,
            level: "warning".into(),
        }];
        engine.load_from_device(1, &configs);
        let alarms = engine.check(1, "voltage", 250.0);
        assert!(alarms.is_some());
        let alarm_list = alarms.unwrap();
        assert!(!alarm_list.is_empty());
        assert!(alarm_list[0].contains("HIGH"));
    }

    #[test]
    fn test_alarm_engine_ok() {
        let mut engine = AlarmEngine::new();
        let configs = vec![DeviceAlarmThreshold {
            sensor_code: "voltage".into(),
            enabled: true,
            min: Some(200.0),
            max: Some(240.0),
            hysteresis: 0.0,
            delay_count: 1,
            level: "warning".into(),
        }];
        engine.load_from_device(1, &configs);
        let alarms = engine.check(1, "voltage", 220.0);
        assert!(alarms.is_none());
    }
}
