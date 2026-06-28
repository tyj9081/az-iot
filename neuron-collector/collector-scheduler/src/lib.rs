use collector_driver::{probe_serial_async, DriverFactory};
use collector_model::{BusType, Device, LatestReading};
use collector_reporter::{Aggregator, AlarmEngine};
use collector_uploader::Uploader;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
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
    /// 每个设备一个聚合器 (device_id → Aggregator)
    aggregators: Arc<tokio::sync::Mutex<HashMap<i64, Aggregator>>>,
    /// 每个设备一个告警引擎 (device_id → AlarmEngine)
    alarm_engines: Arc<tokio::sync::Mutex<HashMap<i64, AlarmEngine>>>,
}

impl Collector {
    pub fn new(registry: SharedRegistry, uploader: Arc<Uploader>) -> Self {
        Self {
            registry,
            uploader,
            aggregators: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            alarm_engines: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!(
            "Scheduler running with {} devices, channel: {}",
            self.registry.read().await.devices.len(),
            self.uploader.active_channel().await
        );

        let mut tick = 0u64;
        let mut probed_devices: HashSet<i64> = HashSet::new();
        loop {
            let devices: Vec<Device> = {
                let registry = self.registry.read().await;
                registry.devices.values().cloned().collect()
            };

            // Cleanup stale aggregators and alarm engines for removed devices
            {
                let current_ids: HashSet<i64> = devices.iter().map(|d| d.id).collect();
                let mut aggs = self.aggregators.lock().await;
                aggs.retain(|id, _| current_ids.contains(id));
                drop(aggs);
                let mut engines = self.alarm_engines.lock().await;
                engines.retain(|id, _| current_ids.contains(id));
            }

            // 心跳日志, 每 30 秒打印一次设备数（无设备时减少噪音）
            tick += 1;
            if devices.is_empty() && tick % 30 == 1 {
                tracing::info!("Scheduler tick={} devices=0 (idle)", tick);
            }

            // ── 串口可用性预检 ──────────────────────────
            // 对新出现或重启后未探过的串口设备，做一次快速端口打开测试。
            // 通过则静默记录，失败则立刻以 ERROR 级别告警。
            for device in &devices {
                if probed_devices.contains(&device.id) {
                    continue;
                }
                if let BusType::Serial { port_name, bus_param } = &device.bus {
                    match probe_serial_async(port_name, bus_param).await {
                        Ok(()) => {
                            tracing::info!(
                                "Port probe OK  device={} port={} protocol={}",
                                device.id, port_name, device.protocol.code()
                            );
                        }
                        Err(e) => {
                            tracing::error!(
                                "Port probe FAIL device={} name={} port={} protocol={}: {:#}",
                                device.id, device.name, port_name, device.protocol.code(), e
                            );
                        }
                    }
                }
                probed_devices.insert(device.id);
            }

            for device in devices {
                let dev_id = device.id;
                let proto = device.protocol.code().to_string();
                let start = Instant::now();

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

                // 带超时保护的 spawn_blocking
                let device_clone = device.clone();
                let timeout_result = tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    tokio::task::spawn_blocking(move || driver.collect(&device_clone)),
                ).await;

                let elapsed_ms = start.elapsed().as_millis() as u64;

                // ── 解析嵌套结果 ────────────────────────────
                enum PollOutcome {
                    Success(HashMap<String, f64>),
                    Failure { error_type: &'static str },
                }

                let mut last_err_msg = String::new();

                let outcome = match timeout_result {
                    // 正常完成
                    Ok(Ok(collect_r)) => match collect_r {
                        Ok(values) if values.is_empty() => {
                            last_err_msg = "empty result (no data points collected)".into();
                            PollOutcome::Failure { error_type: "empty_result" }
                        }
                        Ok(values) => PollOutcome::Success(values),
                        Err(err) => {
                            // 遍历 anyhow error chain 查找底层错误类型。
                            // downcast_ref 只查最外层，会被 .with_context() 遮蔽，
                            // 所以需要同时用 root_cause() 到达 error chain 底部。
                            last_err_msg = format!("{:#}", err);
                            let error_type =
                                if err.downcast_ref::<std::io::Error>().is_some()
                                    || err.root_cause().is::<std::io::Error>()
                                {
                                    "io_error"
                                } else if err.downcast_ref::<serde_json::Error>().is_some()
                                    || err.root_cause().is::<serde_json::Error>()
                                {
                                    "parse_error"
                                } else {
                                    "unknown_error"
                                };
                            PollOutcome::Failure { error_type }
                        }
                    },
                    // spawn_blocking panic (driver 内部 bug)
                    Ok(Err(join_err)) => {
                        last_err_msg = join_err.to_string();
                        tracing::warn!(
                            "Collect task panicked for device {}: {}",
                            dev_id, join_err
                        );
                        PollOutcome::Failure { error_type: "driver_panic" }
                    }
                    // 超时
                    Err(_elapsed) => {
                        last_err_msg = "timeout after 5s".into();
                        PollOutcome::Failure { error_type: "timeout" }
                    }
                };

                // ── 设备状态更新 + 状态上报 ─────────────────
                let mut status_payload: Option<String> = None;

                match &outcome {
                    PollOutcome::Success(values) => {
                        let count = values.len();
                        tracing::info!(
                            "Device {} OK {}pts protocol={} elapsed={}ms",
                            dev_id, count, proto, elapsed_ms
                        );

                        // 更新 registry
                        {
                            let mut registry = self.registry.write().await;
                            if let Some(dev) = registry.devices.get_mut(&dev_id) {
                                dev.consecutive_failures = 0;
                                dev.last_success_at = Some(chrono::Utc::now().timestamp());
                                dev.last_error_msg.clear();
                                if !dev.online {
                                    dev.online = true;
                                    let payload = serde_json::json!({
                                        "device_id": dev_id,
                                        "status": "online",
                                        "timestamp": chrono::Utc::now().to_rfc3339(),
                                    }).to_string();
                                    status_payload = Some(payload);
                                }
                            }
                        }
                    }
                    PollOutcome::Failure { error_type } => {
                        let mut consecutive = 0u32;
                        {
                            let mut registry = self.registry.write().await;
                            if let Some(dev) = registry.devices.get_mut(&dev_id) {
                                dev.consecutive_failures += 1;
                                dev.last_error_at = Some(chrono::Utc::now().timestamp());
                                dev.last_error_msg = last_err_msg.clone();
                                consecutive = dev.consecutive_failures;
                                if dev.consecutive_failures >= 3 && dev.online {
                                    dev.online = false;
                                    let payload = serde_json::json!({
                                        "device_id": dev_id,
                                        "status": "offline",
                                        "timestamp": chrono::Utc::now().to_rfc3339(),
                                    }).to_string();
                                    status_payload = Some(payload);
                                }
                            }
                        }
                        tracing::warn!(
                            "Device {} FAIL protocol={} error_type={} elapsed={}ms consecutive={}",
                            dev_id, proto, error_type, elapsed_ms, consecutive
                        );
                    }
                }

                // 发布设备状态变更
                if let Some(payload) = status_payload {
                    let client_id = self.uploader.client_id().to_string();
                    let topic = format!("{}/{}/status", "neuron", client_id);
                    self.uploader.publish_raw(&topic, &payload).await;
                    tracing::info!("Device {} status published to {}", dev_id, topic);
                }

                // 失败时跳过后续数据处理
                let values = match outcome {
                    PollOutcome::Success(v) => v,
                    PollOutcome::Failure { .. } => continue,
                };

                // ── Channel A: 实时上报 ──────────────────
                {
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

                // ── Channel B: 聚合上报 ──────────────────
                {
                    let mut aggs = self.aggregators.lock().await;
                    let aggregator = aggs
                        .entry(device.id)
                        .or_insert_with(|| Aggregator::new(device.id, 60));
                    let now_ts = chrono::Utc::now().timestamp();
                    for point in &device.data_points {
                        if let Some(value) = values.get(&point.sensor_code) {
                            aggregator.push(device.id, &point.sensor_code, *value, now_ts);
                        }
                    }
                    if aggregator.is_ready() {
                        let readings = aggregator.flush();
                        aggregator.reset();
                        drop(aggs);

                        let client_id = self.uploader.client_id();
                        for agg in readings {
                            let topic = format!(
                                "{}/{}/reading",
                                "neuron", client_id
                            );
                            match serde_json::to_string(&agg) {
                                Ok(payload) => {
                                    self.uploader.publish_raw(&topic, &payload).await;
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Serialize AggregatedReading error: {}", e
                                    );
                                }
                            }
                        }
                    }
                }

                // ── 告警检测 ────────────────────────────
                {
                    let payload_to_send: Vec<(String, String)> = {
                        let mut engines = self.alarm_engines.lock().await;
                        let alarm_engine = engines
                            .entry(device.id)
                            .or_insert_with(|| AlarmEngine::new(&device.alarm_config));

                        let mut to_send = Vec::new();
                        for point in &device.data_points {
                            if let Some(value) = values.get(&point.sensor_code) {
                                if let Some(event) = alarm_engine.check(&device, &point.sensor_code, *value) {
                                    let client_id = self.uploader.client_id();
                                    let topic = format!(
                                        "{}/{}/alarm",
                                        "neuron", client_id
                                    );
                                    match serde_json::to_string(&event) {
                                        Ok(payload) => {
                                            tracing::warn!(
                                                "Alarm triggered: device={} sensor={} level={}",
                                                event.device_id,
                                                event.sensor_code,
                                                event.level
                                            );
                                            to_send.push((topic, payload));
                                        }
                                        Err(e) => {
                                            tracing::error!(
                                                "Serialize AlarmEvent error: {}", e
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        to_send
                    }; // lock released here
                    for (topic, payload) in payload_to_send {
                        self.uploader.publish_raw(&topic, &payload).await;
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
            online: false,
            consecutive_failures: 0,
            last_success_at: None,
            last_error_at: None,
            last_error_msg: String::new(),
        };
        reg.register(device);
        assert!(reg.get(1).is_some());
    }
}
