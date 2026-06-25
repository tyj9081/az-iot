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
@TableName("dev_register_map")
public class DevRegisterMap {

    @TableId(type = IdType.AUTO)
    private Long id;

    private Long modelId;

    private String sensorCode;

    private String sensorName;

    private Integer registerAddress;

    private Integer registerCount;

    private String dataType;

    private String byteOrder;

    private String funcCode;

    private BigDecimal coefficient;

    private BigDecimal offsetVal;

    private String unit;

    private String rw;

    private Integer sortOrder;

    private String description;

    @TableField(fill = FieldFill.INSERT)
    private LocalDateTime createdAt;

    @TableField(fill = FieldFill.INSERT_UPDATE)
    private LocalDateTime updatedAt;
}
