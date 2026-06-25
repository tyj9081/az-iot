package com.aziot.dao.entity.collector;

import com.baomidou.mybatisplus.annotation.FieldFill;
import com.baomidou.mybatisplus.annotation.IdType;
import com.baomidou.mybatisplus.annotation.TableField;
import com.baomidou.mybatisplus.annotation.TableId;
import com.baomidou.mybatisplus.annotation.TableName;
import lombok.Data;

import java.time.LocalDateTime;

@Data
@TableName("dev_serial_port")
public class DevSerialPort {

    @TableId(type = IdType.AUTO)
    private Long id;

    private Long collectorId;

    private String portName;

    private String portLabel;

    private String busType;

    private String busParam;

    private String portType;

    private Integer isActive;

    @TableField(fill = FieldFill.INSERT)
    private LocalDateTime createdAt;

    @TableField(fill = FieldFill.INSERT_UPDATE)
    private LocalDateTime updatedAt;
}
