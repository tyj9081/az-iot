use collector_model::Device;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DeviceRegistry {
    pub devices: HashMap<i64, Device>,
    pub bus_groups: HashMap<String, Vec<i64>>,
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            bus_groups: HashMap::new(),
        }
    }

    pub fn register(&mut self, device: Device) {
        let bus_key = self.bus_key(&device);
        self.bus_groups
            .entry(bus_key)
            .or_default()
            .push(device.id);
        self.devices.insert(device.id, device);
    }

    pub fn get(&self, id: i64) -> Option<&Device> {
        self.devices.get(&id)
    }

    fn bus_key(&self, device: &Device) -> String {
        match &device.bus {
            collector_model::BusType::Serial { port_name, .. } => port_name.clone(),
            collector_model::BusType::Tcp { host, port } => format!("{}:{}", host, port),
        }
    }
}

pub type SharedRegistry = Arc<RwLock<DeviceRegistry>>;

pub struct Collector {
    pub registry: SharedRegistry,
}

impl Collector {
    pub fn new(registry: SharedRegistry) -> Self { Self { registry } }

    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("Scheduler running with {} devices", self.registry.read().await.devices.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use collector_model::*;

    #[test]
    fn test_register_device() {
        let mut reg = DeviceRegistry::new();
        let device = Device {
            id: 1,
            code: "D1".into(),
            name: "test".into(),
            protocol: ProtocolType::ModbusRTU,
            slave_addr: 1,
            bus: BusType::Serial {
                port_name: "COM5".into(),
                bus_param: BusParam {
                    baud: 9600,
                    data_bits: 8,
                    stop_bits: 1,
                    parity: "none".into(),
                },
            },
            collect_interval_sec: None,
            data_points: vec![],
        };
        reg.register(device);
        assert!(reg.get(1).is_some());
    }
}
