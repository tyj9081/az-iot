package com.aziot.dao.entity.device;

import com.baomidou.mybatisplus.annotation.FieldFill;
import com.baomidou.mybatisplus.annotation.IdType;
import com.baomidou.mybatisplus.annotation.TableField;
import com.baomidou.mybatisplus.annotation.TableId;
import com.baomidou.mybatisplus.annotation.TableName;
import lombok.Data;

import java.time.LocalDateTime;

@Data
@TableName("dev_device_model")
public class DevDeviceModel {

    @TableId(type = IdType.AUTO)
    private Long id;

    private Long manufacturerId;

    private Long protocolId;

    private String code;

    private String name;

    private Integer collectIntervalSec;

    private String description;

    private Integer isEnabled;

    @TableField(fill = FieldFill.INSERT)
    private LocalDateTime createdAt;

    @TableField(fill = FieldFill.INSERT_UPDATE)
    private LocalDateTime updatedAt;
}
