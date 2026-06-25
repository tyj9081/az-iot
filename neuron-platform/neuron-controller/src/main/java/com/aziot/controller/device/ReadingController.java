package com.aziot.controller.device;

import com.aziot.common.dto.ApiResponse;
import com.aziot.service.device.DevDeviceReadingService;
import lombok.RequiredArgsConstructor;
import org.springframework.format.annotation.DateTimeFormat;
import org.springframework.web.bind.annotation.*;
import java.time.LocalDateTime;
import java.util.Map;

@RestController
@RequestMapping("/api/v1")
@RequiredArgsConstructor
public class ReadingController {

    private final DevDeviceReadingService readingService;

    @GetMapping("/devices/{id}/readings/latest")
    public ApiResponse<?> latest(@PathVariable Long id) {
        return ApiResponse.ok(readingService.latestByDeviceId(id));
    }

    @GetMapping("/devices/{id}/readings/history")
    public ApiResponse<?> history(
            @PathVariable Long id,
            @RequestParam(required = false) String sensorCode,
            @RequestParam @DateTimeFormat(pattern = "yyyy-MM-dd HH:mm:ss") LocalDateTime from,
            @RequestParam @DateTimeFormat(pattern = "yyyy-MM-dd HH:mm:ss") LocalDateTime to) {
        return ApiResponse.ok(readingService.history(id, sensorCode, from, to));
    }

    @GetMapping("/dashboard/overview")
    public ApiResponse<Map<String, Object>> overview() {
        return ApiResponse.ok(readingService.dashboardOverview());
    }
}
