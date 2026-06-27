use collector_driver::DriverFactory;
use collector_model::{Device, LatestReading};
use collector_uploader::Uploader;
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
        self.remove(device.id);
        self.bus_groups.entry(bus_key).or_default().push(device.id);
        self.devices.insert(device.id, device);
    }

    pub fn remove(&mut self, id: i64) {
        if let Some(device) = self.devices.remove(&id) {
            let bus_key = self.bus_key(&device);
            let mut remove_group = false;
            if let Some(ids) = self.bus_groups.get_mut(&bus_key) {
                ids.retain(|device_id| *device_id != id);
                remove_group = ids.is_empty();
            }
            if remove_group {
                self.bus_groups.remove(&bus_key);
            }
        }
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

impl Default for DeviceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedRegistry = Arc<RwLock<DeviceRegistry>>;

#[derive(Clone)]
pub struct Collector {
    pub registry: SharedRegistry,
    pub uploader: Arc<Uploader>,
}

impl Collector {
    pub fn new(registry: SharedRegistry, uploader: Arc<Uploader>) -> Self {
        Self { registry, uploader }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!(
            "Scheduler running with {} devices, channel: {}",
            self.registry.read().await.devices.len(),
            self.uploader.active_channel().await
        );

        loop {
            let devices: Vec<Device> = {
                let registry = self.registry.read().await;
                registry.devices.values().cloned().collect()
            };

            for device in devices {
                let driver = match DriverFactory::create(&device) {
                    Some(d) => d,
                    None => {
                        tracing::warn!(
                            "No driver available for device {} protocol {:?}",
                            device.id,
                            device.protocol
                        );
                        continue;
                    }
                };

                // 使用 spawn_blocking 卸到独立线程池, 避免阻塞 tokio worker
                let device_clone = device.clone();
                let result = match tokio::task::spawn_blocking(move || {
                    driver.collect(&device_clone)
                }).await {
                    Ok(r) => r,
                    Err(join_err) => {
                        tracing::warn!(
                            "Collect task panicked for device {}: {}",
                            device_clone.id, join_err
                        );
                        continue;
                    }
                };

                match result {
                    Ok(values) => {
                        for point in &device.data_points {
                            if let Some(value) = values.get(&point.sensor_code) {
                                let reading = LatestReading {
                                    device_id: device.id,
                                    sensor_code: point.sensor_code.clone(),
                                    value: *value,
                                    unit: point.unit.clone(),
                                    read_at: chrono::Utc::now().to_rfc3339(),
                                };
                                self.uploader.publish(&reading).await;
                            }
                        }
                    }
                    Err(err) => {
                        tracing::warn!("Collect failed for device {}: {}", device.id, err);
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
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
            alarm_config: None,
        };
        reg.register(device);
        assert!(reg.get(1).is_some());
    }
}
