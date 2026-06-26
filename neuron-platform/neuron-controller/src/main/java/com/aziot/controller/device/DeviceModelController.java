package com.aziot.controller.device;

import com.aziot.common.dto.ApiResponse;
import com.aziot.common.dto.PageResult;
import com.aziot.controller.dto.DeviceModelCreateDTO;
import com.aziot.controller.dto.DeviceModelUpdateDTO;
import com.aziot.common.dto.DeviceModelVO;
import com.aziot.dao.entity.device.DevDeviceModel;
import com.aziot.service.device.DevDeviceModelService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api/v1/device-models")
@RequiredArgsConstructor
public class DeviceModelController {

    private final DevDeviceModelService deviceModelService;

    @GetMapping
    public ApiResponse<PageResult<DevDeviceModel>> list(
            @RequestParam(defaultValue = "1") int page,
            @RequestParam(defaultValue = "20") int pageSize,
            @RequestParam(required = false) Long manufacturerId,
            @RequestParam(required = false) Long protocolId,
            @RequestParam(required = false) String keyword) {
        var result = deviceModelService.page(page, pageSize, manufacturerId, protocolId, keyword);
        return ApiResponse.ok(new PageResult<>(result.getRecords(), result.getTotal(), page, pageSize));
    }

    @GetMapping("/{id}")
    public ApiResponse<DeviceModelVO> get(@PathVariable Long id) {
        return ApiResponse.ok(deviceModelService.getById(id));
    }

    @PostMapping
    public ApiResponse<DevDeviceModel> create(@RequestBody DeviceModelCreateDTO dto) {
        DevDeviceModel model = new DevDeviceModel();
        model.setManufacturerId(dto.getManufacturerId());
        model.setProtocolId(dto.getProtocolId());
        model.setCode(dto.getCode());
        model.setName(dto.getName());
        model.setDescription(dto.getDescription());
        deviceModelService.create(model);
        return ApiResponse.ok(model);
    }

    @PutMapping("/{id}")
    public ApiResponse<DeviceModelVO> update(@PathVariable Long id, @RequestBody DeviceModelUpdateDTO dto) {
        DevDeviceModel model = new DevDeviceModel();
        model.setManufacturerId(dto.getManufacturerId());
        model.setProtocolId(dto.getProtocolId());
        model.setCode(dto.getCode());
        model.setName(dto.getName());
        model.setDescription(dto.getDescription());
        deviceModelService.update(id, model);
        return ApiResponse.ok(deviceModelService.getById(id));
    }

    @DeleteMapping("/{id}")
    public ApiResponse<Void> delete(@PathVariable Long id) {
        deviceModelService.delete(id);
        return ApiResponse.ok();
    }
}
