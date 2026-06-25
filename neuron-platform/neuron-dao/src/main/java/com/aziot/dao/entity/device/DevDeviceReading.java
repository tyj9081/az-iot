package com.aziot.dao.entity.device;

import com.baomidou.mybatisplus.annotation.FieldFill;
import com.baomidou.mybatisplus.annotation.IdType;
import com.baomidou.mybatisplus.annotation.TableField;
import com.baomidou.mybatisplus.annotation.TableId;
import com.baomidou.mybatisplus.annotation.TableName;
import lombok.Data;

import java.math.BigDecimal;
import java.time.LocalDateTime;

@Data
@TableName("dev_device_reading")
public class DevDeviceReading {

    @TableId(type = IdType.AUTO)
    private Long id;

    private Long deviceId;

    private Long registerId;

    private String sensorCode;

    private BigDecimal value;

    private BigDecimal avg;

    private BigDecimal max;

    private BigDecimal min;

    private Integer sampleCount;

    private String quality;

    private LocalDateTime readAt;

    private LocalDateTime windowStart;

    private LocalDateTime windowEnd;

    @TableField(fill = FieldFill.INSERT)
    private LocalDateTime createdAt;
}
