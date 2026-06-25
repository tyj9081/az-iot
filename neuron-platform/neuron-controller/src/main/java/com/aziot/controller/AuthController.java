package com.aziot.controller;

import com.aziot.common.dto.ApiResponse;
import com.aziot.common.exception.BusinessException;
import com.aziot.security.JwtTokenProvider;
import com.aziot.service.system.AuthService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

import java.util.Map;

@RestController
@RequestMapping("/api/v1/auth")
@RequiredArgsConstructor
public class AuthController {

    private final AuthService authService;
    private final JwtTokenProvider jwtTokenProvider;

    @PostMapping("/login")
    public ApiResponse<Map<String, String>> login(@RequestBody Map<String, String> body) {
        String username = body.get("username");
        String password = body.get("password");
        if (username == null || username.isBlank() || password == null || password.isBlank()) {
            throw new BusinessException(400, "用户名和密码不能为空");
        }
        return ApiResponse.ok(authService.login(username, password));
    }

    @PostMapping("/refresh")
    public ApiResponse<Map<String, String>> refresh(@RequestBody Map<String, String> body) {
        String refreshToken = body.get("refreshToken");
        if (refreshToken == null || refreshToken.isBlank()) {
            throw new BusinessException(400, "refreshToken 不能为空");
        }
        return ApiResponse.ok(authService.refresh(refreshToken));
    }

    @GetMapping("/me")
    public ApiResponse<Map<String, Object>> me(@RequestHeader("Authorization") String auth) {
        String token = auth.substring(7);
        Long userId = jwtTokenProvider.getUserId(token);
        var user = authService.getCurrentUser(userId);
        return ApiResponse.ok(Map.of(
                "id", user.getId(),
                "username", user.getUsername(),
                "nickname", user.getNickname()));
    }

    @PostMapping("/logout")
    public ApiResponse<Void> logout() {
        // Stateless JWT — client discards token; future: add token blacklist
        return ApiResponse.ok();
    }
}
