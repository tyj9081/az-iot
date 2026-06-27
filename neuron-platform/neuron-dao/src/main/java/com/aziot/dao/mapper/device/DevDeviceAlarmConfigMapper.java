package com.aziot.dao.mapper.device;

import com.aziot.dao.entity.device.DevDeviceAlarmConfig;
import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import org.apache.ibatis.annotations.Mapper;
import org.apache.ibatis.annotations.Param;
import java.util.List;

@Mapper
public interface DevDeviceAlarmConfigMapper extends BaseMapper<DevDeviceAlarmConfig> {

    /** 按设备ID查询所有告警配置 */
    List<DevDeviceAlarmConfig> selectByDeviceId(@Param("deviceId") Long deviceId);

    /** 按设备ID+传感器编码+告警类型查询唯一配置 */
    DevDeviceAlarmConfig selectByDeviceSensorType(
        @Param("deviceId") Long deviceId,
        @Param("sensorCode") String sensorCode,
        @Param("alarmType") String alarmType
    );

    /** 删除指定设备+传感器的所有告警配置 */
    int deleteByDeviceAndSensor(
        @Param("deviceId") Long deviceId,
        @Param("sensorCode") String sensorCode
    );

    /** 统计启用的告警规则数 */
    long countEnabled();
}
