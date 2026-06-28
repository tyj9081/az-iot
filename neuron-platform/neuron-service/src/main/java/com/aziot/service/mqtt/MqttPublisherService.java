package com.aziot.service.mqtt;

import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.eclipse.paho.client.mqttv3.*;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

import jakarta.annotation.PostConstruct;
import jakarta.annotation.PreDestroy;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ConcurrentMap;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * MQTT 发布服务 — 真正的 MQTT 连接（非 stub）。
 *
 * WHY:
 *   ConfigPushService 之前只 log 不实际 publish，
 *   这里封装 Paho MQTT 客户端，提供 fire-and-forget publish。
 *
 * 连接:
 *   自动连接 EMQX，失败时非阻塞重试。
 *   复用 single IMqttAsyncClient，不每发一条就建连接。
 */
@Slf4j
@Service
@ConditionalOnProperty(name = "app.mqtt.enabled", havingValue = "true", matchIfMissing = true)
public class MqttPublisherService {

    @Value("${app.mqtt.broker-url:tcp://localhost:1883}")
    private String brokerUrl;

    @Value("${app.mqtt.client-id:neuron-server-pub}")
    private String clientId;

    @Value("${app.mqtt.username:}")
    private String username;

    @Value("${app.mqtt.password:}")
    private String password;

    @Value("${app.mqtt.keep-alive-seconds:60}")
    private int keepAlive;

    @Value("${app.mqtt.connection-timeout-seconds:30}")
    private int connTimeout;

    @Value("${app.mqtt.auto-reconnect:true}")
    private boolean autoReconnect;

    private volatile IMqttAsyncClient client;
    private final AtomicBoolean connecting = new AtomicBoolean(false);

    @PostConstruct
    public void init() {
        connect();
    }

    @PreDestroy
    public void destroy() {
        try {
            if (client != null && client.isConnected()) {
                client.disconnect(2000);
            }
            if (client != null) {
                client.close();
            }
        } catch (MqttException e) {
            log.warn("MQTT disconnect failed", e);
        }
    }

    /**
     * 建立 MQTT 连接（非阻塞）
     */
    synchronized void connect() {
        if (client != null && client.isConnected()) return;
        if (!connecting.compareAndSet(false, true)) return;

        try {
            MqttConnectOptions options = new MqttConnectOptions();
            options.setAutomaticReconnect(autoReconnect);
            options.setCleanSession(true);
            options.setConnectionTimeout(connTimeout);
            options.setKeepAliveInterval(keepAlive);
            if (!username.isBlank()) {
                options.setUserName(username);
                options.setPassword(password.toCharArray());
            }

            client = new MqttAsyncClient(brokerUrl, clientId);
            client.setCallback(new MqttCallback() {
                @Override public void connectionLost(Throwable cause) {
                    log.warn("MQTT publish connection lost: {}", cause != null ? cause.getMessage() : "unknown");
                }
                @Override public void messageArrived(String topic, MqttMessage msg) {}
                @Override public void deliveryComplete(IMqttDeliveryToken token) {}
            });

            client.connect(options).waitForCompletion(connTimeout * 1000L);
            log.info("MQTT publisher connected to {}, clientId={}", brokerUrl, clientId);
        } catch (MqttException e) {
            log.error("MQTT publish connect failed to {}: {} — publish will be skipped", brokerUrl, e.getMessage());
            client = null;
        } finally {
            connecting.set(false);
        }
    }

    /**
     * 发布消息到指定 topic。
     * Fire-and-forget: 调用方不关心确认。
     * Broker 不可达时降级为 log.warn，不抛异常。
     */
    public void publish(String topic, String payloadJson) {
        if (client == null || !client.isConnected()) {
            log.warn("MQTT publish skipped (not connected): topic={}", topic);
            connect(); // 尝试重连
            return;
        }
        try {
            MqttMessage msg = new MqttMessage(payloadJson.getBytes(java.nio.charset.StandardCharsets.UTF_8));
            msg.setQos(1);
            msg.setRetained(false);
            client.publish(topic, msg);
            log.debug("MQTT PUBLISH topic={} payload={}", topic, payloadJson);
        } catch (MqttException e) {
            log.warn("MQTT publish failed topic={}: {}", topic, e.getMessage());
        }
    }
}
