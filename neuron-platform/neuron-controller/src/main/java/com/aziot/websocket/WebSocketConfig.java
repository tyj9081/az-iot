package com.aziot.websocket;

import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Configuration;
import org.springframework.http.server.ServerHttpRequest;
import org.springframework.http.server.ServerHttpResponse;
import org.springframework.web.socket.WebSocketHandler;
import org.springframework.web.socket.config.annotation.EnableWebSocket;
import org.springframework.web.socket.config.annotation.WebSocketConfigurer;
import org.springframework.web.socket.config.annotation.WebSocketHandlerRegistry;
import org.springframework.web.socket.server.support.HttpSessionHandshakeInterceptor;

import java.util.Map;

/**
 * WebSocket 配置 — 注册 /ws/realtime 端点用于前端实时数据推送。
 */
@Slf4j
@Configuration
@EnableWebSocket
@RequiredArgsConstructor
public class WebSocketConfig implements WebSocketConfigurer {

    private final RealtimeWebSocketHandler realtimeHandler;

    @Override
    public void registerWebSocketHandlers(WebSocketHandlerRegistry registry) {
        registry.addHandler(realtimeHandler, "/ws/realtime")
                .setAllowedOriginPatterns("*")  // TODO: restrict to frontend domain in production
                .addInterceptors(new HttpSessionHandshakeInterceptor() {
                    @Override
                    public boolean beforeHandshake(ServerHttpRequest request, ServerHttpResponse response,
                            WebSocketHandler wsHandler, Map<String, Object> attributes) {
                        // TODO: validate JWT token from query param or Authorization header
                        // For now, allow connection but log warning about missing auth
                        log.warn("WebSocket handshake without authentication — restrict in production");
                        return true;
                    }
                });
    }
}
