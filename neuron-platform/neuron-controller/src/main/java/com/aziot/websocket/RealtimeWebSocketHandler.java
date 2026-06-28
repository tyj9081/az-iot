package com.aziot.websocket;

import com.aziot.service.cache.RealtimeCache;
import com.fasterxml.jackson.databind.ObjectMapper;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.scheduling.annotation.Scheduled;
import org.springframework.stereotype.Component;
import org.springframework.web.socket.CloseStatus;
import org.springframework.web.socket.TextMessage;
import org.springframework.web.socket.WebSocketSession;
import org.springframework.web.socket.handler.TextWebSocketHandler;

import java.io.IOException;
import java.time.Instant;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

/**
 * WebSocket 实时数据推送处理器。
 *
 * 每 3 秒从 RealtimeCache 取所有活跃设备的最新读数，推送给所有连接的客户端。
 */
@Slf4j
@Component
@RequiredArgsConstructor
public class RealtimeWebSocketHandler extends TextWebSocketHandler {

    private final RealtimeCache realtimeCache;
    private final ObjectMapper objectMapper = new ObjectMapper();

    /** 管理所有连接: sessionId → WebSocketSession */
    private final ConcurrentHashMap<String, WebSocketSession> sessions = new ConcurrentHashMap<>();

    @Override
    public void afterConnectionEstablished(WebSocketSession session) {
        sessions.put(session.getId(), session);
        log.info("WebSocket connected: sessionId={}, activeConnections={}",
            session.getId(), sessions.size());
    }

    @Override
    public void afterConnectionClosed(WebSocketSession session, CloseStatus status) {
        sessions.remove(session.getId());
        log.info("WebSocket disconnected: sessionId={}, closeStatus={}, activeConnections={}",
            session.getId(), status.getCode(), sessions.size());
    }

    @Override
    public void handleTransportError(WebSocketSession session, Throwable exception) {
        log.error("WebSocket transport error: sessionId={}", session.getId(), exception);
        sessions.remove(session.getId());
    }

    /**
     * 每 3 秒推送一次实时数据给所有连接的客户端。
     */
    @Scheduled(fixedRate = 3000)
    public void pushRealtimeData() {
        if (sessions.isEmpty()) return;

        List<Long> activeDeviceIds = realtimeCache.getActiveDeviceIds();
        if (activeDeviceIds.isEmpty()) return;

        List<Map<String, Object>> dataList = new ArrayList<>();
        for (Long deviceId : activeDeviceIds) {
            Map<String, RealtimeCache.RealtimeValue> sensorMap = realtimeCache.getByDevice(deviceId);
            for (Map.Entry<String, RealtimeCache.RealtimeValue> entry : sensorMap.entrySet()) {
                Map<String, Object> item = new LinkedHashMap<>();
                item.put("deviceId", deviceId);
                item.put("sensorCode", entry.getKey());
                item.put("value", entry.getValue().getValue());
                item.put("unit", entry.getValue().getUnit());
                item.put("timestamp", entry.getValue().getTimestamp());
                dataList.add(item);
            }
        }

        Map<String, Object> payload = new LinkedHashMap<>();
        payload.put("type", "realtime");
        payload.put("data", dataList);
        payload.put("timestamp", Instant.now().toString());

        try {
            String json = objectMapper.writeValueAsString(payload);
            TextMessage message = new TextMessage(json);

            List<String> deadSessions = new ArrayList<>();
            for (WebSocketSession session : sessions.values()) {
                if (session.isOpen()) {
                    try {
                        session.sendMessage(message);
                    } catch (IOException e) {
                        // Session may have closed between isOpen check and send
                        deadSessions.add(session.getId());
                    }
                } else {
                    deadSessions.add(session.getId());
                }
            }

            // 清理断开的连接
            deadSessions.forEach(sessions::remove);
        } catch (Exception e) {
            log.error("WebSocket push failed", e);
        }
    }
}
