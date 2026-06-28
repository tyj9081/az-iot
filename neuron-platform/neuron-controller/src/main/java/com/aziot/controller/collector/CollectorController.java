package com.aziot.controller.collector;

import com.aziot.common.dto.ApiResponse;
import com.aziot.common.dto.PageResult;
import com.aziot.controller.dto.CollectorCreateDTO;
import com.aziot.dao.entity.collector.DevCollector;
import com.aziot.dao.entity.device.DevDevice;
import com.aziot.dao.mapper.collector.DevCollectorMapper;
import com.aziot.dao.mapper.device.DevDeviceMapper;
import com.aziot.service.collector.DevCollectorService;
import com.aziot.service.mqtt.ConfigPushService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

import java.time.LocalDateTime;
import java.util.LinkedHashMap;
import java.util.Map;

@RestController
@RequestMapping("/api/v1/collectors")
@RequiredArgsConstructor
public class CollectorController {

    private final DevCollectorService collectorService;
    private final DevCollectorMapper collectorMapper;
    private final ConfigPushService configPushService;
    private final DevDeviceMapper deviceMapper;

    @GetMapping
    public ApiResponse<PageResult<DevCollector>> list(
            @RequestParam(defaultValue = "1") int page,
            @RequestParam(defaultValue = "20") int pageSize,
            @RequestParam(required = false) String status,
            @RequestParam(required = false) String keyword) {
        var result = collectorService.page(page, pageSize, status, keyword);
        return ApiResponse.ok(new PageResult<>(result.getRecords(), result.getTotal(), page, pageSize));
    }

    @GetMapping("/{id}")
    public ApiResponse<DevCollector> get(@PathVariable Long id) {
        return ApiResponse.ok(collectorService.getById(id));
    }

    @PostMapping
    @SuppressWarnings("unchecked")
    public ApiResponse<Map<String, Object>> create(@RequestBody CollectorCreateDTO dto) {
        DevCollector collector = new DevCollector();
        collector.setCode(dto.getCode());
        collector.setName(dto.getName());
        collector.setType(dto.getType());
        collector.setMqttClientId(dto.getMqttClientId());
        collector.setIpAddress(dto.getIpAddress());
        collector.setCollectIntervalSec(dto.getCollectIntervalSec());
        collector.setFirmwareVersion(dto.getFirmwareVersion());
        collector.setDescription(dto.getDescription());

        String rawPassword = collectorService.createWithCredentials(collector);

        Map<String, Object> result = new LinkedHashMap<>();
        result.put("collector", collector);
        Map<String, String> creds = new LinkedHashMap<>();
        creds.put("mqttUsername", collector.getMqttUsername());
        creds.put("mqttPassword", rawPassword);
        result.put("mqttCredentials", creds);
        return ApiResponse.ok(result);
    }

    @PutMapping("/{id}")
    public ApiResponse<DevCollector> update(@PathVariable Long id, @RequestBody CollectorCreateDTO dto) {
        DevCollector collector = new DevCollector();
        collector.setCode(dto.getCode());
        collector.setName(dto.getName());
        collector.setType(dto.getType());
        collector.setMqttClientId(dto.getMqttClientId());
        collector.setIpAddress(dto.getIpAddress());
        collector.setCollectIntervalSec(dto.getCollectIntervalSec());
        collector.setFirmwareVersion(dto.getFirmwareVersion());
        collector.setDescription(dto.getDescription());
        collectorService.update(id, collector);
        return ApiResponse.ok(collectorService.getById(id));
    }

    @DeleteMapping("/{id}")
    public ApiResponse<Void> delete(@PathVariable Long id) {
        collectorService.delete(id);
        return ApiResponse.ok();
    }

    /** 下发该采集器下所有设备的配置 */
    @PostMapping("/{id}/push-config")
    public ApiResponse<Map<String, Object>> pushConfig(@PathVariable Long id) {
        var devices = deviceMapper.selectByCollectorId(id);
        int count = 0;
        for (DevDevice device : devices) {
            configPushService.pushDelta(device.getId(), "add");
            count++;
        }
        return ApiResponse.ok(Map.of("collectorId", id, "deviceCount", count, "status", "pushed"));
    }

    /** 采集器 WS 启动上报 — 存在则更新状态, 不存在则创建 */
    @PostMapping("/register")
    public ApiResponse<Map<String, Object>> register(@RequestBody Map<String, String> body) {
        String mqttClientId = body.get("mqtt_client_id");
        if (mqttClientId == null || mqttClientId.isBlank()) {
            return ApiResponse.fail(400, "mqtt_client_id is required");
        }

        DevCollector collector = collectorMapper.selectByMqttClientId(mqttClientId);
        if (collector != null) {
            // 已存在: 更新在线状态
            collector.setStatus("online");
            collector.setLastHeartbeat(LocalDateTime.now());
            if (body.get("ip_address") != null) collector.setIpAddress(body.get("ip_address"));
            collectorMapper.updateById(collector);
            return ApiResponse.ok(Map.of("id", collector.getId(), "action", "updated", "status", "online"));
        }

        // 新建采集器记录
        collector = new DevCollector();
        collector.setCode(mqttClientId);
        collector.setName(mqttClientId);
        collector.setType("BC-U101");
        collector.setMqttClientId(mqttClientId);
        collector.setIpAddress(body.getOrDefault("ip_address", ""));
        collector.setStatus("online");
        collector.setCollectIntervalSec(5);
        collector.setLastHeartbeat(LocalDateTime.now());
        collectorService.createWithCredentials(collector);

        return ApiResponse.ok(Map.of("id", collector.getId(), "action", "created", "status", "online"));
    }
}
