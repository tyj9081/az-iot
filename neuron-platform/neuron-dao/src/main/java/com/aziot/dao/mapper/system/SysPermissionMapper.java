package com.aziot.dao.mapper.system;

import com.aziot.dao.entity.system.SysPermission;
import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import org.apache.ibatis.annotations.Mapper;

import java.util.List;

@Mapper
public interface SysPermissionMapper extends BaseMapper<SysPermission> {

    /** 查询所有启用权限，按sort_order升序 */
    List<SysPermission> selectAllEnabled();
}
