package com.aziot.controller;

import com.aziot.dao.entity.collector.DevCollector;
import com.aziot.dao.mapper.collector.DevCollectorMapper;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import lombok.RequiredArgsConstructor;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.web.bind.annotation.*;

import java.util.Map;

@RestController
@RequestMapping("/api/v1/mqtt")
@RequiredArgsConstructor
public class MqttAuthController {

    private final DevCollectorMapper collectorMapper;
    private final PasswordEncoder passwordEncoder;

    @PostMapping("/auth")
    public Map<String, Object> auth(@RequestBody Map<String, String> body) {
        String username = body.get("username");
        String password = body.get("password");

        // 允许服务端管理账号
        String adminUser = System.getenv().getOrDefault("MQTT_ADMIN_USER", "");
        String adminPass = System.getenv().getOrDefault("MQTT_ADMIN_PASS", "");
        if (!adminUser.isEmpty() && adminUser.equals(username) && adminPass.equals(password)) {
            return Map.of("result", "allow", "is_superuser", true);
        }

        // 查找采集器凭证
        DevCollector collector = collectorMapper.selectOne(
            new LambdaQueryWrapper<DevCollector>().eq(DevCollector::getMqttUsername, username));
        if (collector == null || !passwordEncoder.matches(password, collector.getMqttPasswordHash())) {
            return Map.of("result", "deny");
        }

        return Map.of("result", "allow", "client_id", collector.getMqttClientId());
    }
}
