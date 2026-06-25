package com.aziot.dao.entity.device;

import com.baomidou.mybatisplus.annotation.*;
import lombok.Data;
import java.math.BigDecimal;
import java.time.LocalDateTime;

@Data
@TableName("dev_device_alarm_config")
public class DevDeviceAlarmConfig {
    @TableId(type = IdType.AUTO)
    private Long id;
    private Long deviceId;
    private String sensorCode;
    private Integer alarmEnabled;
    private BigDecimal minValue;
    private BigDecimal maxValue;
    private BigDecimal hysteresis;
    private Integer delayCount;
    private String alarmLevel;
    private String description;
    @TableField(fill = FieldFill.INSERT)
    private LocalDateTime createdAt;
    @TableField(fill = FieldFill.INSERT_UPDATE)
    private LocalDateTime updatedAt;
}
