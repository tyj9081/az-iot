package com.aziot.controller.system;

import com.aziot.common.dto.ApiResponse;
import com.aziot.dao.entity.system.SysRole;
import com.aziot.service.system.SysRoleService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

import java.util.List;

@RestController
@RequestMapping("/api/v1/roles")
@RequiredArgsConstructor
public class RoleController {

    private final SysRoleService roleService;

    @GetMapping
    public ApiResponse<List<SysRole>> list() {
        return ApiResponse.ok(roleService.listAll());
    }

    @GetMapping("/{id}")
    public ApiResponse<SysRole> get(@PathVariable Long id) {
        return ApiResponse.ok(roleService.getById(id));
    }

    @PostMapping
    public ApiResponse<SysRole> create(@RequestBody SysRole role) {
        roleService.save(role);
        return ApiResponse.ok(role);
    }

    @PutMapping("/{id}")
    public ApiResponse<SysRole> update(@PathVariable Long id, @RequestBody SysRole role) {
        role.setId(id);
        roleService.updateById(role);
        return ApiResponse.ok(roleService.getById(id));
    }

    @DeleteMapping("/{id}")
    public ApiResponse<Void> delete(@PathVariable Long id) {
        roleService.removeById(id);
        return ApiResponse.ok();
    }

    @PutMapping("/{id}/permissions")
    public ApiResponse<Void> assignPermissions(@PathVariable Long id, @RequestBody List<Long> permIds) {
        // TODO: 权限分配 - 需要 SysRolePermission 中间表支持
        return ApiResponse.ok();
    }
}
