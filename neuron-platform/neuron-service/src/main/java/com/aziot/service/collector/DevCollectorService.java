package com.aziot.service.collector;

import com.aziot.common.exception.BusinessException;
import com.aziot.dao.entity.collector.DevCollector;
import com.aziot.dao.entity.collector.DevSerialPort;
import com.aziot.dao.mapper.collector.DevCollectorMapper;
import com.aziot.dao.mapper.collector.DevSerialPortMapper;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.security.SecureRandom;

@Service
public class DevCollectorService extends ServiceImpl<DevCollectorMapper, DevCollector> {

    private final DevSerialPortMapper devSerialPortMapper;
    private final PasswordEncoder passwordEncoder;

    public DevCollectorService(DevSerialPortMapper devSerialPortMapper,
                               PasswordEncoder passwordEncoder) {
        this.devSerialPortMapper = devSerialPortMapper;
        this.passwordEncoder = passwordEncoder;
    }

    public Page<DevCollector> page(int page, int pageSize, String status, String keyword) {
        LambdaQueryWrapper<DevCollector> qw = new LambdaQueryWrapper<>();
        if (status != null && !status.isBlank()) {
            qw.eq(DevCollector::getStatus, status);
        }
        if (keyword != null && !keyword.isBlank()) {
            qw.and(w -> w.like(DevCollector::getName, keyword)
                    .or().like(DevCollector::getCode, keyword));
        }
        qw.orderByAsc(DevCollector::getId);
        return page(new Page<>(page, pageSize), qw);
    }

    public DevCollector getById(Long id) {
        DevCollector collector = super.getById(id);
        if (collector == null) {
            throw BusinessException.notFound("采集器");
        }
        return collector;
    }

    @Transactional
    public void create(DevCollector collector) {
        createWithCredentials(collector);
    }

    @Transactional
    public String createWithCredentials(DevCollector collector) {
        if (existsByCode(collector.getCode())) {
            throw new BusinessException(409, "采集器编码已存在");
        }
        String rawPassword = generatePassword();
        collector.setMqttUsername(generateUsername());
        collector.setMqttPasswordHash(passwordEncoder.encode(rawPassword));
        if (collector.getMqttTlsEnabled() == null) collector.setMqttTlsEnabled(0);
        if (collector.getMqttBrokerHost() == null) collector.setMqttBrokerHost("localhost");
        if (collector.getMqttBrokerPort() == null) collector.setMqttBrokerPort(1883);

        save(collector);

        String busParam = "{\"baud\":9600,\"data_bits\":8,\"stop_bits\":1,\"parity\":\"none\"}";

        // COM1: sms_modem
        createSerialPort(collector.getId(), "COM1", "sms_modem", null, busParam);
        // COM2: io_board
        createSerialPort(collector.getId(), "COM2", "io_board", null, busParam);
        // COM3-COM4: device, serial
        for (int i = 3; i <= 4; i++) {
            createSerialPort(collector.getId(), "COM" + i, "device", "serial", busParam);
        }
        // COM5-COM10: device, serial
        for (int i = 5; i <= 10; i++) {
            createSerialPort(collector.getId(), "COM" + i, "device", "serial", busParam);
        }
        return rawPassword;
    }

    private void createSerialPort(Long collectorId, String portName, String portType, String busType, String busParam) {
        DevSerialPort port = new DevSerialPort();
        port.setCollectorId(collectorId);
        port.setPortName(portName);
        port.setPortLabel(portName);
        port.setPortType(portType);
        port.setBusType(busType);
        port.setBusParam(busParam);
        port.setIsActive(1);
        devSerialPortMapper.insert(port);
    }

    @Transactional
    public void update(Long id, DevCollector collector) {
        getById(id);
        DevCollector exist = getOne(new LambdaQueryWrapper<DevCollector>()
                .eq(DevCollector::getCode, collector.getCode())
                .ne(DevCollector::getId, id));
        if (exist != null) {
            throw new BusinessException(409, "采集器编码已存在");
        }
        collector.setId(id);
        updateById(collector);
    }

    @Transactional
    public void delete(Long id) {
        getById(id);
        long count = devSerialPortMapper.selectCount(
                new LambdaQueryWrapper<DevSerialPort>()
                        .eq(DevSerialPort::getCollectorId, id));
        if (count > 0) {
            throw new BusinessException("该采集器下存在关联串口，无法删除");
        }
        removeById(id);
    }

    private boolean existsByCode(String code) {
        return getOne(new LambdaQueryWrapper<DevCollector>()
                .eq(DevCollector::getCode, code)) != null;
    }

    private String generateUsername() {
        String chars = "abcdefghijklmnopqrstuvwxyz0123456789";
        SecureRandom random = new SecureRandom();
        StringBuilder sb = new StringBuilder("col-");
        for (int i = 0; i < 8; i++) {
            sb.append(chars.charAt(random.nextInt(chars.length())));
        }
        return sb.toString();
    }

    private String generatePassword() {
        String chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        SecureRandom random = new SecureRandom();
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < 16; i++) {
            sb.append(chars.charAt(random.nextInt(chars.length())));
        }
        return sb.toString();
    }
}
