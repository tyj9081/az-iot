package com.aziot.dao.mapper.system;

import com.aziot.dao.entity.system.SysRole;
import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import org.apache.ibatis.annotations.Mapper;

import java.util.List;

@Mapper
public interface SysRoleMapper extends BaseMapper<SysRole> {

    /** 查询所有启用角色，按sort_order升序 */
    List<SysRole> selectAllOrdered();
}
