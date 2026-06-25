package com.aziot.service.system;

import com.aziot.dao.entity.system.SysPermission;
import com.aziot.dao.mapper.system.SysPermissionMapper;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;
import org.springframework.stereotype.Service;

import java.util.*;
import java.util.stream.Collectors;

@Service
public class SysPermissionService extends ServiceImpl<SysPermissionMapper, SysPermission> {

    public List<SysPermission> tree() {
        List<SysPermission> all = list(new LambdaQueryWrapper<SysPermission>()
                .eq(SysPermission::getStatus, "1").orderByAsc(SysPermission::getSortOrder));
        Map<Long, List<SysPermission>> childrenMap = all.stream()
                .filter(p -> p.getParentId() != 0)
                .collect(Collectors.groupingBy(SysPermission::getParentId));
        return all.stream().filter(p -> p.getParentId() == 0)
                .peek(p -> p.setChildren(childrenMap.getOrDefault(p.getId(), Collections.emptyList())))
                .collect(Collectors.toList());
    }
}
