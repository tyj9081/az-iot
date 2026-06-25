package com.aziot.dao.entity.device;

import com.baomidou.mybatisplus.annotation.FieldFill;
import com.baomidou.mybatisplus.annotation.IdType;
import com.baomidou.mybatisplus.annotation.TableField;
import com.baomidou.mybatisplus.annotation.TableId;
import com.baomidou.mybatisplus.annotation.TableLogic;
import com.baomidou.mybatisplus.annotation.TableName;
import lombok.Data;

import java.time.LocalDateTime;

@Data
@TableName("dev_device")
public class DevDevice {

    @TableId(type = IdType.AUTO)
    private Long id;

    private Long serialPortId;

    private Long modelId;

    private String code;

    private String name;

    private Integer slaveAddr;

    private Integer collectIntervalSec;

    private String status;

    private String location;

    private String description;

    @TableLogic
    private Integer isDeleted;

    @TableField(fill = FieldFill.INSERT)
    private LocalDateTime createdAt;

    @TableField(fill = FieldFill.INSERT_UPDATE)
    private LocalDateTime updatedAt;
}
