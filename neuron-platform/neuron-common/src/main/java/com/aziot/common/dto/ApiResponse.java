package com.aziot.common.dto;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@NoArgsConstructor
@AllArgsConstructor
public class ApiResponse<T> {
    private int code;
    private String message;
    private T data;
    private long timestamp;

    public static <T> ApiResponse<T> ok(T data) {
        return new ApiResponse<>(0, "success", data, System.currentTimeMillis());
    }

    public static <T> ApiResponse<T> ok() {
        return ok(null);
    }

    public static <T> ApiResponse<T> fail(int code, String message) {
        return new ApiResponse<>(code, message, null, System.currentTimeMillis());
    }

    public static <T> ApiResponse<T> fail(String message) {
        return fail(500, message);
    }
}
