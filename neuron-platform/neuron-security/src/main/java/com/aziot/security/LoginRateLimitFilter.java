package com.aziot.security;

import com.aziot.common.dto.ApiResponse;
import com.fasterxml.jackson.databind.ObjectMapper;
import jakarta.servlet.*;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.annotation.Order;
import org.springframework.http.HttpStatus;
import org.springframework.http.MediaType;
import org.springframework.stereotype.Component;

import java.io.IOException;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.TimeUnit;

/**
 * 登录接口限流 — 基于 IP 的令牌桶算法。
 * 每个 IP 每分钟最多 5 次登录尝试。
 */
@Slf4j
@Component
@Order(1)
public class LoginRateLimitFilter implements Filter {

    private static final int MAX_ATTEMPTS_PER_MINUTE = 5;
    private static final long WINDOW_MS = TimeUnit.MINUTES.toMillis(1);

    private final Map<String, RateBucket> buckets = new ConcurrentHashMap<>();
    private final ObjectMapper objectMapper = new ObjectMapper();

    @Override
    public void doFilter(ServletRequest request, ServletResponse response, FilterChain chain)
            throws IOException, ServletException {
        HttpServletRequest req = (HttpServletRequest) request;
        HttpServletResponse res = (HttpServletResponse) response;

        if (!"/api/v1/auth/login".equals(req.getRequestURI()) || !"POST".equalsIgnoreCase(req.getMethod())) {
            chain.doFilter(request, response);
            return;
        }

        String ip = getClientIp(req);
        buckets.computeIfAbsent(ip, k -> new RateBucket());
        RateBucket bucket = buckets.get(ip);

        long now = System.currentTimeMillis();
        synchronized (bucket) {
            if (now - bucket.windowStart > WINDOW_MS) {
                bucket.windowStart = now;
                bucket.count = 0;
            }
            if (bucket.count >= MAX_ATTEMPTS_PER_MINUTE) {
                log.warn("Login rate limit exceeded for IP: {}", ip);
                res.setStatus(HttpStatus.TOO_MANY_REQUESTS.value());
                res.setContentType(MediaType.APPLICATION_JSON_VALUE);
                res.getWriter().write(objectMapper.writeValueAsString(
                    ApiResponse.fail(429, "操作过于频繁，请稍后再试")
                ));
                return;
            }
            bucket.count++;
        }

        chain.doFilter(request, response);
    }

    private String getClientIp(HttpServletRequest request) {
        String xff = request.getHeader("X-Forwarded-For");
        if (xff != null && !xff.isBlank()) {
            return xff.split(",")[0].trim();
        }
        return request.getRemoteAddr();
    }

    private static class RateBucket {
        long windowStart = System.currentTimeMillis();
        int count = 0;
    }
}
