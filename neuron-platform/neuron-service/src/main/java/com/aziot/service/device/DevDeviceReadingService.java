package com.aziot.service.device;

import com.aziot.dao.entity.device.DevDeviceReading;
import com.aziot.dao.mapper.device.DevDeviceAlarmConfigMapper;
import com.aziot.dao.mapper.device.DevDeviceMapper;
import com.aziot.dao.mapper.device.DevDeviceReadingMapper;
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

    public List<DevDeviceReading> latestByDeviceId(Long deviceId) {
        List<DevDeviceReading> all = baseMapper.selectLatestByDeviceId(deviceId, LocalDateTime.now().minusHours(1));
        Map<String, DevDeviceReading> latest = new LinkedHashMap<>();
        for (DevDeviceReading r : all) {
            latest.putIfAbsent(r.getSensorCode(), r);
        }
        return new ArrayList<>(latest.values());
    }

    public List<DevDeviceReading> history(Long deviceId, String sensorCode, LocalDateTime from, LocalDateTime to) {
        return baseMapper.selectHistory(deviceId, sensorCode, from, to);
    }

    public Map<String, Object> dashboardOverview() {
        Map<String, Object> stats = new LinkedHashMap<>();
        stats.put("totalDevices", deviceMapper.selectCount(null));
        stats.put("todayReadings", baseMapper.countTodayReadings(
            LocalDateTime.now().withHour(0).withMinute(0).withSecond(0)));
        stats.put("onlineDevices", deviceMapper.countByStatus("online"));
        stats.put("alarms", deviceMapper.countByStatus("alarm"));
        stats.put("alarmRules", alarmConfigMapper.countEnabled());
        stats.put("recentReadings", baseMapper.selectRecent20());
        return stats;
    }
}
