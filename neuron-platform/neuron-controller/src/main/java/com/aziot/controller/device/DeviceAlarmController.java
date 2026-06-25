package com.aziot.controller.device;

import com.aziot.common.dto.ApiResponse;
import com.aziot.dao.entity.device.DevDeviceAlarmConfig;
import com.aziot.service.device.DevDeviceAlarmConfigService;
import com.aziot.service.mqtt.ConfigPushService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;
import java.util.List;

@RestController
@RequestMapping("/api/v1/devices")
@RequiredArgsConstructor
public class DeviceAlarmController {

    private final DevDeviceAlarmConfigService alarmConfigService;
    private final ConfigPushService configPushService;

    @GetMapping("/{deviceId}/alarm-config")
    public ApiResponse<List<DevDeviceAlarmConfig>> list(@PathVariable Long deviceId) {
        return ApiResponse.ok(alarmConfigService.listByDeviceId(deviceId));
    }

    @PutMapping("/{deviceId}/alarm-config/{alarmType}/{sensorCode}")
    public ApiResponse<DevDeviceAlarmConfig> save(
            @PathVariable Long deviceId,
            @PathVariable String alarmType,
            @PathVariable String sensorCode,
            @RequestBody DevDeviceAlarmConfig config) {
        config.setAlarmType(alarmType);
        alarmConfigService.saveOrUpdateAlarm(deviceId, sensorCode, config);
        configPushService.pushDelta(deviceId, "update");
        return ApiResponse.ok(config);
    }

    @DeleteMapping("/{deviceId}/alarm-config/{alarmType}/{sensorCode}")
    public ApiResponse<Void> delete(
            @PathVariable Long deviceId,
            @PathVariable String alarmType,
            @PathVariable String sensorCode) {
        alarmConfigService.deleteByDeviceAndSensor(deviceId, sensorCode);
        configPushService.pushDelta(deviceId, "update");
        return ApiResponse.ok();
    }
}
