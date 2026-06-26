package com.aziot.controller.device;

import com.aziot.common.dto.ApiResponse;
import com.aziot.controller.dto.DeviceInstructionCreateDTO;
import com.aziot.dao.entity.device.DevDeviceInstruction;
import com.aziot.service.device.DevDeviceInstructionService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

import java.util.List;

@RestController
@RequestMapping("/api/v1/devices/{deviceId}/instructions")
@RequiredArgsConstructor
public class DeviceInstructionController {

    private final DevDeviceInstructionService instructionService;

    @GetMapping
    public ApiResponse<List<DevDeviceInstruction>> list(@PathVariable Long deviceId) {
        return ApiResponse.ok(instructionService.listByDeviceId(deviceId));
    }

    @GetMapping("/{id}")
    public ApiResponse<DevDeviceInstruction> get(@PathVariable Long deviceId, @PathVariable Long id) {
        return ApiResponse.ok(instructionService.getById(id));
    }

    @PostMapping
    public ApiResponse<DevDeviceInstruction> create(@PathVariable Long deviceId,
                                                     @RequestBody DeviceInstructionCreateDTO dto) {
        DevDeviceInstruction instr = new DevDeviceInstruction();
        instr.setDeviceId(deviceId);
        instr.setInstructionCode(dto.getInstructionCode());
        instr.setInstructionName(dto.getInstructionName());
        instr.setInstructionType(dto.getInstructionType());
        instr.setFuncCode(dto.getFuncCode());
        instr.setRegisterAddress(dto.getRegisterAddress());
        instr.setRegisterCount(dto.getRegisterCount());
        instr.setParams(dto.getParams());
        instr.setSortOrder(dto.getSortOrder());
        instr.setDescription(dto.getDescription());
        instructionService.create(instr);
        return ApiResponse.ok(instr);
    }

    @PutMapping("/{id}")
    public ApiResponse<DevDeviceInstruction> update(@PathVariable Long deviceId,
                                                     @PathVariable Long id,
                                                     @RequestBody DeviceInstructionCreateDTO dto) {
        DevDeviceInstruction instr = new DevDeviceInstruction();
        instr.setInstructionCode(dto.getInstructionCode());
        instr.setInstructionName(dto.getInstructionName());
        instr.setInstructionType(dto.getInstructionType());
        instr.setFuncCode(dto.getFuncCode());
        instr.setRegisterAddress(dto.getRegisterAddress());
        instr.setRegisterCount(dto.getRegisterCount());
        instr.setParams(dto.getParams());
        instr.setSortOrder(dto.getSortOrder());
        instr.setDescription(dto.getDescription());
        instructionService.update(id, instr);
        return ApiResponse.ok(instructionService.getById(id));
    }

    @DeleteMapping("/{id}")
    public ApiResponse<Void> delete(@PathVariable Long deviceId, @PathVariable Long id) {
        instructionService.delete(id);
        return ApiResponse.ok();
    }
}
