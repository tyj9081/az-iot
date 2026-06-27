package com.aziot.service.device;

import com.aziot.dao.entity.device.DevDeviceReading;
import com.aziot.dao.entity.device.DevDevice;
import com.aziot.dao.entity.device.DevDeviceAlarmConfig;
import com.aziot.dao.mapper.device.DevDeviceAlarmConfigMapper;
import com.aziot.dao.mapper.device.DevDeviceMapper;
import com.aziot.dao.mapper.device.DevDeviceReadingMapper;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;
import org.springframework.stereotype.Service;
import java.time.LocalDateTime;
import java.util.*;

@Service
public class DevDeviceReadingService extends ServiceImpl<DevDeviceReadingMapper, DevDeviceReading> {

    private final DevDeviceMapper deviceMapper;
    private final DevDeviceAlarmConfigMapper alarmConfigMapper;

    public DevDeviceReadingService(DevDeviceMapper deviceMapper, DevDeviceAlarmConfigMapper alarmConfigMapper) {
        this.deviceMapper = deviceMapper;
        this.alarmConfigMapper = alarmConfigMapper;
    }

    // Latest readings for a device (all sensor codes, most recent only)
    public List<DevDeviceReading> latestByDeviceId(Long deviceId) {
        // Simple approach: get all readings for this device in the last hour, group by sensor_code
        LambdaQueryWrapper<DevDeviceReading> qw = new LambdaQueryWrapper<>();
        qw.eq(DevDeviceReading::getDeviceId, deviceId)
          .ge(DevDeviceReading::getReadAt, LocalDateTime.now().minusHours(1))
          .orderByDesc(DevDeviceReading::getReadAt);
        List<DevDeviceReading> all = list(qw);
        
        // Group by sensor_code, keep only the most recent
        Map<String, DevDeviceReading> latest = new LinkedHashMap<>();
        for (DevDeviceReading r : all) {
            latest.putIfAbsent(r.getSensorCode(), r);
        }
        return new ArrayList<>(latest.values());
    }

    // History for a device, specific sensor_code within time range
    public List<DevDeviceReading> history(Long deviceId, String sensorCode, LocalDateTime from, LocalDateTime to) {
        LambdaQueryWrapper<DevDeviceReading> qw = new LambdaQueryWrapper<>();
        qw.eq(DevDeviceReading::getDeviceId, deviceId)
          .eq(sensorCode != null, DevDeviceReading::getSensorCode, sensorCode)
          .ge(DevDeviceReading::getReadAt, from)
          .le(DevDeviceReading::getReadAt, to)
          .orderByAsc(DevDeviceReading::getReadAt);
        return list(qw);
    }

    // Dashboard stats
    public Map<String, Object> dashboardOverview() {
        Map<String, Object> stats = new LinkedHashMap<>();
        stats.put("totalDevices", deviceMapper.selectCount(null));
        stats.put("todayReadings", baseMapper.selectCount(
            new LambdaQueryWrapper<DevDeviceReading>()
                .ge(DevDeviceReading::getReadAt, LocalDateTime.now().withHour(0).withMinute(0).withSecond(0))));
        stats.put("onlineDevices", deviceMapper.selectCount(
            new LambdaQueryWrapper<DevDevice>()
                .eq(DevDevice::getStatus, "online")));
        stats.put("alarms", deviceMapper.selectCount(
            new LambdaQueryWrapper<DevDevice>()
                .eq(DevDevice::getStatus, "alarm")));
        stats.put("alarmRules", alarmConfigMapper.selectCount(
            new LambdaQueryWrapper<DevDeviceAlarmConfig>()
                .eq(DevDeviceAlarmConfig::getAlarmEnabled, 1)));
        stats.put("recentReadings", list(new LambdaQueryWrapper<DevDeviceReading>()
            .orderByDesc(DevDeviceReading::getReadAt)
            .last("LIMIT 20")));
        return stats;
    }
}
