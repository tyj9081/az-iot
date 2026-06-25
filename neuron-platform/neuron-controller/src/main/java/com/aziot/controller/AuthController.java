package com.aziot.controller;

import com.aziot.common.dto.ApiResponse;
import com.aziot.security.JwtTokenProvider;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.aziot.dao.entity.system.SysUser;
import com.aziot.dao.mapper.system.SysUserMapper;
import lombok.RequiredArgsConstructor;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.web.bind.annotation.*;
import java.util.Map;

@RestController
@RequestMapping("/api/v1/auth")
@RequiredArgsConstructor
public class AuthController {

    private final SysUserMapper userMapper;
    private final PasswordEncoder passwordEncoder;
    private final JwtTokenProvider jwtTokenProvider;

    @PostMapping("/login")
    public ApiResponse<Map<String, String>> login(@RequestBody Map<String, String> body) {
        String username = body.get("username");
        String password = body.get("password");

        SysUser user = userMapper.selectOne(
                new LambdaQueryWrapper<SysUser>().eq(SysUser::getUsername, username));
        if (user == null || "0".equals(user.getStatus())) {
            throw new RuntimeException("账号不存在或已禁用");
        }
        if (!passwordEncoder.matches(password, user.getPasswordHash())) {
            throw new RuntimeException("密码错误");
        }

        String accessToken = jwtTokenProvider.generateAccessToken(user.getId(), username);
        String refreshToken = jwtTokenProvider.generateRefreshToken(user.getId(), username);

        return ApiResponse.ok(Map.of("accessToken", accessToken, "refreshToken", refreshToken));
    }

    @PostMapping("/refresh")
    public ApiResponse<Map<String, String>> refresh(@RequestBody Map<String, String> body) {
        String refreshToken = body.get("refreshToken");
        if (!jwtTokenProvider.validateToken(refreshToken)) {
            throw new RuntimeException("Token 无效或已过期");
        }
        Long userId = jwtTokenProvider.getUserId(refreshToken);
        String username = jwtTokenProvider.getUsername(refreshToken);

        String newAccess = jwtTokenProvider.generateAccessToken(userId, username);
        String newRefresh = jwtTokenProvider.generateRefreshToken(userId, username);
        return ApiResponse.ok(Map.of("accessToken", newAccess, "refreshToken", newRefresh));
    }

    @GetMapping("/me")
    public ApiResponse<Map<String, Object>> me(@RequestHeader("Authorization") String auth) {
        String token = auth.substring(7);
        Long userId = jwtTokenProvider.getUserId(token);
        SysUser user = userMapper.selectById(userId);
        return ApiResponse.ok(Map.of(
                "id", user.getId(), "username", user.getUsername(), "nickname", user.getNickname()));
    }

    @PostMapping("/logout")
    public ApiResponse<Void> logout() {
        return ApiResponse.ok();
    }
}
