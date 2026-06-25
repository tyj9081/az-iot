use collector_model::{LatestReading, AggregatedReading};
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
    thresholds: HashMap<String, (f64, f64)>, // sensor_code → (min, max)
}

impl AlarmEngine {
    pub fn new() -> Self { Self { thresholds: HashMap::new() } }

    pub fn set_threshold(&mut self, sensor_code: &str, min: f64, max: f64) {
        self.thresholds.insert(sensor_code.to_string(), (min, max));
    }

    pub fn check(&self, readings: &HashMap<String, f64>) -> Vec<String> {
        let mut alarms = Vec::new();
        for (code, value) in readings {
            if let Some((min, max)) = self.thresholds.get(code) {
                if *value < *min {
                    alarms.push(format!("{} LOW: {:.2} < {:.2}", code, value, min));
                } else if *value > *max {
                    alarms.push(format!("{} HIGH: {:.2} > {:.2}", code, value, max));
                }
            }
        }
        alarms
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
        engine.set_threshold("voltage", 200.0, 240.0);
        let readings: HashMap<String, f64> = [("voltage".into(), 190.0)].into();
        let alarms = engine.check(&readings);
        assert!(!alarms.is_empty());
        assert!(alarms[0].contains("LOW"));
    }

    #[test]
    fn test_alarm_engine_high() {
        let mut engine = AlarmEngine::new();
        engine.set_threshold("voltage", 200.0, 240.0);
        let readings: HashMap<String, f64> = [("voltage".into(), 250.0)].into();
        let alarms = engine.check(&readings);
        assert!(!alarms.is_empty());
        assert!(alarms[0].contains("HIGH"));
    }

    #[test]
    fn test_alarm_engine_ok() {
        let mut engine = AlarmEngine::new();
        engine.set_threshold("voltage", 200.0, 240.0);
        let readings: HashMap<String, f64> = [("voltage".into(), 220.0)].into();
        let alarms = engine.check(&readings);
        assert!(alarms.is_empty());
    }
}
