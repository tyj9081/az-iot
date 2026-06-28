//! collector-uploader: MQTT 主通道 + HTTP 备通道双路上报
//!
//! # 架构
//! ```text
//! Collector::run() → Uploader::publish(LatestReading)
//!                         ├── [主] MQTT → neuron/{device_id}/reading
//!                         └── [备] HTTP → POST to fallback endpoint
//! ```
//!
//! # Fallback 策略
//! 1. MQTT 连接成功 → 走 MQTT 发布
//! 2. MQTT eventloop 连接失败 3 次 → 切换 HTTP fallback
//! 3. HTTP fallback 期间定期探测 MQTT 是否恢复
//! 4. MQTT ConnAck 收到 → 切回主通道
//!
//! # MQTT 生命周期管理
//! - 单个后台任务 `run_eventloop` 管理完整的 connect→eventloop→reconnect 循环
//! - 使用 CancellationToken 确保旧 eventloop 被取消后才启动新连接
//! - 重连时旧 client 句柄被替换为新 client, 避免 eventloop 泄露

use anyhow::{Context, Result};
use collector_model::LatestReading;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Notify, RwLock};
use tokio_util::sync::CancellationToken;

// ─── Config ────────────────────────────────────────────

/// MQTT 通道配置
#[derive(Debug, Clone)]
pub struct MqttUploadConfig {
    pub broker: String,
    pub client_id: String,
    pub username: String,
    pub password: String,
    pub topic_prefix: String,
}

/// HTTP fallback 配置 (替代原 WS 实现)
#[derive(Debug, Clone)]
pub struct FallbackConfig {
    pub ws_url: String,        // 实际作为 HTTP endpoint 使用
    pub enabled: bool,
}

// ─── Channel State ─────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Channel {
    Mqtt,
    HttpFallback,
    Disconnected,
}

// ─── Uploader ──────────────────────────────────────────

/// 双通道数据上报器 — 正确的 MQTT 生命周期管理
pub struct Uploader {
    mqtt_config: MqttUploadConfig,
    fallback_config: FallbackConfig,
    channel: Arc<RwLock<Channel>>,
    mqtt_client: Arc<RwLock<Option<AsyncClient>>>,
    ws_connected: Arc<RwLock<bool>>,
    /// 通知 eventloop 任务重启 (MQTT 恢复探测用)
    reconnect_notify: Arc<Notify>,
    /// MQTT eventloop 任务的取消令牌
    cancel_token: Arc<RwLock<Option<CancellationToken>>>,
    /// MQTT 失败计数器 — 仅 eventloop 任务写入
    mqtt_fail_count: Arc<RwLock<u32>>,
}

impl Uploader {
    /// 创建 Uploader 并异步启动 MQTT 连接
    pub async fn new(mqtt_config: MqttUploadConfig, fallback_config: FallbackConfig) -> Self {
        let reconnect_notify = Arc::new(Notify::new());

        let uploader = Self {
            mqtt_config,
            fallback_config,
            channel: Arc::new(RwLock::new(Channel::Disconnected)),
            mqtt_client: Arc::new(RwLock::new(None)),
            ws_connected: Arc::new(RwLock::new(false)),
            reconnect_notify: reconnect_notify.clone(),
            cancel_token: Arc::new(RwLock::new(None)),
            mqtt_fail_count: Arc::new(RwLock::new(0)),
        };

        // 启动 eventloop 后台任务
        uploader.spawn_eventloop().await;
        uploader
    }

    // ─── Eventloop 生命周期管理 ──────────────────────────

