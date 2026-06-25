package com.aziot.service.system;

import com.aziot.dao.entity.system.SysRole;
import com.aziot.dao.mapper.system.SysRoleMapper;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;
import org.springframework.stereotype.Service;

import java.util.List;

@Service
public class SysRoleService extends ServiceImpl<SysRoleMapper, SysRole> {

    public List<SysRole> listAll() {
        return list(new LambdaQueryWrapper<SysRole>().orderByAsc(SysRole::getSortOrder));
    }
}
