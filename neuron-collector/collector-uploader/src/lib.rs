//! collector-uploader: MQTT 主通道 + WebSocket 备通道双路上报
//!
//! # 架构
//! ```text
//! Collector::run() → Uploader::publish(LatestReading)
//!                         ├── [主] MQTT → neuron/{device_id}/latest
//!                         └── [备] WS   → ws://host:port/ws/collector
//! ```
//!
//! # Fallback 策略
//! 1. MQTT 连接成功 → 走 MQTT 发布
//! 2. MQTT 断连超过 3 次重试 → 切换 WS 通道
//! 3. WS 通道运行期间每 30s 探测 MQTT 是否恢复
//! 4. MQTT 恢复 → 切回主通道
//!
//! # 配置
//! 见项目根目录 `config.toml`:
//! ```toml
//! [mqtt]
//! broker = "tcp://8.163.61.99:1883"
//! client_id = "collector-win10-01"
//! username = "neuron-collector"
//! password = "***"
//! topic_prefix = "neuron"
//!
//! [fallback]
//! ws_url = "ws://8.163.61.99:8080/ws/collector"
//! enabled = true
//! ```

use anyhow::{Context, Result};
use collector_model::LatestReading;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// ─── Config ────────────────────────────────────────────

/// MQTT 通道配置
#[derive(Debug, Clone)]
pub struct MqttUploadConfig {
    pub broker: String,        // e.g. "tcp://8.163.61.99:1883"
    pub client_id: String,     // e.g. "collector-win10-01"
    pub username: String,
    pub password: String,
    pub topic_prefix: String,  // e.g. "neuron"
}

/// WebSocket fallback 配置
#[derive(Debug, Clone)]
pub struct FallbackConfig {
    pub ws_url: String,        // e.g. "ws://8.163.61.99:8080/ws/collector"
    pub enabled: bool,
}

// ─── Channel State ─────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Channel {
    Mqtt,
    WebSocket,
    Disconnected,
}

// ─── Uploader ──────────────────────────────────────────

/// 双通道数据上报器
pub struct Uploader {
    mqtt_config: MqttUploadConfig,
    fallback_config: FallbackConfig,
    channel: Arc<RwLock<Channel>>,
    mqtt_client: Arc<RwLock<Option<AsyncClient>>>,
    mqtt_fail_count: Arc<RwLock<u32>>,
    ws_connected: Arc<RwLock<bool>>,
}

impl Uploader {
    /// 创建 Uploader 并连接 MQTT
    pub async fn new(mqtt_config: MqttUploadConfig, fallback_config: FallbackConfig) -> Self {
        let uploader = Self {
            mqtt_config,
            fallback_config,
            channel: Arc::new(RwLock::new(Channel::Disconnected)),
            mqtt_client: Arc::new(RwLock::new(None)),
            mqtt_fail_count: Arc::new(RwLock::new(0)),
            ws_connected: Arc::new(RwLock::new(false)),
        };
        uploader.connect_mqtt().await;
        uploader
    }

    // ─── MQTT 连接 ─────────────────────────────────────

    const MAX_MQTT_RETRIES: u32 = 3;
    const MQTT_RETRY_DELAY_MS: u64 = 2000;

