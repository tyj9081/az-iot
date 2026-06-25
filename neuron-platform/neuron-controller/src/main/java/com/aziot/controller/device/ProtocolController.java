package com.aziot.controller.device;

import com.aziot.common.dto.ApiResponse;
import com.aziot.dao.entity.device.DevProtocol;
import com.aziot.service.device.DevProtocolService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;

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
