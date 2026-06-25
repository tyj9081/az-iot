package com.aziot.service.device;

import com.aziot.dao.entity.device.DevDeviceAlarmConfig;
import com.aziot.dao.mapper.device.DevDeviceAlarmConfigMapper;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;
import java.util.List;

@Service
public class DevDeviceAlarmConfigService extends ServiceImpl<DevDeviceAlarmConfigMapper, DevDeviceAlarmConfig> {

    public List<DevDeviceAlarmConfig> listByDeviceId(Long deviceId) {
        return list(new LambdaQueryWrapper<DevDeviceAlarmConfig>()
                .eq(DevDeviceAlarmConfig::getDeviceId, deviceId));
    }

    @Transactional
    public void saveOrUpdateAlarm(Long deviceId, String sensorCode, DevDeviceAlarmConfig config) {
        config.setDeviceId(deviceId);
        config.setSensorCode(sensorCode);
        DevDeviceAlarmConfig existing = getOne(new LambdaQueryWrapper<DevDeviceAlarmConfig>()
                .eq(DevDeviceAlarmConfig::getDeviceId, deviceId)
                .eq(DevDeviceAlarmConfig::getSensorCode, sensorCode)
                .eq(DevDeviceAlarmConfig::getAlarmType, config.getAlarmType()));
        if (existing != null) {
            config.setId(existing.getId());
            updateById(config);
        } else {
            save(config);
        }
    }

    @Transactional
    public void deleteByDeviceAndSensor(Long deviceId, String sensorCode) {
        remove(new LambdaQueryWrapper<DevDeviceAlarmConfig>()
                .eq(DevDeviceAlarmConfig::getDeviceId, deviceId)
                .eq(DevDeviceAlarmConfig::getSensorCode, sensorCode));
    }
}
