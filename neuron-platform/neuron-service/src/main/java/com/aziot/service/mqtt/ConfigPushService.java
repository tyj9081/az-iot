package com.aziot.service.mqtt;

import com.aziot.dao.entity.collector.DevCollector;
import com.aziot.dao.entity.collector.DevSerialPort;
import com.aziot.dao.entity.device.*;
import com.aziot.dao.mapper.collector.DevCollectorMapper;
import com.aziot.dao.mapper.collector.DevSerialPortMapper;
import com.aziot.dao.mapper.device.*;
import com.fasterxml.jackson.databind.ObjectMapper;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import java.util.*;

/**
 * 配置推送服务 — 将设备配置通过 MQTT 推送给采集端。
 *
 * 核心设计:
 *   不同协议的设备数据点（data_points）结构不同:
 *   - MODBUS_RTU / MODBUS_TCP / DL_T645 等轮询协议:
 *       推送 register_address、func_code、byte_order 等完整寄存器字段。
 *       采集端根据这些字段构造 Modbus 请求帧，定时轮询。
 *   - MQTT (设备主动上报):
 *       推送 topic、json_path 等订阅/解析配置。
 *       采集端订阅对应 topic，收到消息后按 json_path 提取值。
 *   - HTTP_JSON / SNMP / OPC_UA / BACNET_IP 等:
 *       推送 extra_params 中的协议特有配置(url、oid、node_id 等)。
 *
 * bus 配置:
 *   - serial 协议: 推送串口参数(port_name、baud_rate 等)
 *   - tcp 协议: 推送 TCP 连接参数(host、port)
 *   - MQTT 协议: 推送 broker(host、port、topic)
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class ConfigPushService {
    private final DevDeviceMapper deviceMapper;
    private final DevSerialPortMapper serialPortMapper;
    private final DevCollectorMapper collectorMapper;
    private final DevDeviceModelMapper modelMapper;
    private final DevProtocolMapper protocolMapper;
    private final DevRegisterMapMapper registerMapMapper;
    private final DevDeviceAlarmConfigMapper alarmConfigMapper;
    private final MqttPublisherService mqttPublisher;
    private final ObjectMapper objectMapper;

    /** 需要轮询寄存器的协议编码集合 */
    private static final Set<String> POLLING_PROTOCOLS = Set.of(
        "MODBUS_RTU", "MODBUS_TCP", "DL_T645_2007", "DL_T645_1997",
        "IEC_60870_5_101", "IEC_60870_5_104", "DNP3"
    );

    public void pushDelta(Long deviceId, String action) {
        try {
            // 1. 加载设备关联链路
            DeviceContext ctx = loadContext(deviceId);
            if (ctx == null) return;

            // 2. 构建推送 payload
            Map<String, Object> payload = new LinkedHashMap<>();
            payload.put("version", System.currentTimeMillis());
            payload.put("action", action);

            Map<String, Object> deviceNode = buildDeviceNode(ctx);
            payload.put("device", deviceNode);

            // 3. 发布
            String json = objectMapper.writeValueAsString(payload);
            String topic = "neuron/" + ctx.collector.getMqttClientId() + "/config/delta";
            mqttPublisher.publish(topic, json);
            log.info("MQTT CONFIG PUSH topic={} deviceId={} action={}", topic, deviceId, action);
        } catch (Exception e) {
            log.error("Config push failed for device " + deviceId, e);
        }
    }

    // ══════════════════════════════════════════
    // 上下文加载
    // ══════════════════════════════════════════

    private static class DeviceContext {
        DevDevice device;
        DevDeviceModel model;
        DevProtocol protocol;
        DevSerialPort port;       // 串口设备有值，网络协议设备为 null
        DevCollector collector;
        List<DevRegisterMap> registers;
    }

    private DeviceContext loadContext(Long deviceId) {
        DevDevice device = deviceMapper.selectById(deviceId);
        if (device == null) return null;
        DevDeviceModel model = modelMapper.selectById(device.getModelId());
        if (model == null) return null;
        DevProtocol protocol = protocolMapper.selectById(model.getProtocolId());
        if (protocol == null) return null;
        List<DevRegisterMap> registers = registerMapMapper.selectByModelId(device.getModelId());

        DevSerialPort port = null;
        DevCollector collector = null;

        if (device.getSerialPortId() != null) {
            port = serialPortMapper.selectById(device.getSerialPortId());
            if (port != null) collector = collectorMapper.selectById(port.getCollectorId());
        }

        // 网络协议设备无串口，需要显式绑定采集器
        if (collector == null) {
            log.warn("Device {} is network-protocol device with no collector binding. " +
                     "Config push may go to wrong collector in multi-collector setup. " +
                     "Falling back to first available collector as best-effort.", deviceId);
            var list = collectorMapper.selectList(null);
            if (list != null && !list.isEmpty()) {
                collector = list.get(0);
            } else {
                log.error("No collector available for device {}, config push aborted", deviceId);
                return null;
            }
        }
        if (collector == null) return null;

        DeviceContext ctx = new DeviceContext();
        ctx.device = device;
        ctx.model = model;
        ctx.protocol = protocol;
        ctx.port = port;
        ctx.collector = collector;
        ctx.registers = registers;
        return ctx;
    }

    // ══════════════════════════════════════════
    // 设备节点构建
    // ══════════════════════════════════════════

    private Map<String, Object> buildDeviceNode(DeviceContext ctx) {
        Map<String, Object> node = new LinkedHashMap<>();
        node.put("id", ctx.device.getId());
        node.put("code", ctx.device.getCode());
        node.put("name", ctx.device.getName());
        node.put("protocol", ctx.protocol.getCode());
        node.put("slave_addr", ctx.device.getSlaveAddr());

        // bus 配置 — 按协议类型区分
        node.put("bus", buildBusConfig(ctx));

        // data_points — 按协议类型区分
        node.put("data_points", buildDataPoints(ctx));

        // 采集间隔
        Integer interval = ctx.device.getCollectIntervalSec();
        if (interval == null) interval = ctx.collector.getCollectIntervalSec();
        node.put("collect_interval_sec", interval);

        // 告警配置
        node.put("alarm_config", buildAlarmConfig(ctx.device.getId()));

        return node;
    }

    // ══════════════════════════════════════════
    // bus 配置
    // ══════════════════════════════════════════

    /**
     * 构建 bus 配置，按协议总线类型分发。
     *
     * 路由依据: DevProtocol.busType (数据库字段，取值 "serial" / "tcp")，
     * 特殊情况: MQTT 协议虽 bus_type=tcp，但需要 broker 配置和 subscribe_topic。
     *
     * Rust 端 BusType 枚举只接受两种变体:
     *   Serial { port_name, bus_param }   — 对应 bus_type="serial"
     *   Tcp { host, port }                — 对应 bus_type="tcp"
     */
    @SuppressWarnings("unchecked")
    private Map<String, Object> buildBusConfig(DeviceContext ctx) {
        Map<String, Object> bus = new LinkedHashMap<>();
        String protocolCode = ctx.protocol.getCode();
        String busType = nvl(ctx.protocol.getBusType(), "serial");

        if ("MQTT".equals(protocolCode)) {
            // MQTT 设备: 采集端需要知道 broker 地址和订阅 topic (Rust MQTT 驱动自行处理连接)
            Map<String, Object> mqttBus = new LinkedHashMap<>();
            mqttBus.put("broker_host", nvl(ctx.collector.getMqttBrokerHost(), "127.0.0.1"));
            mqttBus.put("broker_port", nvl(ctx.collector.getMqttBrokerPort(), 1883));
            if (!ctx.registers.isEmpty()) {
                try {
                    Map<String, Object> ep = objectMapper.readValue(
                        nvl(ctx.registers.get(0).getExtraParams(), "{}"), Map.class);
                    mqttBus.put("subscribe_topic", ep.getOrDefault("topic", "+/data"));
                } catch (Exception ignored) {
                    mqttBus.put("subscribe_topic", "+/data");
                }
            }
            bus.put("mqtt", mqttBus);
        } else if ("tcp".equals(busType)) {
            // TCP 网络协议: Modbus TCP, BACnet/IP, OPC UA, IEC 104, DNP3, S7, FINS,
            //                EtherNet/IP, Mitsubishi MC, SNMP, HTTP JSON
            Map<String, Object> tcpBus = new LinkedHashMap<>();
            tcpBus.put("host", nvl(ctx.collector.getIpAddress(), "127.0.0.1"));

            // Protocol default port mapping
            int defaultPort = switch (protocolCode) {
                case "MODBUS_TCP" -> 502;
                case "BACNET_IP" -> 47808;
                case "OPC_UA" -> 4840;
                case "SNMP" -> 161;
                case "HTTP_JSON" -> 80;
                case "HTTPS_JSON" -> 443;
                case "IEC_104" -> 2404;
                case "DNP3_TCP" -> 20000;
                case "S7" -> 102;
                case "FINS_TCP" -> 9600;
                case "ETHERNET_IP" -> 44818;
                case "MITSUBISHI_MC" -> 5000;
                default -> {
                    log.warn("Unknown TCP protocol {}, defaulting port to 502", protocolCode);
                    yield 502;
                }
            };
            tcpBus.put("port", defaultPort);
            bus.put("tcp", tcpBus);
        } else {
            // serial 协议: Modbus RTU, DL/T645, IEC 101, CAN Bus, PROFIBUS, HostLink
            Map<String, Object> serialBus = new LinkedHashMap<>();
            if (ctx.port != null) {
                serialBus.put("port_name", ctx.port.getPortName());
                try {
                    serialBus.put("bus_param", objectMapper.readTree(ctx.port.getBusParam()));
                } catch (Exception e) {
                    serialBus.put("bus_param", ctx.port.getBusParam());
                }
            } else {
                serialBus.put("port_name", "N/A");
                serialBus.put("bus_param", objectMapper.createObjectNode());
            }
            bus.put("serial", serialBus);
        }
        return bus;
    }

    // ══════════════════════════════════════════
    // data_points — 按协议类型区分配置字段
    // ══════════════════════════════════════════

    private List<Map<String, Object>> buildDataPoints(DeviceContext ctx) {
        String protocolCode = ctx.protocol.getCode();
        return switch (protocolCode) {
            case "MQTT"       -> buildMqttDataPoints(ctx.registers);
            case "HTTP_JSON"  -> buildHttpDataPoints(ctx.registers);
            case "OPC_UA"     -> buildOpcUaDataPoints(ctx.registers);
            default           -> buildPollingDataPoints(ctx.registers);
        };
    }

    /**
     * 轮询协议数据点 (Modbus RTU/TCP, DL/T645, DNP3 等)
     * 采集端需要寄存器地址、功能码、字节序等构造轮询帧
     */
    private List<Map<String, Object>> buildPollingDataPoints(List<DevRegisterMap> registers) {
        List<Map<String, Object>> dps = new ArrayList<>();
        for (DevRegisterMap rm : registers) {
            Map<String, Object> dp = new LinkedHashMap<>();
            dp.put("sensor_code", rm.getSensorCode());
            dp.put("sensor_name", rm.getSensorName());
            dp.put("register_address", rm.getRegisterAddress());
            dp.put("register_count", nvl(rm.getRegisterCount(), 1));
            dp.put("data_type", rm.getDataType());
            dp.put("byte_order", rm.getByteOrder());
            dp.put("func_code", rm.getFuncCode());
            dp.put("coefficient", nvl(rm.getCoefficient(), 1));
            dp.put("offset", nvl(rm.getOffsetVal(), 0));
            dp.put("unit", nvl(rm.getUnit(), ""));
            putExtraParams(dp, rm.getExtraParams());
            dps.add(dp);
        }
        return dps;
    }

    /**
     * MQTT 协议数据点 — 设备主动上报，采集端订阅解析
     *
     * extra_params 预期格式:
     *   {"topic": "device/+/sensor", "json_path": "$.temperature", "qos": 1}
     */
    private List<Map<String, Object>> buildMqttDataPoints(List<DevRegisterMap> registers) {
        List<Map<String, Object>> dps = new ArrayList<>();
        for (DevRegisterMap rm : registers) {
            Map<String, Object> dp = new LinkedHashMap<>();
            dp.put("sensor_code", rm.getSensorCode());
            dp.put("sensor_name", rm.getSensorName());
            dp.put("unit", nvl(rm.getUnit(), ""));

            // 从 extra_params 解析 MQTT 特有的 topic / json_path
            Map<String, Object> ep = parseExtraParams(rm.getExtraParams());
            dp.put("topic", ep.getOrDefault("topic", "+/data"));
            dp.put("json_path", ep.getOrDefault("json_path", "$"));
            dp.put("qos", ep.getOrDefault("qos", 1));

            // 数据类型仍需(用于校验/存储)
            dp.put("data_type", rm.getDataType());
            dps.add(dp);
        }
        return dps;
    }

    /**
     * HTTP JSON 协议数据点 — 采集端定时 GET 并解析 JSON
     *
     * extra_params 预期格式:
     *   {"url": "http://192.168.1.100/api/data", "method": "GET", "json_path": "$.data.temp"}
     */
    private List<Map<String, Object>> buildHttpDataPoints(List<DevRegisterMap> registers) {
        List<Map<String, Object>> dps = new ArrayList<>();
        for (DevRegisterMap rm : registers) {
            Map<String, Object> dp = new LinkedHashMap<>();
            dp.put("sensor_code", rm.getSensorCode());
            dp.put("sensor_name", rm.getSensorName());
            dp.put("unit", nvl(rm.getUnit(), ""));
            dp.put("data_type", rm.getDataType());

            Map<String, Object> ep = parseExtraParams(rm.getExtraParams());
            dp.put("url", ep.getOrDefault("url", ""));
            dp.put("method", ep.getOrDefault("method", "GET"));
            dp.put("json_path", ep.getOrDefault("json_path", "$"));

            dps.add(dp);
        }
        return dps;
    }

    /**
     * OPC UA 协议数据点
     *
     * extra_params 预期格式:
     *   {"node_id": "ns=2;s=Temperature", "namespace_index": 2}
     */
    private List<Map<String, Object>> buildOpcUaDataPoints(List<DevRegisterMap> registers) {
        List<Map<String, Object>> dps = new ArrayList<>();
        for (DevRegisterMap rm : registers) {
            Map<String, Object> dp = new LinkedHashMap<>();
            dp.put("sensor_code", rm.getSensorCode());
            dp.put("sensor_name", rm.getSensorName());
            dp.put("unit", nvl(rm.getUnit(), ""));
            dp.put("data_type", rm.getDataType());

            Map<String, Object> ep = parseExtraParams(rm.getExtraParams());
            dp.put("node_id", ep.getOrDefault("node_id", ""));
            dp.put("namespace_index", ep.getOrDefault("namespace_index", 0));

            dps.add(dp);
        }
        return dps;
    }

    // ══════════════════════════════════════════
    // 告警配置
    // ══════════════════════════════════════════

    @SuppressWarnings("unchecked")
    private List<Map<String, Object>> buildAlarmConfig(Long deviceId) {
        List<DevDeviceAlarmConfig> alarms = alarmConfigMapper.selectByDeviceId(deviceId)
            .stream().filter(a -> a.getAlarmEnabled() == 1).toList();
        List<Map<String, Object>> alarmList = new ArrayList<>();
        for (DevDeviceAlarmConfig ac : alarms) {
            Map<String, Object> acm = new LinkedHashMap<>();
            acm.put("sensor_code", ac.getSensorCode());
            acm.put("alarm_type", ac.getAlarmType());
            acm.put("enabled", true);
            acm.put("level", ac.getAlarmLevel());
            try { acm.put("params", objectMapper.readTree(ac.getParams())); }
            catch (Exception e) { acm.put("params", ac.getParams()); }
            alarmList.add(acm);
        }
        return alarmList;
    }

    // ══════════════════════════════════════════
    // 工具方法
    // ══════════════════════════════════════════

    @SuppressWarnings("unchecked")
    private Map<String, Object> parseExtraParams(String extraParams) {
        if (extraParams == null || extraParams.isBlank()) return Collections.emptyMap();
        try { return objectMapper.readValue(extraParams, Map.class); }
        catch (Exception e) { return Collections.emptyMap(); }
    }

    private void putExtraParams(Map<String, Object> dp, String extraParams) {
        if (extraParams == null || extraParams.isBlank()) return;
        try { dp.put("extra_params", objectMapper.readTree(extraParams)); }
        catch (Exception e) { dp.put("extra_params", extraParams); }
    }

    @SuppressWarnings("unchecked")
    private static <T> T nvl(T value, T defaultValue) {
        return value != null ? value : defaultValue;
    }
}
