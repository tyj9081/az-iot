package com.aziot.dao.entity.device;

import com.baomidou.mybatisplus.annotation.FieldFill;
import com.baomidou.mybatisplus.annotation.IdType;
import com.baomidou.mybatisplus.annotation.TableField;
import com.baomidou.mybatisplus.annotation.TableId;
import com.baomidou.mybatisplus.annotation.TableName;
import lombok.Data;

import java.time.LocalDateTime;

@Data
@TableName("dev_device_instruction")
public class DevDeviceInstruction {

    @TableId(type = IdType.AUTO)
    private Long id;

    private Long deviceId;

    private String instructionCode;

    private String instructionName;

    /** READ / WRITE / CONTROL / CONFIG */
    private String instructionType;

    /** e.g. 0x03 = Read Holding Registers */
    private String funcCode;

    private Integer registerAddress;

    private Integer registerCount;

    /** JSON: extra params like dataType, byteOrder, coefficient, etc. */
    private String params;

    private Integer sortOrder;

    private String description;

    private Integer isEnabled;

    @TableField(fill = FieldFill.INSERT)
    private LocalDateTime createdAt;

    @TableField(fill = FieldFill.INSERT_UPDATE)
    private LocalDateTime updatedAt;
}
