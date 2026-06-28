package com.aziot.service.mqtt;

import com.aziot.dao.entity.device.DevDeviceReading;
import com.aziot.dao.mapper.device.DevDeviceReadingMapper;
import com.aziot.dao.mapper.device.DevRegisterMapMapper;
import com.aziot.dao.mapper.collector.DevCollectorMapper;
import com.aziot.dao.entity.collector.DevCollector;
import com.aziot.dao.entity.device.DevRegisterMap;
import com.fasterxml.jackson.databind.ObjectMapper;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.eclipse.paho.client.mqttv3.*;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

import jakarta.annotation.PostConstruct;
import jakarta.annotation.PreDestroy;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import java.math.BigDecimal;
import java.time.LocalDateTime;
import java.time.OffsetDateTime;
import java.time.format.DateTimeFormatter;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * MQTT 订阅服务 — 接收采集器上报数据，写入 MySQL。
 *
 * 订阅 Topic:
 *   - neuron/+/reading   # 实时读数（采集器主通道）
 *   - neuron/+/status    # 采集器心跳/状态
 *
 * 数据流:
 *   Collector → EMQX → MqttSubscriberService → dev_device_reading (MySQL)
 *
 * WHY:
 *   后端通过 MQTT 订阅采集器上报，而不是等前端轮询，
 *   实现真正的实时数据通路。
 */
@Slf4j
@Service
@ConditionalOnProperty(name = "app.mqtt.enabled", havingValue = "true", matchIfMissing = true)
public class MqttSubscriberService {

    @Value("${app.mqtt.broker-url:tcp://localhost:1883}")
    private String brokerUrl;

    @Value("${app.mqtt.client-id:neuron-server-sub}")
    private String clientId;

    @Value("${app.mqtt.username:}")
    private String username;

    @Value("${app.mqtt.password:}")
    private String password;

    @Value("${app.mqtt.keep-alive-seconds:60}")
    private int keepAlive;

    @Value("${app.mqtt.connection-timeout-seconds:30}")
    private int connTimeout;

    private volatile IMqttAsyncClient client;
    private final AtomicBoolean started = new AtomicBoolean(false);

    private final DevDeviceReadingMapper readingMapper;
    private final DevRegisterMapMapper registerMapMapper;
    private final DevCollectorMapper collectorMapper;
    private final ObjectMapper objectMapper;

    public MqttSubscriberService(DevDeviceReadingMapper readingMapper,
                                  DevRegisterMapMapper registerMapMapper,
                                  DevCollectorMapper collectorMapper) {
        this.readingMapper = readingMapper;
        this.registerMapMapper = registerMapMapper;
        this.collectorMapper = collectorMapper;
        this.objectMapper = new ObjectMapper();
    }

    /**
     * 注册中心缓存: sensor_code → DevRegisterMap
     */
    private final ConcurrentHashMap<String, DevRegisterMap> registerCache = new ConcurrentHashMap<>();

