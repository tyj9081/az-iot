//! AZ-IOT Collector 入口
//!
//! 从 config.toml 加载配置, 初始化 MQTT/WS 双通道 Uploader,
//! 启动调度器循环采集并上报。

use collector_scheduler::*;
use collector_storage::LocalStorage;
use collector_telemetry::Telemetry;
use collector_uploader::{FallbackConfig, MqttUploadConfig, Uploader};
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_ansi(false) // Windows 终端兼容
        .init();
    tracing::info!("AZ-IOT Collector v1.0.0 starting...");

    // 加载配置 (默认值, 生产环境从 config.toml 读取)
    let config = CollectorConfig::load().unwrap_or_default();
    let sync_mqtt = config.mqtt.clone();

    tracing::info!("Config loaded:");
    tracing::info!("  MQTT broker: {}", config.mqtt.broker);
    tracing::info!("  MQTT client_id: {}", config.mqtt.client_id);
    tracing::info!("  Topic prefix: {}", config.mqtt.topic_prefix);

    // WS 启动上报 — 通知服务端采集器身份
    let server_url = config.server_url();
    register_via_http(&server_url, &config.mqtt.client_id).await;

    // 初始化双通道 Uploader
    let uploader = Arc::new(
        Uploader::new(config.mqtt, config.fallback).await,
    );
    let active = uploader.active_channel().await;
    tracing::info!("Collector initialized, active channel: {}", active);

    // 设备注册表 — 优先从本地文件恢复
    let mut registry = DeviceRegistry::new();
    let persisted = LocalStorage::load_devices_static();
    if !persisted.is_empty() {
        for device in persisted {
            registry.register(device);
        }
    }
    let registry = Arc::new(RwLock::new(registry));

    // ─── 遥测心跳 ──────────────────────────────────────
    let device_count = Arc::new(RwLock::new(persisted.len() as u64));
    {
        let uploader = uploader.clone();
        let publisher: Arc<dyn Fn(String, String) + Send + Sync> = Arc::new(
            move |topic: String, payload: String| {
                let u = uploader.clone();
                tokio::spawn(async move {
                    u.publish_raw(&topic, &payload).await;
                });
            },
        );
        let telemetry = Telemetry::new(
            config.mqtt.client_id.clone(),
            publisher,
            device_count.clone(),
        );
        tokio::spawn(async move {
            telemetry.run(30).await;
        });
        tracing::info!("Telemetry heartbeat started (interval=30s)");
    }

    // 启动采集器
    let collector = Collector::new(registry, uploader.clone());

    // 启动配置同步 (后台任务)
    let sync_collector = collector.clone();
    let sub_topic = format!("{}/{}/config/delta", sync_mqtt.topic_prefix, sync_mqtt.client_id);
    tracing::info!("Starting config sync, subscribing to: {}", sub_topic);
    tokio::spawn(async move {
        collector_config_sync::run(sync_collector, sync_mqtt, device_count).await;
    });

    collector.run().await?;

    tracing::info!("AZ-IOT Collector shutdown.");
    Ok(())
}

/// WS 通道主动上报采集器身份到服务端
/// 失败不中断: 只记日志, 不影响后续 MQTT 流程
async fn register_via_http(server_url: &str, client_id: &str) {
    if server_url.is_empty() {
        tracing::info!("WS register skipped: no server_url configured");
        return;
    }

    #[derive(Serialize)]
    struct RegisterBody<'a> {
        mqtt_client_id: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        ip_address: Option<String>,
        version: &'a str,
    }

    let body = RegisterBody {
        mqtt_client_id: client_id,
        ip_address: local_ip(),
        version: env!("CARGO_PKG_VERSION"),
    };

    let url = format!("{}/api/v1/collectors/register", server_url.trim_end_matches('/'));
    tracing::info!("WS register: POST {} mqtt_client_id={}", url, client_id);

    match reqwest::Client::new()
        .post(&url)
        .json(&body)
        .timeout(Duration::from_secs(3))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            tracing::info!("WS register OK (HTTP {})", resp.status().as_u16());
        }
        Ok(resp) => {
            tracing::warn!("WS register skipped (HTTP {}), will retry via MQTT status", resp.status().as_u16());
        }
        Err(e) => {
            tracing::warn!("WS register skipped (unreachable: {}), will retry via MQTT status", e);
        }
    }
}

/// 获取本机 IP (仅日志用)
fn local_ip() -> Option<String> {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .ok()
        .and_then(|s| {
            s.connect("8.8.8.8:80").ok()?;
            s.local_addr().ok().map(|a| a.ip().to_string())
        })
}

/// 采集器全局配置
#[derive(Debug, Clone)]
struct CollectorConfig {
    mqtt: MqttUploadConfig,
    fallback: FallbackConfig,
}

impl CollectorConfig {
    /// 从 ws_url 推导 HTTP server_url
    fn server_url(&self) -> String {
        if self.fallback.ws_url.is_empty() {
            return String::new();
        }
        url::Url::parse(&self.fallback.ws_url)
            .map(|u| format!("http://{}:{}", 
                u.host_str().unwrap_or("127.0.0.1"),
                u.port().unwrap_or(8080)))
            .unwrap_or_default()
    }
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            mqtt: MqttUploadConfig {
                broker: "tcp://127.0.0.1:1883".into(),
                client_id: "aziot-collector-01".into(),
                username: String::new(),
                password: String::new(),
                topic_prefix: "neuron".into(),
            },
            fallback: FallbackConfig {
                ws_url: "ws://127.0.0.1:8080/ws/collector".into(),
                enabled: true,
            },
        }
    }
}

impl CollectorConfig {
    /// 从 config.toml 加载配置，文件不存在时返回 None
    fn load() -> Option<Self> {
        let path = std::path::Path::new("config.toml");
        if !path.exists() {
            tracing::warn!("config.toml not found, using defaults");
            return None;
        }
        let content = std::fs::read_to_string(path).ok()?;
        let parsed: toml::Value = content.parse().ok()?;

        let mqtt = MqttUploadConfig {
            broker: parsed.get("mqtt")?.get("broker")?.as_str()?.to_string(),
            client_id: parsed.get("mqtt")?.get("client_id")?
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "aziot-collector-01".into()),
            username: parsed.get("mqtt")?.get("username")?
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_default(),
            password: parsed.get("mqtt")?.get("password")?
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_default(),
            topic_prefix: parsed.get("mqtt")?.get("topic_prefix")?
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "neuron".into()),
        };

        let fallback = FallbackConfig {
            ws_url: parsed.get("fallback")?.get("ws_url")?
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "ws://127.0.0.1:8080/ws/collector".into()),
            enabled: parsed.get("fallback")?.get("enabled")?
                .as_bool()
                .unwrap_or(true),
        };

        Some(Self { mqtt, fallback })
    }
}