    /// 取消当前 eventloop (如果存在), 然后启动新的
    async fn spawn_eventloop(&self) {
        // 1. 取消旧任务
        {
            let mut token_guard = self.cancel_token.write().await;
            if let Some(token) = token_guard.take() {
                token.cancel();
            }
        }

        // 2. 短暂等待旧任务清理
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 3. 创建新取消令牌
        let cancel = CancellationToken::new();
        *self.cancel_token.write().await = Some(cancel.clone());

        // 4. 准备共享状态
        let broker = self.mqtt_config.broker.clone();
        let client_id = self.mqtt_config.client_id.clone();
        let username = self.mqtt_config.username.clone();
        let password = self.mqtt_config.password.clone();
        let channel = self.channel.clone();
        let mqtt_client = self.mqtt_client.clone();
        let ws_connected = self.ws_connected.clone();
        let fallback_cfg = self.fallback_config.clone();
        let reconnect_notify = self.reconnect_notify.clone();
        let fail_count = self.mqtt_fail_count.clone();
        let topic_prefix = self.mqtt_config.topic_prefix.clone();

        tracing::info!("[UPLOADER] Starting MQTT eventloop task");

        tokio::spawn(async move {
            // 外层循环: 重连循环
            'reconnect: loop {
                // 检查是否被取消
                if cancel.is_cancelled() {
                    tracing::info!("[UPLOADER] Eventloop task cancelled, exiting");
                    break;
                }

                // 建立 MQTT 连接
                let (host, port) = parse_broker_url(&broker)
                    .unwrap_or_else(|| (broker.trim_start_matches("tcp://").to_string(), 1883));

                let mut opts = MqttOptions::new(&client_id, &host, port);
                opts.set_keep_alive(Duration::from_secs(30));
                // LWT: 采集器断连时 EMQX 自动发布离线状态
                let lwt_topic = format!("{}/{}/status", topic_prefix, client_id);
                opts.set_last_will(rumqttc::LastWill::new(
                    lwt_topic,
                    r#"{"status":"offline"}"#,
                    QoS::AtLeastOnce,
                    true,
                ));
                opts.set_clean_session(true);
                if !username.is_empty() {
                    opts.set_credentials(&username, &password);
                }

                let (client, mut eventloop) = AsyncClient::new(opts, 100);
                tracing::info!("[UPLOADER] MQTT connecting to {}:{}", host, port);

                // 存储 client 句柄
                *mqtt_client.write().await = Some(client);
                *channel.write().await = Channel::Mqtt;
                *fail_count.write().await = 0;

                // 内层循环: 处理 eventloop 事件
                loop {
                    tokio::select! {
                        _ = cancel.cancelled() => {
                            tracing::info!("[UPLOADER] Eventloop cancelled during operation");
                            break 'reconnect;
                        }
                        _ = reconnect_notify.notified() => {
                            // 外部触发重连探测
                            tracing::debug!("[UPLOADER] Reconnect notify received (probe)");
                            if *channel.read().await == Channel::HttpFallback {
                                // 如果在 HTTP fallback, 尝试重连 MQTT
                                break; // 退出内层循环, 进入外层重连
                            }
                        }
                        event = eventloop.poll() => {
                            match event {
                                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                                    tracing::info!("[UPLOADER] MQTT ConnAck received");
                                    *channel.write().await = Channel::Mqtt;
                                    *fail_count.write().await = 0;

                                    // 上报采集器在线状态
                                    if let Some(client) = mqtt_client.read().await.as_ref() {
                                        let status_topic = format!("{}/{}/status", topic_prefix, client_id);
                                        let status_payload = r#"{"status":"online"}"#;
                                        let _ = client.publish(&status_topic, QoS::AtLeastOnce, false, status_payload);
                                        tracing::info!("[UPLOADER] Status published to {}", status_topic);
                                    }
                                }
                                Ok(_) => {}
                                Err(e) => {
                                    let mut cnt = fail_count.write().await;
                                    *cnt += 1;
                                    let current_cnt = *cnt;
                                    drop(cnt);

                                    tracing::warn!(
                                        "[UPLOADER] MQTT eventloop error (fail {}/{}): {}",
                                        current_cnt, Self::MAX_MQTT_RETRIES, e
                                    );

                                    if current_cnt >= Self::MAX_MQTT_RETRIES {
                                        // 切换到 fallback
                                        *channel.write().await = Channel::HttpFallback;
                                        tracing::warn!(
                                            "[UPLOADER] MQTT failed {} times, switching to HTTP fallback",
                                            current_cnt
                                        );

                                        // 清空客户端
                                        *mqtt_client.write().await = None;

                                        if fallback_cfg.enabled {
                                            let _ = Self::try_http_connect(
                                                &fallback_cfg.ws_url,
                                                &ws_connected,
                                            ).await;
                                        }

                                        // 定期探测 MQTT 是否恢复
                                        tokio::time::sleep(Duration::from_secs(30)).await;

                                        if cancel.is_cancelled() {
                                            break 'reconnect;
                                        }
                                        // 退出内层循环, 尝试重连
                                        break;
                                    }

                                    // 失败次数未达阈值, 等一会儿重试
                                    tokio::time::sleep(Duration::from_secs(2)).await;
                                    if cancel.is_cancelled() {
                                        break 'reconnect;
                                    }
                                    // 退出内层循环, 重连
                                    break;
                                }
                            }
                        }
                    }
                }

                // 退出内层循环后, 检查是否需要继续
                if cancel.is_cancelled() {
                    break 'reconnect;
                }
                // 否则回到 'reconnect 循环重新建立连接
            }

            // 清理
            *channel.write().await = Channel::Disconnected;
            *mqtt_client.write().await = None;
            tracing::info!("[UPLOADER] Eventloop task exited");
        });
    }

    const MAX_MQTT_RETRIES: u32 = 3;

    // ─── HTTP Fallback ──────────────────────────────────

    async fn try_http_connect(fallback_url: &str, connected: &Arc<RwLock<bool>>) -> Result<()> {
        tracing::info!("[UPLOADER] Activating HTTP fallback: {}", fallback_url);
        // 简单探测 endpoint 是否可达
        match reqwest::Client::new()
            .head(fallback_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 405 => {
                tracing::info!("[UPLOADER] HTTP fallback endpoint reachable");
                *connected.write().await = true;
                Ok(())
            }
            Ok(resp) => {
                tracing::warn!("[UPLOADER] HTTP fallback returned {}", resp.status());
                *connected.write().await = false;
                Err(anyhow::anyhow!("HTTP {} from fallback", resp.status()))
            }
            Err(e) => {
                tracing::error!("[UPLOADER] HTTP fallback unreachable: {}", e);
                *connected.write().await = false;
                Err(anyhow::anyhow!("Fallback unreachable: {}", e))
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
            Channel::HttpFallback => {
                self.publish_http_fallback(reading).await;
            }
            Channel::Disconnected => {
                tracing::warn!(
                    "[UPLOADER] No channel available, dropping reading for device {}",
                    reading.device_id
                );
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
                    tracing::info!(
                        "[UPLOADER] Published device={} sensor={} value={:.3}{}",
                        reading.device_id, reading.sensor_code, reading.value, reading.unit
                    );
                }
                Err(e) => {
                    tracing::warn!("[UPLOADER] MQTT publish failed: {}", e);
                }
            }
        } else {
            tracing::warn!("[UPLOADER] MQTT client not available");
            // 触发重连探测
            self.reconnect_notify.notify_one();
        }
    }

    /// HTTP fallback 上报 (原 publish_ws, 实际使用 HTTP POST)
    async fn publish_http_fallback(&self, reading: &LatestReading) {
        if !*self.ws_connected.read().await {
            let _ = Self::try_http_connect(&self.fallback_config.ws_url, &self.ws_connected).await;
        }

        let payload = match serde_json::to_string(reading) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("[UPLOADER] HTTP serialize error: {}", e);
                return;
            }
        };

        // 将 ws:// 转为 http:// 用于实际传输
        let http_url = if let Some(rest) = self.fallback_config.ws_url.strip_prefix("ws://") {
            format!("http://{}", rest)
        } else if let Some(rest) = self.fallback_config.ws_url.strip_prefix("wss://") {
            format!("https://{}", rest)
        } else {
            self.fallback_config.ws_url.clone()
        };

        if *self.ws_connected.read().await {
            match reqwest::Client::new()
                .post(&http_url)
                .header("Content-Type", "application/json")
                .header("X-Channel", "http-fallback")
                .body(payload.clone())
                .timeout(Duration::from_secs(5))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    tracing::debug!("[UPLOADER] HTTP-fallback published: {:.3}", reading.value);
                }
                Ok(resp) => {
                    tracing::warn!(
                        "[UPLOADER] HTTP-fallback HTTP {}: {}",
                        resp.status(),
                        resp.text().await.unwrap_or_default()
                    );
                    *self.ws_connected.write().await = false;
                }
                Err(e) => {
                    tracing::warn!("[UPLOADER] HTTP-fallback error: {}", e);
                    *self.ws_connected.write().await = false;
                }
            }
        }

        // 定期探测 MQTT 是否恢复
        self.reconnect_notify.notify_one();
    }

    /// 发布原始 payload 到指定 MQTT topic (供 telemetry 等模块使用)
    pub async fn publish_raw(&self, topic: &str, payload: &str) {
        let channel = *self.channel.read().await;
        if channel != Channel::Mqtt {
            tracing::debug!("[UPLOADER] publish_raw skipped: channel={:?}", channel);
            return;
        }
        let client_guard = self.mqtt_client.read().await;
        if let Some(ref client) = *client_guard {
            match client.publish(topic, QoS::AtLeastOnce, false, payload.as_bytes()).await {
                Ok(_) => tracing::debug!("[UPLOADER] publish_raw OK topic={}", topic),
                Err(e) => tracing::warn!("[UPLOADER] publish_raw failed topic={}: {}", topic, e),
            }
        } else {
            tracing::debug!("[UPLOADER] publish_raw skipped: no MQTT client");
        }
    }

    /// 获取 MQTT client_id，供上报模块构造 topic
    pub fn client_id(&self) -> &str {
        &self.mqtt_config.client_id
    }

    /// 当前活跃通道
    pub async fn active_channel(&self) -> &'static str {
        match *self.channel.read().await {
            Channel::Mqtt => "MQTT",
            Channel::HttpFallback => "HTTP-fallback",
            Channel::Disconnected => "Disconnected",
        }
    }
}

// ─── Helpers ───────────────────────────────────────────

/// 解析 "tcp://host:port" 或 "mqtt://host:port" 格式
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
            ws_connected: Arc::new(RwLock::new(false)),
            reconnect_notify: Arc::new(Notify::new()),
            cancel_token: Arc::new(RwLock::new(None)),
            mqtt_fail_count: Arc::new(RwLock::new(0)),
        };
    }

    #[test]
    fn test_channel_state() {
        assert_eq!(Channel::Mqtt as u8, 0);
        assert_eq!(Channel::HttpFallback as u8, 1);
        assert_eq!(Channel::Disconnected as u8, 2);
    }
}
