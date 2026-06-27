use collector_model::ConfigDelta;
use collector_scheduler::Collector;
use collector_uploader::MqttUploadConfig;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::time::Duration;

pub async fn run(collector: Collector, mqtt: MqttUploadConfig) {
    loop {
        if let Err(err) = run_once(collector.clone(), mqtt.clone()).await {
            tracing::warn!("Config sync stopped: {err}. Reconnecting in 5s");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

async fn run_once(collector: Collector, mqtt: MqttUploadConfig) -> anyhow::Result<()> {
    let (host, port) = parse_broker_url(&mqtt.broker)
        .unwrap_or_else(|| (mqtt.broker.trim_start_matches("tcp://").to_string(), 1883));
    let client_id = format!("{}-config-sync", mqtt.client_id);
    let mut options = MqttOptions::new(client_id, host, port);
    options.set_keep_alive(Duration::from_secs(30));
    options.set_clean_session(true);
    if !mqtt.username.is_empty() {
        options.set_credentials(mqtt.username.clone(), mqtt.password.clone());
    }

    let (client, mut eventloop) = AsyncClient::new(options, 100);
    let topic = format!("{}/{}/config/delta", mqtt.topic_prefix, mqtt.client_id);
    client.subscribe(topic.clone(), QoS::AtLeastOnce).await?;
    tracing::info!("Config sync subscribed: {}", topic);

    loop {
        match eventloop.poll().await? {
            Event::Incoming(Packet::Publish(packet)) => {
                let payload = String::from_utf8_lossy(&packet.payload);
                match serde_json::from_str::<ConfigDelta>(&payload) {
                    Ok(delta) => apply_delta(&collector, delta).await,
                    Err(err) => tracing::warn!("Invalid config delta: {err}; payload={payload}"),
                }
            }
            _ => {}
        }
    }
}

async fn apply_delta(collector: &Collector, delta: ConfigDelta) {
    let mut registry = collector.registry.write().await;
    match delta.action.as_str() {
        "add" | "update" => {
            if let Some(device) = delta.device {
                let id = device.id;
                registry.register(device);
                tracing::info!("Applied config delta action={} device_id={}", delta.action, id);
            } else {
                tracing::warn!("Config delta action={} missing device", delta.action);
            }
        }
        "remove" => {
            if let Some(device) = delta.device {
                registry.remove(device.id);
                tracing::info!("Applied config delta remove device_id={}", device.id);
            } else {
                tracing::warn!("Config delta remove missing device");
            }
        }
        other => tracing::warn!("Unknown config delta action={}", other),
    }
}

fn parse_broker_url(url: &str) -> Option<(String, u16)> {
    let stripped = url.trim_start_matches("tcp://").trim_start_matches("mqtt://");
    let parts: Vec<&str> = stripped.rsplitn(2, ':').collect();
    if parts.len() == 2 {
        Some((parts[1].to_string(), parts[0].parse().ok()?))
    } else {
        None
    }
}
