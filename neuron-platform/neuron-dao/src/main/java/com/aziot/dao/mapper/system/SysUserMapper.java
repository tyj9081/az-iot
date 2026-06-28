package com.aziot.dao.mapper.system;

import com.aziot.dao.entity.system.SysUser;
import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import org.apache.ibatis.annotations.Mapper;
import org.apache.ibatis.annotations.Param;

import java.util.List;

@Mapper
public interface SysUserMapper extends BaseMapper<SysUser> {

    /** 按用户名精确查询 */
    SysUser selectByUsername(@Param("username") String username);

    /** 分页条件查询 */
    Page<SysUser> selectPageByCondition(
        Page<SysUser> page,
        @Param("keyword") String keyword,
        @Param("status") Integer status
    );

    /** 按角色编码查询用户列表 */
    List<SysUser> selectByRoleCode(@Param("roleCode") String roleCode);

    /** 按状态统计 */
    int countByStatus(@Param("status") Integer status);
}
