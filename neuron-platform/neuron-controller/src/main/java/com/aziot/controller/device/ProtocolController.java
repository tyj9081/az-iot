package com.aziot.controller.device;

import com.aziot.common.dto.ApiResponse;
import com.aziot.dao.entity.device.DevProtocol;
import com.aziot.service.device.DevProtocolService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;

/**
 * 协议列表接口。
 * 返回的 DevProtocol 中已包含 busType 字段（serial/tcp），
 * 前端可通过 busType 统一判断协议总线类型，无需硬编码列表。
 *
 * TODO: 前端 device/index.vue 需改为读取 busType 字段，不再硬编码 TCP 协议列表。
 */
@RestController
@RequestMapping("/api/v1/protocols")
@RequiredArgsConstructor
public class ProtocolController {

    private final DevProtocolService protocolService;

    @GetMapping
    public ApiResponse<List<DevProtocol>> listAll() {
        return ApiResponse.ok(protocolService.listAll());
    }
}
