package com.aziot.dao.entity.system;

import com.baomidou.mybatisplus.annotation.FieldFill;
import com.baomidou.mybatisplus.annotation.IdType;
import com.baomidou.mybatisplus.annotation.TableField;
import com.baomidou.mybatisplus.annotation.TableId;
import com.baomidou.mybatisplus.annotation.TableName;
import lombok.Data;

import java.time.LocalDateTime;
import java.util.List;

@Data
@TableName("sys_permission")
public class SysPermission {

    @TableId(type = IdType.AUTO)
    private Long id;

    private Long parentId;

    private String permCode;

    private String permName;

    private String permType;

    private String path;

    private String icon;

    private Integer sortOrder;

    private String status;

    @TableField(fill = FieldFill.INSERT)
    private LocalDateTime createdAt;

    @TableField(fill = FieldFill.INSERT_UPDATE)
    private LocalDateTime updatedAt;

    @TableField(exist = false)
    private List<SysPermission> children;
}
