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
    private final DevDeviceAlarmConfigMapper alarmConfigMapper;
    private final MqttPublisherService mqttPublisher;
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

            // 追加告警配置
            List<DevDeviceAlarmConfig> alarms = alarmConfigMapper.selectList(
                new LambdaQueryWrapper<DevDeviceAlarmConfig>()
                    .eq(DevDeviceAlarmConfig::getDeviceId, device.getId())
                    .eq(DevDeviceAlarmConfig::getAlarmEnabled, 1));

            List<Map<String, Object>> alarmList = new ArrayList<>();
            for (DevDeviceAlarmConfig ac : alarms) {
                Map<String, Object> acm = new LinkedHashMap<>();
                acm.put("sensor_code", ac.getSensorCode());
                acm.put("alarm_type", ac.getAlarmType());
                acm.put("enabled", true);
                acm.put("level", ac.getAlarmLevel());
                acm.put("params", objectMapper.readTree(ac.getParams()));
                alarmList.add(acm);
            }
            deviceNode.put("alarm_config", alarmList);

            payload.put("device", deviceNode);

            String json = objectMapper.writeValueAsString(payload);
            String topic = "neuron/" + collector.getMqttClientId() + "/config/delta";
            mqttPublisher.publish(topic, json);
            log.info("MQTT CONFIG PUSH topic={} deviceId={} action={}", topic, deviceId, action);
        } catch (Exception e) {
            log.error("Config push failed for device " + deviceId, e);
        }
    }
}
