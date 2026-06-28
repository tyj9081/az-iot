package com.aziot.controller.device;

import com.aziot.common.dto.ApiResponse;
import com.aziot.common.dto.DevDeviceVO;
import com.aziot.common.dto.PageResult;
import com.aziot.controller.dto.DeviceCreateDTO;
import com.aziot.controller.dto.DeviceUpdateDTO;
import com.aziot.dao.entity.device.DevDevice;
import com.aziot.service.device.DevDeviceService;
import com.aziot.service.mqtt.ConfigPushService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

import java.util.Map;

@RestController
@RequestMapping("/api/v1/devices")
@RequiredArgsConstructor
public class DeviceController {

    private final DevDeviceService deviceService;
    private final ConfigPushService configPushService;

    @GetMapping
    public ApiResponse<PageResult<DevDeviceVO>> list(
            @RequestParam(defaultValue = "1") int page,
            @RequestParam(defaultValue = "20") int pageSize,
            @RequestParam(required = false) Long collectorId,
            @RequestParam(required = false) Long serialPortId,
            @RequestParam(required = false) Long modelId,
            @RequestParam(required = false) String status,
            @RequestParam(required = false) String keyword) {
        var result = deviceService.pageWithDetails(page, pageSize, collectorId, serialPortId, modelId, status, keyword);
        return ApiResponse.ok(new PageResult<>(result.getRecords(), result.getTotal(), page, pageSize));
    }

    @GetMapping("/{id}")
    public ApiResponse<DevDeviceVO> get(@PathVariable Long id) {
        return ApiResponse.ok(deviceService.getById(id));
    }

    @PostMapping
    public ApiResponse<DevDeviceVO> create(@RequestBody DeviceCreateDTO dto) {
        DevDevice device = new DevDevice();
        device.setSerialPortId(dto.getSerialPortId());
        device.setModelId(dto.getModelId());
        device.setCode(dto.getCode());
        device.setName(dto.getName());
        device.setSlaveAddr(dto.getSlaveAddr());
        device.setCollectIntervalSec(dto.getCollectIntervalSec());
        device.setLocation(dto.getLocation());
        device.setDescription(dto.getDescription());
        deviceService.create(device);
        // 返回 VO 而非 Entity, 符合 S03 规范
        return ApiResponse.ok(deviceService.getById(device.getId()));
    }

    @PutMapping("/{id}")
    public ApiResponse<DevDeviceVO> update(@PathVariable Long id, @RequestBody DeviceUpdateDTO dto) {
        DevDevice device = new DevDevice();
        device.setSerialPortId(dto.getSerialPortId());
        device.setModelId(dto.getModelId());
        device.setCode(dto.getCode());
        device.setName(dto.getName());
        device.setSlaveAddr(dto.getSlaveAddr());
        device.setCollectIntervalSec(dto.getCollectIntervalSec());
        device.setLocation(dto.getLocation());
        device.setDescription(dto.getDescription());
        deviceService.update(id, device);
        return ApiResponse.ok(deviceService.getById(id));
    }

    @DeleteMapping("/{id}")
    public ApiResponse<Void> delete(@PathVariable Long id) {
        deviceService.delete(id);
        return ApiResponse.ok();
    }

    @PutMapping("/{id}/status")
    public ApiResponse<DevDeviceVO> updateStatus(@PathVariable Long id, @RequestBody Map<String, String> body) {
        String status = body.get("status");
        deviceService.updateStatus(id, status);
        return ApiResponse.ok(deviceService.getById(id));
    }

    /** 手动触发设备配置下发 */
    @PostMapping("/{id}/push-config")
    public ApiResponse<Map<String, Object>> pushConfig(@PathVariable Long id) {
        configPushService.pushDelta(id, "add");
        return ApiResponse.ok(Map.of("deviceId", id, "action", "add", "status", "pushed"));
    }
}