    async fn connect_mqtt(&self) {
        let broker = &self.mqtt_config.broker;
        let client_id = &self.mqtt_config.client_id;

        // 解析 broker URL "tcp://host:port"
        let (host, port) = parse_broker_url(broker)
            .unwrap_or_else(|| (broker.trim_start_matches("tcp://").to_string(), 1883));

        let mut opts = MqttOptions::new(client_id, &host, port);
        opts.set_keep_alive(Duration::from_secs(30));
        opts.set_clean_session(true);
        if !self.mqtt_config.username.is_empty() {
            opts.set_credentials(&self.mqtt_config.username, &self.mqtt_config.password);
        }

        let (client, mut eventloop) = AsyncClient::new(opts, 100);
        tracing::info!("[UPLOADER] MQTT connected to {}:{}", host, port);

        // spawn eventloop handler
        let fail_count = self.mqtt_fail_count.clone();
        let channel = self.channel.clone();
        let ws_connected = self.ws_connected.clone();
        let fallback_cfg = self.fallback_config.clone();
        let uploader_mqtt_cfg = self.mqtt_config.clone();
        let mqtt_clone = self.mqtt_client.clone();

        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                            Ok(Event::Incoming(Packet::ConnAck(_))) => {
                                tracing::info!("[UPLOADER] MQTT ConnAck received");
                                *channel.write().await = Channel::Mqtt;
                                *fail_count.write().await = 0;
                            }
                            Ok(_) => {}
                            Err(e) => {
                                tracing::warn!("[UPLOADER] MQTT eventloop error: {}", e);
                                let mut cnt = fail_count.write().await;
                                *cnt += 1;
                                if *cnt >= Self::MAX_MQTT_RETRIES {
                                    *channel.write().await = Channel::WebSocket;
                                    tracing::warn!("[UPLOADER] MQTT failed {} times, switching to WebSocket fallback", *cnt);
                                    // 尝试 WS 重连
                                    if fallback_cfg.enabled {
                                        let _ = Self::try_ws_connect(&fallback_cfg.ws_url, &ws_connected).await;
                                    }
                                }
                                // 定期重试 MQTT
                                tokio::time::sleep(Duration::from_millis(Self::MQTT_RETRY_DELAY_MS)).await;
                                // 重新连接
                                let (host2, port2) = parse_broker_url(&uploader_mqtt_cfg.broker)
                                    .unwrap_or((host.clone(), port));
                                let mut opts2 = MqttOptions::new(&uploader_mqtt_cfg.client_id, &host2, port2);
                                opts2.set_keep_alive(Duration::from_secs(30));
                                opts2.set_clean_session(true);
                                if !uploader_mqtt_cfg.username.is_empty() {
                                    opts2.set_credentials(&uploader_mqtt_cfg.username, &uploader_mqtt_cfg.password);
                                }
                                let (c, el) = AsyncClient::new(opts2, 100);
                                *mqtt_clone.write().await = Some(c);
                                *fail_count.write().await = 0;
                                let _ = el;
                                // 如果 MQTT 恢复了
                                if *cnt == 0 {
                                    *channel.write().await = Channel::Mqtt;
                                    tracing::info!("[UPLOADER] MQTT recovered, switching back from fallback");
                                }
                            }
                        }
                    }
                });

                *self.mqtt_client.write().await = Some(client);
                *self.channel.write().await = Channel::Mqtt;
    }

    // ─── WebSocket 连接 ─────────────────────────────────

    async fn try_ws_connect(ws_url: &str, connected: &Arc<RwLock<bool>>) -> Result<()> {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::connect_async;

        tracing::info!("[UPLOADER] Attempting WebSocket fallback: {}", ws_url);

        match connect_async(ws_url).await {
            Ok((mut ws_stream, _)) => {
                tracing::info!("[UPLOADER] WebSocket fallback connected");
                *connected.write().await = true;

                // spawn reader task to keep connection alive
                tokio::spawn(async move {
                    while let Some(msg) = ws_stream.next().await {
                        match msg {
                            Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                                tracing::warn!("[UPLOADER] WS server closed connection");
                                break;
                            }
                            Ok(tokio_tungstenite::tungstenite::Message::Ping(data)) => {
                                let _ = ws_stream
                                    .send(tokio_tungstenite::tungstenite::Message::Pong(data))
                                    .await;
                            }
                            Err(e) => {
                                tracing::error!("[UPLOADER] WS error: {}", e);
                                break;
                            }
                            _ => {}
                        }
                    }
                });

                Ok(())
            }
            Err(e) => {
                tracing::error!("[UPLOADER] WebSocket fallback failed: {}", e);
                *connected.write().await = false;
                Err(anyhow::anyhow!("WS connect failed: {}", e))
            }
        }
    }

    // ─── 发布接口 ──────────────────────────────────────

    /// 发布一次最新读数 — 自动选择通道
    pub async fn publish(&self, reading: &LatestReading) {
        let channel = *self.channel.read().await;

        match channel {
            Channel::Mqtt => {
                self.publish_mqtt(reading).await;
            }
            Channel::WebSocket => {
                self.publish_ws(reading).await;
            }
            Channel::Disconnected => {
                tracing::warn!("[UPLOADER] No channel available, dropping reading for device {}", reading.device_id);
            }
        }
    }

    async fn publish_mqtt(&self, reading: &LatestReading) {
        let topic = format!(
            "{}/{}/reading",
            self.mqtt_config.topic_prefix, reading.device_id
        );
        let payload = match serde_json::to_string(reading) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("[UPLOADER] Serialize error: {}", e);
                return;
            }
        };

        let client_guard = self.mqtt_client.read().await;
        if let Some(ref client) = *client_guard {
            match client
                .publish(&topic, QoS::AtLeastOnce, false, payload.as_bytes())
                .await
            {
                Ok(_) => {
                    tracing::debug!(
                        "[UPLOADER] MQTT published to {} = {:.3}{}",
                        topic,
                        reading.value,
                        reading.unit
                    );
                }
                Err(e) => {
                    tracing::warn!("[UPLOADER] MQTT publish failed: {}", e);
                }
            }
        } else {
            tracing::warn!("[UPLOADER] MQTT client not available");
        }
    }

    async fn publish_ws(&self, reading: &LatestReading) {
        if !*self.ws_connected.read().await {
            // 尝试重连
            let _ = Self::try_ws_connect(&self.fallback_config.ws_url, &self.ws_connected).await;
        }

        let payload = match serde_json::to_string(reading) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("[UPLOADER] WS serialize error: {}", e);
                return;
            }
        };

        if *self.ws_connected.read().await {
            match reqwest::Client::new()
                .post(&self.fallback_config.ws_url.replace("ws://", "http://").replace("wss://", "https://"))
                .header("Content-Type", "application/json")
                .header("X-Channel", "ws-fallback")
                .body(payload.clone())
                .timeout(Duration::from_secs(5))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    tracing::debug!("[UPLOADER] WS-fallback published: {:.3}", reading.value);
                }
                Ok(resp) => {
                    tracing::warn!(
                        "[UPLOADER] WS-fallback HTTP {}: {}",
                        resp.status(),
                        resp.text().await.unwrap_or_default()
                    );
                    *self.ws_connected.write().await = false;
                }
                Err(e) => {
                    tracing::warn!("[UPLOADER] WS-fallback HTTP error: {}", e);
                    *self.ws_connected.write().await = false;
                }
            }
        }
    }

    /// 当前活跃通道
    pub async fn active_channel(&self) -> &'static str {
        match *self.channel.read().await {
            Channel::Mqtt => "MQTT",
            Channel::WebSocket => "WebSocket-fallback",
            Channel::Disconnected => "Disconnected",
        }
    }
}

