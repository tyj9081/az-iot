package com.aziot.service.mqtt;

import com.aziot.dao.entity.device.DevDeviceReading;
import com.aziot.dao.mapper.device.DevDeviceReadingMapper;
import com.aziot.dao.mapper.device.DevRegisterMapMapper;
import com.aziot.dao.mapper.device.DevDeviceMapper;
import com.aziot.dao.mapper.collector.DevCollectorMapper;
import com.aziot.dao.entity.collector.DevCollector;
import com.aziot.dao.entity.device.DevDevice;
import com.aziot.dao.entity.device.DevRegisterMap;
import com.aziot.service.cache.RealtimeCache;
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

    @Value("${app.mqtt.enabled:true}")
    private boolean mqttEnabled;

    private volatile IMqttAsyncClient client;
    private final AtomicBoolean started = new AtomicBoolean(false);

    private final DevDeviceReadingMapper readingMapper;
    private final DevRegisterMapMapper registerMapMapper;
    private final DevDeviceMapper deviceMapper;
    private final DevCollectorMapper collectorMapper;
    private final RealtimeCache realtimeCache;
    private final ObjectMapper objectMapper;

    /** 已显式订阅的采集器 mqttClientId（避免 wildcard ACL 问题） */
    private final Map<String, Boolean> subscribedCollectors = new ConcurrentHashMap<>();

    /** 设备在线状态缓存(减少 DB 查询): deviceId → 最后在线时间戳 */
    private final Map<Long, Long> deviceOnlineCache = new ConcurrentHashMap<>();

    public MqttSubscriberService(DevDeviceReadingMapper readingMapper,
                                  DevRegisterMapMapper registerMapMapper,
                                  DevDeviceMapper deviceMapper,
                                  DevCollectorMapper collectorMapper,
                                  RealtimeCache realtimeCache) {
        this.readingMapper = readingMapper;
        this.registerMapMapper = registerMapMapper;
        this.deviceMapper = deviceMapper;
        this.collectorMapper = collectorMapper;
        this.realtimeCache = realtimeCache;
        this.objectMapper = new ObjectMapper();
    }

    /**
     * 注册中心缓存: sensor_code → DevRegisterMap
     */
    private final ConcurrentHashMap<String, DevRegisterMap> registerCache = new ConcurrentHashMap<>();

    @PostConstruct
    public void init() {
        if (!mqttEnabled) {
            log.info("MQTT subscriber disabled by configuration");
            return;
        }
        // Start MQTT connection asynchronously to avoid blocking Spring startup
        new Thread(this::start, "mqtt-subscriber-init").start();
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
            // 仅用一个窄 wildcard 感知新采集器上线（status 消息触发显式订阅）
            client.subscribe("neuron/+/status", 1);
            log.info("MQTT subscribed: neuron/+/status (wildcard for new collector detection)");

            // 从 DB 加载已有采集器，逐个显式订阅（避免 wildcard ACL 问题）
            var collectors = collectorMapper.selectList(null);
            for (var collector : collectors) {
                subscribeCollector(collector.getMqttClientId());
            }
        } catch (Exception e) {
            log.error("MQTT subscribe failed: {}", e.getMessage());
        }
    }

    /**
     * 为指定采集器显式订阅 3 个 topic（neuron/{clientId}/latest, /reading, /status）
     * 去重：同一 clientId 只订阅一次。
     */
    private void subscribeCollector(String mqttClientId) {
        if (mqttClientId == null || mqttClientId.isBlank()) return;
        if (client == null || !client.isConnected()) return;
        if (subscribedCollectors.putIfAbsent(mqttClientId, Boolean.TRUE) != null) {
            return; // 已订阅过
        }
        try {
            client.subscribe("neuron/" + mqttClientId + "/latest", 1);
            client.subscribe("neuron/" + mqttClientId + "/reading", 1);
            client.subscribe("neuron/" + mqttClientId + "/status", 1);
            log.info("MQTT subscribed collector: {}", mqttClientId);
        } catch (MqttException e) {
            log.error("MQTT subscribe failed for {}: {}", mqttClientId, e.getMessage());
            subscribedCollectors.remove(mqttClientId); // 允许下次重试
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
            double value = valueRaw instanceof Number
                ? ((Number) valueRaw).doubleValue()
                : Double.parseDouble(valueRaw.toString());

            // 解析时间
            String readAtStr = (String) data.getOrDefault("read_at", "");
            long timestamp;
            try {
                timestamp = OffsetDateTime.parse(readAtStr, DateTimeFormatter.ISO_OFFSET_DATE_TIME)
                    .toInstant().toEpochMilli();
            } catch (Exception e) {
                try {
                    timestamp = LocalDateTime.parse(readAtStr)
                        .atZone(java.time.ZoneId.systemDefault()).toInstant().toEpochMilli();
                } catch (Exception ignored) {
                    timestamp = System.currentTimeMillis();
                }
            }

            // 实时通道(neuron/+/latest): 写入内存缓存 + 落库
            if (topic.endsWith("/latest")) {
                handleLatestReading(deviceId, sensorCode, value, data, timestamp);
                markDeviceOnline(deviceId);
                // 不 return，继续走下面的 MySQL 落库逻辑
            }

            // 聚合通道(neuron/+/reading): 仍写入 MySQL
            Long registerId = getRegisterId(sensorCode);

            DevDeviceReading reading = new DevDeviceReading();
            reading.setDeviceId(deviceId);
            reading.setRegisterId(registerId != null ? registerId : 0L);
            reading.setSensorCode(sensorCode);
            reading.setValue(BigDecimal.valueOf(value));
            reading.setAvg(BigDecimal.valueOf(value));       // 单次上报，avg = value
            reading.setMax(BigDecimal.valueOf(value));
            reading.setMin(BigDecimal.valueOf(value));
            reading.setSampleCount(1);   // 实时通道每次 1 条
            reading.setQuality(0);       // 正常
            reading.setReadAt(LocalDateTime.now());

            readingMapper.insert(reading);

            markDeviceOnline(deviceId);
        } catch (Exception e) {
            log.error("MQTT message handle failed topic={}: {}", topic, e.getMessage());
            // Retry once after 100ms
            try {
                Thread.sleep(100);
                handleMessage(topic, message);
            } catch (Exception retryEx) {
                log.error("MQTT message retry also failed topic={}, payload will be dropped. Error: {}", 
                    topic, retryEx.getMessage());
                // TODO: implement dead letter queue for critical message recovery
            }
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

            // 首次见到此采集器 → 显式订阅（绕过 wildcard ACL 限制）
            subscribeCollector(mqttClientId);

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
     * 实时读数处理 — 写入内存缓存，不走 MySQL。
     * 双通道架构：通道A（实时）→ 内存缓存 → WebSocket 推送前端。
     */
    private void handleLatestReading(long deviceId, String sensorCode, double value,
                                     Map<String, Object> data, long timestamp) {
        String unit = (String) data.getOrDefault("unit", "");
        realtimeCache.put(deviceId, sensorCode, value, unit, timestamp);
        log.debug("RealtimeCache updated: deviceId={} sensorCode={} value={}", deviceId, sensorCode, value);
    }

    /**
     * 标记设备在线（缓存 10s 过期，减少 DB 查询）。
     */
    private void markDeviceOnline(long deviceId) {
        Long now = System.currentTimeMillis();
        Long cached = deviceOnlineCache.get(deviceId);
        if (cached == null || (now - cached) > 10_000) {
            deviceOnlineCache.put(deviceId, now);
            DevDevice device = deviceMapper.selectById(deviceId);
            if (device != null && !"online".equals(device.getStatus())) {
                device.setStatus("online");
                deviceMapper.updateById(device);
            }
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
