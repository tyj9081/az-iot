use collector_model::Device;
use rand::Rng;
use std::collections::HashMap;

pub trait ProtocolDriver: Send + Sync {
    fn protocol_name(&self) -> &str;
    fn collect(&self, device: &Device) -> anyhow::Result<HashMap<String, f64>>;
}

pub struct DriverFactory;

impl DriverFactory {
    pub fn create(device: &Device) -> Option<Box<dyn ProtocolDriver>> {
        match &device.protocol {
            collector_model::ProtocolType::ModbusRTU => {
                Some(Box::new(MockModbusDriver))
            }
            collector_model::ProtocolType::ModbusTCP => {
                Some(Box::new(MockModbusDriver))
            }
            collector_model::ProtocolType::DL645 => {
                Some(Box::new(MockModbusDriver))
            }
            _ => None,
        }
    }
}

/// Mock driver for development — generates synthetic readings.
struct MockModbusDriver;

impl ProtocolDriver for MockModbusDriver {
    fn protocol_name(&self) -> &str { "MockModbus" }

    fn collect(&self, device: &Device) -> anyhow::Result<HashMap<String, f64>> {
        let mut readings = HashMap::new();
        for dp in &device.data_points {
            // Generate synthetic but plausible values
            let base: f64 = match dp.data_type.as_str() {
                "bool" => if rand_val() > 0.5 { 1.0 } else { 0.0 },
                "float32" | "float64" => 220.0 + rand_val() * 5.0,
                "uint16" | "int16" => (100.0 + rand_val() * 50.0).round(),
                "uint32" | "int32" => (10000.0 + rand_val() * 5000.0).round(),
                _ => rand_val() * 100.0,
            };
            let value = base * dp.coefficient + dp.offset;
            readings.insert(dp.sensor_code.clone(), value);
        }
        Ok(readings)
    }
}

fn rand_val() -> f64 {
    rand::thread_rng().gen()
}
