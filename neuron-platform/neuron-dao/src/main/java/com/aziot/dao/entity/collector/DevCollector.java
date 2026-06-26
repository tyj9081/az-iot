package com.aziot.dao.entity.collector;

import com.baomidou.mybatisplus.annotation.FieldFill;
import com.baomidou.mybatisplus.annotation.IdType;
import com.baomidou.mybatisplus.annotation.TableField;
import com.baomidou.mybatisplus.annotation.TableId;
import com.baomidou.mybatisplus.annotation.TableName;
import lombok.Data;

import java.time.LocalDateTime;

@Data
@TableName("dev_collector")
public class DevCollector {

    @TableId(type = IdType.AUTO)
    private Long id;

    private String code;

    private String name;

    private String type;

    private String mqttClientId;

    private String ipAddress;

    private Integer collectIntervalSec;

    private String status;

    private String firmwareVersion;

    private LocalDateTime lastHeartbeat;

    private String description;

    private String mqttUsername;

    private String mqttPasswordHash;

    private Integer mqttTlsEnabled;

    private String mqttBrokerHost;

    private Integer mqttBrokerPort;

    @TableField(fill = FieldFill.INSERT)
    private LocalDateTime createdAt;

    @TableField(fill = FieldFill.INSERT_UPDATE)
    private LocalDateTime updatedAt;
}