    @PostConstruct
    public void init() {
        start();
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
            log.warn("MQTT subscriber disconnect failed", e);
        }
    }

    synchronized void start() {
        if (!started.compareAndSet(false, true)) return;
        try {
            MqttConnectOptions options = new MqttConnectOptions();
            options.setAutomaticReconnect(true);
            options.setCleanSession(true);
            options.setConnectionTimeout(connTimeout);
            options.setKeepAliveInterval(keepAlive);
            if (!username.isBlank()) {
                options.setUserName(username);
                options.setPassword(password.toCharArray());
            }

            client = new MqttAsyncClient(brokerUrl, clientId);
            client.setCallback(new MqttCallbackExtended() {
                @Override public void connectComplete(boolean reconnect, String serverURI) {
                    log.info("MQTT subscriber connected to {}, reconnect={}", serverURI, reconnect);
                    subscribe();
                }
                @Override public void connectionLost(Throwable cause) {
                    log.warn("MQTT subscriber connection lost: {}", cause != null ? cause.getMessage() : "unknown");
                }
                @Override
                public void messageArrived(String topic, MqttMessage message) {
                    handleMessage(topic, message);
                }
                @Override public void deliveryComplete(IMqttDeliveryToken token) {}
            });

            client.connect(options).waitForCompletion(connTimeout * 1000L);
            log.info("MQTT subscriber connected to {}", brokerUrl);
            subscribe();
        } catch (MqttException e) {
            log.error("MQTT subscriber connect failed to {}: {}", brokerUrl, e.getMessage());
            client = null;
            started.set(false);
        }
    }

    private void subscribe() {
        if (client == null || !client.isConnected()) return;
        try {
            client.subscribe("neuron/+/reading", 1);
            client.subscribe("neuron/+/latest", 1);
            client.subscribe("neuron/+/status", 1);
            log.info("MQTT subscribed: neuron/+/reading, neuron/+/latest, neuron/+/status");
        } catch (MqttException e) {
            log.error("MQTT subscribe failed: {}", e.getMessage());
        }
    }

    /**
     * 处理收到的 MQTT 消息。
     *
     * 格式 (与 collector-uploader 保持一致):
     *   {
     *     "device_id": 1,
     *     "sensor_code": "demo",
     *     "value": 42.5,
     *     "unit": "units",
     *     "read_at": "2026-06-25T18:00:00+08:00"
     *   }
     */
    @SuppressWarnings("unchecked")
    private void handleMessage(String topic, MqttMessage message) {
        String payload = new String(message.getPayload(), java.nio.charset.StandardCharsets.UTF_8);
        log.debug("MQTT RECV topic={} payload={}", topic, payload);

        // 处理采集器状态消息
        if (topic.endsWith("/status")) {
            handleStatusMessage(topic, payload);
            return;
        }

        try {
            Map<String, Object> data = objectMapper.readValue(payload, Map.class);

            // 解析 device_id
            Object deviceIdRaw = data.get("device_id");
            if (deviceIdRaw == null) {
                log.warn("MQTT message missing device_id: {}", topic);
                return;
            }
            long deviceId = ((Number) deviceIdRaw).longValue();

            // 解析 sensor_code
            String sensorCode = (String) data.getOrDefault("sensor_code", "unknown");

            // 解析 value
            Object valueRaw = data.get("value");
            if (valueRaw == null) {
                log.warn("MQTT message missing value for device {} sensor {}", deviceId, sensorCode);
                return;
            }
            BigDecimal value = valueRaw instanceof Number
                ? BigDecimal.valueOf(((Number) valueRaw).doubleValue())
                : new BigDecimal(valueRaw.toString());

            // 解析时间
            String readAtStr = (String) data.getOrDefault("read_at", "");
            LocalDateTime readAt;
            try {
                readAt = OffsetDateTime.parse(readAtStr, DateTimeFormatter.ISO_OFFSET_DATE_TIME).toLocalDateTime();
            } catch (Exception e) {
                try {
                    readAt = LocalDateTime.parse(readAtStr);
                } catch (Exception ignored) {
                    readAt = LocalDateTime.now();
                }
            }

            // 查找 register_id
            Long registerId = getRegisterId(sensorCode);

            // 写入 dev_device_reading
            DevDeviceReading reading = new DevDeviceReading();
            reading.setDeviceId(deviceId);
            reading.setRegisterId(registerId != null ? registerId : 0L);
            reading.setSensorCode(sensorCode);
            reading.setValue(value);
            reading.setAvg(value);       // 单次上报，avg = value
            reading.setMax(value);
            reading.setMin(value);
            reading.setSampleCount(1);   // 实时通道每次 1 条
            reading.setQuality("0");       // 正常
            reading.setReadAt(readAt);

            readingMapper.insert(reading);
        } catch (Exception e) {
            log.error("MQTT message handle failed topic={}: {}", topic, e.getMessage());
        }
    }

    /**
     * 处理采集器状态消息。
     * Topic 格式: neuron/{mqttClientId}/status
     * Payload:   {"status":"online"}
     */
    private void handleStatusMessage(String topic, String payload) {
        try {
            String[] segments = topic.split("/");
            if (segments.length < 3) return;
            String mqttClientId = segments[1];

            Map<String, Object> data = objectMapper.readValue(payload, Map.class);
            String status = (String) data.getOrDefault("status", "online");

            DevCollector collector = collectorMapper.selectByMqttClientId(mqttClientId);
            if (collector != null) {
                collector.setStatus(status);
                collector.setLastHeartbeat(LocalDateTime.now());
                collectorMapper.updateById(collector);
                log.info("Collector {} status updated to {}", mqttClientId, status);
            } else {
                log.warn("Unknown collector mqttClientId: {}", mqttClientId);
            }
        } catch (Exception e) {
            log.error("Status message handle failed: {}", e.getMessage());
        }
    }

    /**
     * 根据 sensor_code 查找 register_id（带缓存）。
     */
    private Long getRegisterId(String sensorCode) {
        DevRegisterMap cached = registerCache.get(sensorCode);
        if (cached != null) return cached.getId();

        DevRegisterMap rm = registerMapMapper.selectBySensorCode(sensorCode);
        if (rm != null) {
            registerCache.put(sensorCode, rm);
            return rm.getId();
        }
        return null;
    }
}
