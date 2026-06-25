package com.aziot.controller.device;

import com.aziot.common.dto.ApiResponse;
import com.aziot.common.dto.PageResult;
import com.aziot.controller.dto.ManufacturerCreateDTO;
import com.aziot.controller.dto.ManufacturerUpdateDTO;
import com.aziot.dao.entity.device.DevManufacturer;
import com.aziot.service.device.DevManufacturerService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api/v1/manufacturers")
@RequiredArgsConstructor
public class ManufacturerController {

    private final DevManufacturerService manufacturerService;

    @GetMapping
    public ApiResponse<PageResult<DevManufacturer>> list(
            @RequestParam(defaultValue = "1") int page,
            @RequestParam(defaultValue = "20") int pageSize,
            @RequestParam(required = false) String keyword) {
        var result = manufacturerService.page(page, pageSize, keyword);
        return ApiResponse.ok(new PageResult<>(result.getRecords(), result.getTotal(), page, pageSize));
    }

    @GetMapping("/{id}")
    public ApiResponse<DevManufacturer> get(@PathVariable Long id) {
        return ApiResponse.ok(manufacturerService.getById(id));
    }

    @PostMapping
    public ApiResponse<DevManufacturer> create(@RequestBody ManufacturerCreateDTO dto) {
        DevManufacturer manufacturer = new DevManufacturer();
        manufacturer.setCode(dto.getCode());
        manufacturer.setName(dto.getName());
        manufacturer.setCountry(dto.getCountry());
        manufacturer.setWebsite(dto.getWebsite());
        manufacturer.setDescription(dto.getDescription());
        manufacturerService.create(manufacturer);
        return ApiResponse.ok(manufacturer);
    }

    @PutMapping("/{id}")
    public ApiResponse<DevManufacturer> update(@PathVariable Long id, @RequestBody ManufacturerUpdateDTO dto) {
        DevManufacturer manufacturer = new DevManufacturer();
        manufacturer.setCode(dto.getCode());
        manufacturer.setName(dto.getName());
        manufacturer.setCountry(dto.getCountry());
        manufacturer.setWebsite(dto.getWebsite());
        manufacturer.setDescription(dto.getDescription());
        manufacturerService.update(id, manufacturer);
        return ApiResponse.ok(manufacturerService.getById(id));
    }

    @DeleteMapping("/{id}")
    public ApiResponse<Void> delete(@PathVariable Long id) {
        manufacturerService.delete(id);
        return ApiResponse.ok();
    }
}
