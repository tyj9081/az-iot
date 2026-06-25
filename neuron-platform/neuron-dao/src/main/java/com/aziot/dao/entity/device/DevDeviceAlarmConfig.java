package com.aziot.dao.entity.device;

import com.baomidou.mybatisplus.annotation.*;
import lombok.Data;
import java.time.LocalDateTime;

@Data
@TableName("dev_device_alarm_config")
public class DevDeviceAlarmConfig {
    @TableId(type = IdType.AUTO)
    private Long id;
    private Long deviceId;
    private String sensorCode;
    private Integer alarmEnabled;
    private String alarmType;       // limit_upper, limit_lower, limit_both, rate_rise, rate_fall, deviation, di_change, timeout, deadband, custom
    private String params;          // JSON string
    private String alarmLevel;      // info, warning, critical
    private String description;
    @TableField(fill = FieldFill.INSERT)
    private LocalDateTime createdAt;
    @TableField(fill = FieldFill.INSERT_UPDATE)
    private LocalDateTime updatedAt;
}
