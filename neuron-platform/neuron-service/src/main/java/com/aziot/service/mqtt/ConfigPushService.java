package com.aziot.service.mqtt;

import com.aziot.dao.entity.collector.DevCollector;
import com.aziot.dao.entity.collector.DevSerialPort;
import com.aziot.dao.entity.device.*;
import com.aziot.dao.mapper.collector.DevCollectorMapper;
import com.aziot.dao.mapper.collector.DevSerialPortMapper;
import com.aziot.dao.mapper.device.*;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.fasterxml.jackson.databind.ObjectMapper;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import java.util.*;

@Slf4j
@Service
@RequiredArgsConstructor
public class ConfigPushService {
    private final DevDeviceMapper deviceMapper;
    private final DevSerialPortMapper serialPortMapper;
    private final DevCollectorMapper collectorMapper;
    private final DevDeviceModelMapper modelMapper;
    private final DevRegisterMapMapper registerMapMapper;
    private final ObjectMapper objectMapper;

    public void pushDelta(Long deviceId, String action) {
        try {
            DevDevice device = deviceMapper.selectById(deviceId);
            if (device == null) return;
            DevSerialPort port = serialPortMapper.selectById(device.getSerialPortId());
            if (port == null) return;
            DevCollector collector = collectorMapper.selectById(port.getCollectorId());
            if (collector == null) return;
            DevDeviceModel model = modelMapper.selectById(device.getModelId());
            List<DevRegisterMap> registers = registerMapMapper.selectList(
                new LambdaQueryWrapper<DevRegisterMap>()
                    .eq(DevRegisterMap::getModelId, device.getModelId()));

            Map<String, Object> payload = new LinkedHashMap<>();
            payload.put("version", System.currentTimeMillis());
            payload.put("action", action);

            Map<String, Object> deviceNode = new LinkedHashMap<>();
            deviceNode.put("id", device.getId());
            deviceNode.put("code", device.getCode());
            deviceNode.put("name", device.getName());
            deviceNode.put("protocol", model.getProtocolId());
            deviceNode.put("slave_addr", device.getSlaveAddr());

            Map<String, Object> bus = new LinkedHashMap<>();
            bus.put("port_name", port.getPortName());
            bus.put("bus_type", port.getBusType());
            bus.put("bus_param", objectMapper.readTree(port.getBusParam()));
            deviceNode.put("serial_port", bus);

            List<Map<String, Object>> dps = new ArrayList<>();
            for (DevRegisterMap rm : registers) {
                Map<String, Object> dp = new LinkedHashMap<>();
                dp.put("sensor_code", rm.getSensorCode());
                dp.put("sensor_name", rm.getSensorName());
                dp.put("register_address", rm.getRegisterAddress());
                dp.put("register_count", rm.getRegisterCount());
                dp.put("data_type", rm.getDataType());
                dp.put("byte_order", rm.getByteOrder());
                dp.put("func_code", rm.getFuncCode());
                dp.put("coefficient", rm.getCoefficient());
                dp.put("offset", rm.getOffsetVal());
                dp.put("unit", rm.getUnit());
                dps.add(dp);
            }
            deviceNode.put("data_points", dps);
            deviceNode.put("collect_interval_sec", device.getCollectIntervalSec());
            payload.put("device", deviceNode);

            String json = objectMapper.writeValueAsString(payload);
            String topic = "neuron/" + collector.getMqttClientId() + "/config/delta";
            log.info("MQTT PUBLISH topic={} payload={}", topic, json);
        } catch (Exception e) {
            log.error("Config push failed for device " + deviceId, e);
        }
    }
}
