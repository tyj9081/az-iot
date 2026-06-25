package com.aziot.service.system;

import com.aziot.common.exception.BusinessException;
import com.aziot.dao.entity.system.SysUser;
import com.aziot.dao.mapper.system.SysUserMapper;
import com.aziot.security.JwtTokenProvider;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import lombok.RequiredArgsConstructor;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;

import java.util.Map;

@Service
@RequiredArgsConstructor
public class AuthService {

    private final SysUserMapper userMapper;
    private final PasswordEncoder passwordEncoder;
    private final JwtTokenProvider jwtTokenProvider;

    /**
     * 用户登录，返回 accessToken 和 refreshToken。
     * 登录失败统一返回 401，不区分"账号不存在"和"密码错误"以防范用户枚举攻击。
     */
    public Map<String, String> login(String username, String password) {
        SysUser user = userMapper.selectOne(
                new LambdaQueryWrapper<SysUser>().eq(SysUser::getUsername, username));
        if (user == null || !"1".equals(user.getStatus())) {
            throw new BusinessException(401, "用户名或密码错误");
        }
        if (!passwordEncoder.matches(password, user.getPasswordHash())) {
            throw new BusinessException(401, "用户名或密码错误");
        }

        String accessToken = jwtTokenProvider.generateAccessToken(user.getId(), username);
        String refreshToken = jwtTokenProvider.generateRefreshToken(user.getId(), username);

        return Map.of("accessToken", accessToken, "refreshToken", refreshToken);
    }

    /**
     * 使用 Refresh Token 刷新令牌对。
     * 校验 Token 类型必须为 "refresh"，防止 Access Token 也能刷新的安全风险。
     */
    public Map<String, String> refresh(String refreshToken) {
        if (!jwtTokenProvider.validateToken(refreshToken)) {
            throw new BusinessException(401, "登录已过期，请重新登录");
        }
        String tokenType = jwtTokenProvider.getTokenType(refreshToken);
        if (!"refresh".equals(tokenType)) {
            throw new BusinessException(401, "无效的刷新令牌");
        }

        Long userId = jwtTokenProvider.getUserId(refreshToken);
        String username = jwtTokenProvider.getUsername(refreshToken);

        String newAccess = jwtTokenProvider.generateAccessToken(userId, username);
        String newRefresh = jwtTokenProvider.generateRefreshToken(userId, username);
        return Map.of("accessToken", newAccess, "refreshToken", newRefresh);
    }

    /**
     * 获取当前登录用户信息。
     */
    public SysUser getCurrentUser(Long userId) {
        SysUser user = userMapper.selectById(userId);
        if (user == null) {
            throw BusinessException.notFound("用户");
        }
        return user;
    }
}
