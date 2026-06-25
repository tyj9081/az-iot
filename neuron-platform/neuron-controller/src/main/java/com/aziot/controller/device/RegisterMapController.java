package com.aziot.controller.device;

import com.aziot.common.dto.ApiResponse;
import com.aziot.controller.dto.RegisterMapDTO;
import com.aziot.dao.entity.device.DevRegisterMap;
import com.aziot.service.device.DevRegisterMapService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

import java.util.ArrayList;
import java.util.List;

@RestController
@RequestMapping("/api/v1/device-models/{modelId}/registers")
@RequiredArgsConstructor
public class RegisterMapController {

    private final DevRegisterMapService registerMapService;

    @GetMapping
    public ApiResponse<List<DevRegisterMap>> list(@PathVariable Long modelId) {
        return ApiResponse.ok(registerMapService.listByModelId(modelId));
    }

    @GetMapping("/{id}")
    public ApiResponse<DevRegisterMap> get(@PathVariable Long modelId, @PathVariable Long id) {
        return ApiResponse.ok(registerMapService.getById(id));
    }

    @PostMapping
    public ApiResponse<DevRegisterMap> create(@PathVariable Long modelId, @RequestBody RegisterMapDTO dto) {
        DevRegisterMap register = fromDTO(dto);
        register.setModelId(modelId);
        registerMapService.create(register);
        return ApiResponse.ok(register);
    }

    @PostMapping("/batch")
    public ApiResponse<Void> batchCreate(@PathVariable Long modelId, @RequestBody List<RegisterMapDTO> dtoList) {
        List<DevRegisterMap> list = new ArrayList<>();
        for (RegisterMapDTO dto : dtoList) {
            DevRegisterMap register = fromDTO(dto);
            register.setModelId(modelId);
            list.add(register);
        }
        registerMapService.batchCreate(modelId, list);
        return ApiResponse.ok();
    }

    @PutMapping("/{id}")
    public ApiResponse<DevRegisterMap> update(@PathVariable Long modelId, @PathVariable Long id,
                                               @RequestBody RegisterMapDTO dto) {
        DevRegisterMap register = fromDTO(dto);
        register.setModelId(modelId);
        registerMapService.update(id, register);
        return ApiResponse.ok(registerMapService.getById(id));
    }

    @DeleteMapping("/{id}")
    public ApiResponse<Void> delete(@PathVariable Long modelId, @PathVariable Long id) {
        registerMapService.delete(id);
        return ApiResponse.ok();
    }

    private DevRegisterMap fromDTO(RegisterMapDTO dto) {
        DevRegisterMap register = new DevRegisterMap();
        register.setSensorCode(dto.getSensorCode());
        register.setSensorName(dto.getSensorName());
        register.setRegisterAddress(dto.getRegisterAddress());
        register.setRegisterCount(dto.getRegisterCount());
        register.setDataType(dto.getDataType());
        register.setByteOrder(dto.getByteOrder());
        register.setFuncCode(dto.getFuncCode());
        register.setCoefficient(dto.getCoefficient());
        register.setOffsetVal(dto.getOffsetVal());
        register.setUnit(dto.getUnit());
        register.setRw(dto.getRw());
        register.setSortOrder(dto.getSortOrder());
        register.setDescription(dto.getDescription());
        return register;
    }
}
