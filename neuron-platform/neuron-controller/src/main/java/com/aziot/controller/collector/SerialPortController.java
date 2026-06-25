package com.aziot.controller.collector;

import com.aziot.common.dto.ApiResponse;
import com.aziot.dao.entity.collector.DevSerialPort;
import com.aziot.service.collector.DevSerialPortService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

import java.util.List;

@RestController
@RequestMapping("/api/v1/collectors/{collectorId}/serial-ports")
@RequiredArgsConstructor
public class SerialPortController {

    private final DevSerialPortService serialPortService;

    @GetMapping
    public ApiResponse<List<DevSerialPort>> list(@PathVariable Long collectorId) {
        return ApiResponse.ok(serialPortService.listByCollectorId(collectorId));
    }

    @PutMapping("/{id}")
    public ApiResponse<DevSerialPort> update(@PathVariable Long collectorId, @PathVariable Long id, @RequestBody DevSerialPort port) {
        port.setId(id);
        serialPortService.update(id, port);
        return ApiResponse.ok(serialPortService.getById(id));
    }
}