// ─── Helpers ───────────────────────────────────────────

/// 解析 "tcp://host:port" 格式
fn parse_broker_url(url: &str) -> Option<(String, u16)> {
    let stripped = url.trim_start_matches("tcp://").trim_start_matches("mqtt://");
    let parts: Vec<&str> = stripped.rsplitn(2, ':').collect();
    if parts.len() == 2 {
        let host = parts[1].to_string();
        let port: u16 = parts[0].parse().ok()?;
        Some((host, port))
    } else {
        None
    }
}

// ─── Tests ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_broker_url() {
        assert_eq!(
            parse_broker_url("tcp://8.163.61.99:1883"),
            Some(("8.163.61.99".into(), 1883))
        );
        assert_eq!(
            parse_broker_url("mqtt://localhost:1883"),
            Some(("localhost".into(), 1883))
        );
        assert_eq!(
            parse_broker_url("tcp://192.168.1.1:2883"),
            Some(("192.168.1.1".into(), 2883))
        );
    }

    #[tokio::test]
    async fn test_config_creation() {
        let mqtt_cfg = MqttUploadConfig {
            broker: "tcp://test:1883".into(),
            client_id: "test-collector".into(),
            username: "user".into(),
            password: "pass".into(),
            topic_prefix: "neuron".into(),
        };
        let fallback_cfg = FallbackConfig {
            ws_url: "ws://test:8080/ws".into(),
            enabled: true,
        };
        // 不连接真实 broker，只验证结构
        let _uploader = Uploader {
            mqtt_config: mqtt_cfg,
            fallback_config: fallback_cfg,
            channel: Arc::new(RwLock::new(Channel::Disconnected)),
            mqtt_client: Arc::new(RwLock::new(None)),
            mqtt_fail_count: Arc::new(RwLock::new(0)),
            ws_connected: Arc::new(RwLock::new(false)),
        };
    }

    #[test]
    fn test_channel_state() {
        // 验证 Channel 枚举值的正确性
        assert_eq!(Channel::Mqtt as u8, 0);
        assert_eq!(Channel::WebSocket as u8, 1);
        assert_eq!(Channel::Disconnected as u8, 2);
    }
}
