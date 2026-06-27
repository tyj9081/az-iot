package com.aziot.service.device;

import com.aziot.common.dto.DevDeviceVO;
import com.aziot.common.exception.BusinessException;
import com.aziot.dao.entity.collector.DevCollector;
import com.aziot.dao.entity.collector.DevSerialPort;
import com.aziot.dao.entity.device.DevDevice;
import com.aziot.dao.entity.device.DevDeviceModel;
import com.aziot.dao.entity.device.DevProtocol;
import com.aziot.dao.mapper.collector.DevCollectorMapper;
import com.aziot.dao.mapper.collector.DevSerialPortMapper;
import com.aziot.dao.mapper.device.DevDeviceMapper;
import com.aziot.dao.mapper.device.DevDeviceModelMapper;
import com.aziot.dao.mapper.device.DevProtocolMapper;
import com.aziot.service.mqtt.ConfigPushService;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.*;
import java.util.function.Function;
import java.util.stream.Collectors;

@Service
public class DevDeviceService extends ServiceImpl<DevDeviceMapper, DevDevice> {

    private final DevSerialPortMapper devSerialPortMapper;
    private final DevCollectorMapper devCollectorMapper;
    private final DevDeviceModelMapper devDeviceModelMapper;
    private final DevProtocolMapper devProtocolMapper;
    private final ConfigPushService configPushService;

    public DevDeviceService(DevSerialPortMapper devSerialPortMapper,
                            DevCollectorMapper devCollectorMapper,
                            DevDeviceModelMapper devDeviceModelMapper,
                            DevProtocolMapper devProtocolMapper,
                            ConfigPushService configPushService) {
        this.devSerialPortMapper = devSerialPortMapper;
        this.devCollectorMapper = devCollectorMapper;
        this.devDeviceModelMapper = devDeviceModelMapper;
        this.devProtocolMapper = devProtocolMapper;
        this.configPushService = configPushService;
    }

    public Page<DevDevice> page(int page, int pageSize, Long collectorId, Long serialPortId, Long modelId, String status, String keyword) {
        LambdaQueryWrapper<DevDevice> qw = new LambdaQueryWrapper<>();
        if (serialPortId != null) {
            qw.eq(DevDevice::getSerialPortId, serialPortId);
        }
        if (collectorId != null) {
            List<Long> portIds = devSerialPortMapper.selectList(
                    new LambdaQueryWrapper<DevSerialPort>()
                            .eq(DevSerialPort::getCollectorId, collectorId)
                            .select(DevSerialPort::getId))
                    .stream().map(DevSerialPort::getId).collect(Collectors.toList());
            if (portIds.isEmpty()) {
                return new Page<>(page, pageSize);
            }
            qw.in(DevDevice::getSerialPortId, portIds);
        }
        if (modelId != null) {
            qw.eq(DevDevice::getModelId, modelId);
        }
        if (status != null && !status.isBlank()) {
            qw.eq(DevDevice::getStatus, status);
        }
        if (keyword != null && !keyword.isBlank()) {
            qw.and(w -> w.like(DevDevice::getName, keyword)
                    .or().like(DevDevice::getCode, keyword));
        }
        qw.orderByAsc(DevDevice::getId);
        return page(new Page<>(page, pageSize), qw);
    }

