package com.aziot.controller.system;

import com.aziot.common.dto.ApiResponse;
import com.aziot.common.dto.PageResult;
import com.aziot.dao.entity.system.SysUser;
import com.aziot.service.system.SysUserService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

import java.util.Map;

@RestController
@RequestMapping("/api/v1/users")
@RequiredArgsConstructor
public class UserController {

    private final SysUserService userService;

    @GetMapping
    public ApiResponse<PageResult<SysUser>> list(
            @RequestParam(defaultValue = "1") int page,
            @RequestParam(defaultValue = "20") int pageSize,
            @RequestParam(required = false) String keyword) {
        var result = userService.page(page, pageSize, keyword);
        return ApiResponse.ok(new PageResult<>(result.getRecords(), result.getTotal(), page, pageSize));
    }

    @GetMapping("/{id}")
    public ApiResponse<SysUser> get(@PathVariable Long id) {
        return ApiResponse.ok(userService.getById(id));
    }

    @PostMapping
    public ApiResponse<SysUser> create(@RequestBody SysUser user) {
        userService.save(user);
        return ApiResponse.ok(user);
    }

    @PutMapping("/{id}")
    public ApiResponse<SysUser> update(@PathVariable Long id, @RequestBody SysUser user) {
        user.setId(id);
        userService.updateById(user);
        return ApiResponse.ok(userService.getById(id));
    }

    @PutMapping("/{id}/password")
    public ApiResponse<Void> updatePassword(@PathVariable Long id, @RequestBody Map<String, String> body) {
        userService.updatePassword(id, body.get("password"));
        return ApiResponse.ok();
    }

    @PutMapping("/{id}/status")
    public ApiResponse<Void> updateStatus(@PathVariable Long id, @RequestBody Map<String, String> body) {
        userService.updateStatus(id, body.get("status"));
        return ApiResponse.ok();
    }
}
