package com.aziot.service.system;

import com.aziot.dao.entity.system.SysUser;
import com.aziot.dao.mapper.system.SysUserMapper;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

@Service
public class SysUserService extends ServiceImpl<SysUserMapper, SysUser> {

    private final PasswordEncoder passwordEncoder;

    public SysUserService(PasswordEncoder passwordEncoder) {
        this.passwordEncoder = passwordEncoder;
    }

    public Page<SysUser> page(int page, int pageSize, String keyword) {
        LambdaQueryWrapper<SysUser> qw = new LambdaQueryWrapper<>();
        if (keyword != null && !keyword.isBlank()) {
            qw.like(SysUser::getUsername, keyword).or().like(SysUser::getNickname, keyword);
        }
        qw.orderByDesc(SysUser::getCreatedAt);
        return page(new Page<>(page, pageSize), qw);
    }

    @Transactional
    public void updatePassword(Long userId, String newPassword) {
        SysUser user = new SysUser();
        user.setId(userId);
        user.setPasswordHash(passwordEncoder.encode(newPassword));
        updateById(user);
    }

    @Transactional
    public void updateStatus(Long userId, String status) {
        SysUser user = new SysUser();
        user.setId(userId);
        user.setStatus(status);
        updateById(user);
    }
}