    /**
     * 分页查询设备列表，并自动填充关联的串口名、采集器名、型号名、协议名。
     */
    public Page<DevDeviceVO> pageWithDetails(int page, int pageSize, Long collectorId,
                                              Long serialPortId, Long modelId, String status, String keyword) {
        Page<DevDevice> devicePage = page(page, pageSize, collectorId, serialPortId, modelId, status, keyword);
        List<DevDevice> devices = devicePage.getRecords();
        if (devices.isEmpty()) {
            Page<DevDeviceVO> voPage = new Page<>(page, pageSize);
            voPage.setTotal(0);
            return voPage;
        }

        // 批量收集关联ID
        Set<Long> portIds = devices.stream().map(DevDevice::getSerialPortId).filter(Objects::nonNull).collect(Collectors.toSet());
        Set<Long> modelIds = devices.stream().map(DevDevice::getModelId).filter(Objects::nonNull).collect(Collectors.toSet());

        // 批量查询串口
        Map<Long, DevSerialPort> portMap = portIds.isEmpty() ? Collections.emptyMap() :
                devSerialPortMapper.selectBatchIds(portIds).stream()
                        .collect(Collectors.toMap(DevSerialPort::getId, Function.identity()));

        // 批量查询采集器（通过串口关联）
        Set<Long> collectorIds = portMap.values().stream()
                .map(DevSerialPort::getCollectorId).filter(Objects::nonNull).collect(Collectors.toSet());
        Map<Long, DevCollector> collectorMap = collectorIds.isEmpty() ? Collections.emptyMap() :
                devCollectorMapper.selectBatchIds(collectorIds).stream()
                        .collect(Collectors.toMap(DevCollector::getId, Function.identity()));

        // 批量查询型号
        Map<Long, DevDeviceModel> modelMap = modelIds.isEmpty() ? Collections.emptyMap() :
                devDeviceModelMapper.selectBatchIds(modelIds).stream()
                        .collect(Collectors.toMap(DevDeviceModel::getId, Function.identity()));

        // 批量查询协议（通过型号关联）
        Set<Long> protocolIds = modelMap.values().stream()
                .map(DevDeviceModel::getProtocolId).filter(Objects::nonNull).collect(Collectors.toSet());
        Map<Long, DevProtocol> protocolMap = protocolIds.isEmpty() ? Collections.emptyMap() :
                devProtocolMapper.selectBatchIds(protocolIds).stream()
                        .collect(Collectors.toMap(DevProtocol::getId, Function.identity()));

        // 组装 VO
        List<DevDeviceVO> voList = devices.stream().map(d -> {
            DevDeviceVO vo = new DevDeviceVO();
            vo.setId(d.getId());
            vo.setSerialPortId(d.getSerialPortId());
            vo.setModelId(d.getModelId());
            vo.setCode(d.getCode());
            vo.setName(d.getName());
            vo.setSlaveAddr(d.getSlaveAddr());
            vo.setCollectIntervalSec(d.getCollectIntervalSec());
            vo.setStatus(d.getStatus());
            vo.setLocation(d.getLocation());
            vo.setDescription(d.getDescription());
            vo.setCreatedAt(d.getCreatedAt());
            vo.setUpdatedAt(d.getUpdatedAt());

            DevSerialPort port = portMap.get(d.getSerialPortId());
            if (port != null) {
                vo.setSerialPortName(port.getPortName());
                vo.setCollectorId(port.getCollectorId());
                DevCollector collector = collectorMap.get(port.getCollectorId());
                if (collector != null) {
                    vo.setCollectorName(collector.getName());
                }
            }

            DevDeviceModel model = modelMap.get(d.getModelId());
            if (model != null) {
                vo.setModelName(model.getName());
                vo.setProtocolId(model.getProtocolId());
                DevProtocol protocol = protocolMap.get(model.getProtocolId());
                if (protocol != null) {
                    vo.setProtocolName(protocol.getName());
                }
            }

            return vo;
        }).collect(Collectors.toList());

        Page<DevDeviceVO> voPage = new Page<>(page, pageSize);
        voPage.setRecords(voList);
        voPage.setTotal(devicePage.getTotal());
        return voPage;
    }

    public DevDeviceVO getById(Long id) {
        DevDevice device = super.getById(id);
        if (device == null) {
            throw BusinessException.notFound("设备");
        }
        DevDeviceVO vo = new DevDeviceVO();
        vo.setId(device.getId());
        vo.setSerialPortId(device.getSerialPortId());
        vo.setModelId(device.getModelId());
        vo.setCode(device.getCode());
        vo.setName(device.getName());
        vo.setSlaveAddr(device.getSlaveAddr());
        vo.setCollectIntervalSec(device.getCollectIntervalSec());
        vo.setStatus(device.getStatus());
        vo.setLocation(device.getLocation());
        vo.setDescription(device.getDescription());
        vo.setCreatedAt(device.getCreatedAt());
        vo.setUpdatedAt(device.getUpdatedAt());

        DevSerialPort port = devSerialPortMapper.selectById(device.getSerialPortId());
        if (port != null) {
            vo.setSerialPortName(port.getPortName());
            vo.setCollectorId(port.getCollectorId());
            DevCollector collector = devCollectorMapper.selectById(port.getCollectorId());
            if (collector != null) {
                vo.setCollectorName(collector.getName());
            }
        }

        DevDeviceModel model = devDeviceModelMapper.selectById(device.getModelId());
        if (model != null) {
            vo.setModelName(model.getName());
            vo.setProtocolId(model.getProtocolId());
            DevProtocol protocol = devProtocolMapper.selectById(model.getProtocolId());
            if (protocol != null) {
                vo.setProtocolName(protocol.getName());
            }
        }

        return vo;
    }

    @Transactional
    public void create(DevDevice device) {
        if (existsByCode(device.getCode())) {
            throw new BusinessException(409, "设备编码已存在");
        }
        if (device.getStatus() == null || device.getStatus().isBlank()) {
            device.setStatus("offline");
        }
        save(device);
        configPushService.pushDelta(device.getId(), "add");
    }

    @Transactional
    public void update(Long id, DevDevice device) {
        if (super.getById(id) == null) {
            throw BusinessException.notFound("设备");
        }
        DevDevice exist = getOne(new LambdaQueryWrapper<DevDevice>()
                .eq(DevDevice::getCode, device.getCode())
                .ne(DevDevice::getId, id));
        if (exist != null) {
            throw new BusinessException(409, "设备编码已存在");
        }
        device.setId(id);
        updateById(device);
        configPushService.pushDelta(id, "update");
    }

    @Transactional
    public void delete(Long id) {
        if (super.getById(id) == null) {
            throw BusinessException.notFound("设备");
        }
        configPushService.pushDelta(id, "remove");
        removeById(id);
    }

    public void updateStatus(Long id, String status) {
        DevDevice device = super.getById(id);
        if (device == null) {
            throw BusinessException.notFound("设备");
        }
        if (!Set.of("offline", "online", "alarm", "disabled").contains(status)) {
            throw new BusinessException(400, "无效设备状态");
        }
        device.setStatus(status);
        updateById(device);
    }

    private boolean existsByCode(String code) {
        return getOne(new LambdaQueryWrapper<DevDevice>()
                .eq(DevDevice::getCode, code)) != null;
    }
}
