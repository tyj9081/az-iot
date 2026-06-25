package com.aziot.controller.collector;

import com.aziot.common.dto.ApiResponse;
import com.aziot.common.dto.PageResult;
import com.aziot.controller.dto.CollectorCreateDTO;
import com.aziot.dao.entity.collector.DevCollector;
import com.aziot.service.collector.DevCollectorService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

import java.util.LinkedHashMap;
import java.util.Map;

@RestController
@RequestMapping("/api/v1/collectors")
@RequiredArgsConstructor
public class CollectorController {

    private final DevCollectorService collectorService;

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
}
