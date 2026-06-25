//! AZ-IOT Collector 入口
//!
//! 从 config.toml 加载配置, 初始化 MQTT/WS 双通道 Uploader,
//! 启动调度器循环采集并上报。

use collector_scheduler::*;
use collector_uploader::{FallbackConfig, MqttUploadConfig, Uploader};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("AZ-IOT Collector v1.0.0 starting...");

    // 加载配置 (默认值, 生产环境从 config.toml 读取)
    let config = CollectorConfig::load().unwrap_or_default();

    // 初始化双通道 Uploader
    let uploader = Arc::new(
        Uploader::new(config.mqtt, config.fallback).await,
    );
    let active = uploader.active_channel().await;
    tracing::info!("Collector initialized, active channel: {}", active);

    // 设备注册表
    let registry = Arc::new(RwLock::new(DeviceRegistry::new()));

    // 启动采集器
    let collector = Collector::new(registry, uploader.clone());

    // 启动配置同步 (后台任务)
    let sync_collector = collector.clone();
    tokio::spawn(async move {
        collector_config_sync::run(sync_collector).await;
    });

    collector.run().await?;

    tracing::info!("AZ-IOT Collector shutdown.");
    Ok(())
}

/// 采集器全局配置
#[derive(Debug, Clone)]
struct CollectorConfig {
    mqtt: MqttUploadConfig,
    fallback: FallbackConfig,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            mqtt: MqttUploadConfig {
                broker: "tcp://127.0.0.1:1883".into(),
                client_id: format!("aziot-collector-{}", rand::random::<u16>()),
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
                .unwrap_or_else(|| format!("aziot-collector-{}", rand::random::<u16>())),
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
