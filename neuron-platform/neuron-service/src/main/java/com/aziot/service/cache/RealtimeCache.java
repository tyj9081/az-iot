package com.aziot.service.cache;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

/**
 * 实时数据内存缓存 — 替代直接写入 MySQL，实现双通道架构的通道A（不落库）。
 *
 * 数据流:
 *   Collector → EMQX → MqttSubscriberService → RealtimeCache (内存)
 *                                                ↓
 *                                   RealtimeWebSocketHandler → 前端推送
 */
@Slf4j
@Component
public class RealtimeCache {

    /** device_id -> sensor_code -> RealtimeValue */
    // TODO: add scheduled eviction for entries older than 30 minutes to prevent OOM on long-running instances
    private final ConcurrentHashMap<Long, ConcurrentHashMap<String, RealtimeValue>> cache = new ConcurrentHashMap<>();

    /**
     * 存入一条实时读数。
     */
    public void put(long deviceId, String sensorCode, double value, String unit, long timestamp) {
        cache.computeIfAbsent(deviceId, k -> new ConcurrentHashMap<>())
             .put(sensorCode, new RealtimeValue(value, unit, timestamp));
    }

    /**
     * 获取指定设备的所有传感器最新值。
     */
    public Map<String, RealtimeValue> getByDevice(Long deviceId) {
        Map<String, RealtimeValue> sensorMap = cache.get(deviceId);
        return sensorMap != null ? Collections.unmodifiableMap(sensorMap) : Collections.emptyMap();
    }

    /**
     * 获取指定设备的指定传感器最新值。
     */
    public RealtimeValue get(Long deviceId, String sensorCode) {
        Map<String, RealtimeValue> sensorMap = cache.get(deviceId);
        return sensorMap != null ? sensorMap.get(sensorCode) : null;
    }

    /**
     * 获取所有有缓存数据的活跃设备 ID 列表。
     */
    public List<Long> getActiveDeviceIds() {
        return new ArrayList<>(cache.keySet());
    }

    /**
     * Remove cached data for a device (call when device is deleted).
     */
    public void removeDevice(Long deviceId) {
        cache.remove(deviceId);
    }

    /**
     * Clear all cached data.
     */
    public void clear() {
        cache.clear();
    }

    /**
     * Get current cache size for monitoring.
     */
    public int size() {
        return cache.size();
    }

    @Data
    @AllArgsConstructor
    public static class RealtimeValue {
        double value;
        String unit;
        long timestamp;
    }
}
