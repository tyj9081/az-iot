package com.aziot.controller.system;

import com.aziot.common.dto.ApiResponse;
import com.aziot.dao.entity.system.SysPermission;
import com.aziot.service.system.SysPermissionService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;

@RestController
@RequestMapping("/api/v1/permissions")
@RequiredArgsConstructor
public class PermissionController {

    private final SysPermissionService permissionService;

    @GetMapping
    public ApiResponse<List<SysPermission>> tree() {
        return ApiResponse.ok(permissionService.tree());
    }
}
