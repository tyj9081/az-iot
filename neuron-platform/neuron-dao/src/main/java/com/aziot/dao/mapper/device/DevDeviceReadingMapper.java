package com.aziot.dao.mapper.device;

import com.aziot.dao.entity.device.DevDeviceReading;
import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import org.apache.ibatis.annotations.Mapper;
import org.apache.ibatis.annotations.Param;

import java.time.LocalDateTime;
import java.util.List;

@Mapper
public interface DevDeviceReadingMapper extends BaseMapper<DevDeviceReading> {

    /** 查询设备最近1小时读数 */
    List<DevDeviceReading> selectLatestByDeviceId(
        @Param("deviceId") Long deviceId,
        @Param("sinceTime") LocalDateTime sinceTime
    );

    /** 查询设备历史读数 */
    List<DevDeviceReading> selectHistory(
        @Param("deviceId") Long deviceId,
        @Param("sensorCode") String sensorCode,
        @Param("fromTime") LocalDateTime fromTime,
        @Param("toTime") LocalDateTime toTime
    );

    /** 统计今日读数总数 */
    long countTodayReadings(@Param("todayStart") LocalDateTime todayStart);

    /** 查询最近20条读数 */
    List<DevDeviceReading> selectRecent20();
}
