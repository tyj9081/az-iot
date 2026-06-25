//! MQTT 订阅驱动 — 从 MQTT Broker 订阅设备遥测主题。
//! 使用 rumqttc 实现真实 MQTT 连接、订阅和数据接收。

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use rumqttc::{Client, MqttOptions, QoS};
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;

use super::ProtocolDriver;

pub struct MqttDriver;

impl MqttDriver { pub fn new() -> Self { Self } }

impl ProtocolDriver for MqttDriver {
    fn protocol_name(&self) -> &str { "MQTT" }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let (host, port) = match &device.bus {
            BusType::Tcp { host, port: p } => (host.clone(), *p),
            _ => anyhow::bail!("MQTT driver requires TCP bus (broker address)"),
        };

        let client_id = format!("aziot-collector-{}", device.id);
        let mut mqtt_opts = MqttOptions::new(&client_id, &host, port);
        mqtt_opts.set_keep_alive(Duration::from_secs(30));
        mqtt_opts.set_clean_session(true);

        let (mut client, mut conn) = Client::new(mqtt_opts, 100);

        // Subscribe to device topic
        let topic = format!("neuron/{}/telemetry", device.code);
        client.subscribe(&topic, QoS::AtLeastOnce)
            .context("MQTT subscribe")?;

        // Wait for incoming data (blocking with timeout)
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for notification in conn.iter() {
                if let Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish))) = notification {
                    let payload = String::from_utf8_lossy(&publish.payload).to_string();
                    let _ = tx.send(payload);
                }
            }
        });

        let payload = rx.recv_timeout(Duration::from_secs(10))
            .unwrap_or_default();

        let mut readings = HashMap::new();
        if !payload.is_empty() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&payload) {
                for pt in &device.data_points {
                    if let Some(val) = json.get(&pt.sensor_code).and_then(|v| v.as_f64()) {
                        let transformed = super::apply_transform(val, pt.coefficient, pt.offset);
                        readings.insert(pt.sensor_code.clone(), transformed);
                    }
                }
            }
        }
        Ok(readings)
    }
}
