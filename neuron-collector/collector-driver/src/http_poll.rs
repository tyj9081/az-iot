//! HTTP JSON 轮询驱动 — 对 REST API 端点定时 GET，解析 JSON 响应。

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::time::Duration;

use super::ProtocolDriver;

pub struct HttpJsonDriver;

impl HttpJsonDriver { pub fn new() -> Self { Self } }

impl ProtocolDriver for HttpJsonDriver {
    fn protocol_name(&self) -> &str { "HTTP JSON" }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let base_url = match &device.bus {
            BusType::Tcp { host, port } => format!("http://{host}:{port}"),
            _ => anyhow::bail!("HTTP driver requires TCP bus (host:port)"),
        };

        let runtime = tokio::runtime::Handle::current();
        let _guard = runtime.enter();

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .context("HTTP client build")?;

        let url = format!("{base_url}/api/readings");
        let resp = client.get(&url).send().context("HTTP GET")?;
        let body: serde_json::Value = resp.json().context("JSON parse")?;

        let mut readings = HashMap::new();
        for pt in &device.data_points {
            if let Some(val) = body.get(&pt.sensor_code).and_then(|v| v.as_f64()) {
                let transformed = super::apply_transform(val, pt.coefficient, pt.offset);
                readings.insert(pt.sensor_code.clone(), transformed);
            } else if let Some(arr) = body.as_array() {
                // Array format: [{"code": "voltage_a", "value": 220.5}, ...]
                for item in arr {
                    if let Some(code) = item.get("code").and_then(|v| v.as_str()) {
                        if code == pt.sensor_code {
                            if let Some(v) = item.get("value").and_then(|v| v.as_f64()) {
                                let transformed = super::apply_transform(v, pt.coefficient, pt.offset);
                                readings.insert(pt.sensor_code.clone(), transformed);
                            }
                        }
                    }
                }
            }
        }
        Ok(readings)
    }
}
