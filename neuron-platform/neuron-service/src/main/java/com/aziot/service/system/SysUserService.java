package com.aziot.service.system;

import com.aziot.dao.entity.system.SysUser;
import com.aziot.dao.mapper.system.SysUserMapper;
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
        return baseMapper.selectPageByCondition(new Page<>(page, pageSize), keyword, null);
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
        user.setStatus(Integer.parseInt(status));
        updateById(user);
    }
}
