package com.aziot.dao.entity.system;

import com.baomidou.mybatisplus.annotation.FieldFill;
import com.baomidou.mybatisplus.annotation.IdType;
import com.baomidou.mybatisplus.annotation.TableField;
import com.baomidou.mybatisplus.annotation.TableId;
import com.baomidou.mybatisplus.annotation.TableName;
import lombok.Data;

import java.time.LocalDateTime;

@Data
@TableName("sys_audit_log")
public class SysAuditLog {

    @TableId(type = IdType.AUTO)
    private Long id;

    private String module;

    private String action;

    private Long operatorId;

    private String operatorName;

    private String requestIp;

    private String requestMethod;

    private String requestUrl;

    private String requestParams;

    private String responseResult;

    private Integer costMs;

    private String status;

    @TableField(fill = FieldFill.INSERT)
    private LocalDateTime createdAt;
}
