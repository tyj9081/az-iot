package com.aziot.service.collector;

import com.aziot.common.exception.BusinessException;
import com.aziot.dao.entity.collector.DevSerialPort;
import com.aziot.dao.mapper.collector.DevSerialPortMapper;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;
import org.springframework.stereotype.Service;

import java.util.List;

@Service
public class DevSerialPortService extends ServiceImpl<DevSerialPortMapper, DevSerialPort> {

    public List<DevSerialPort> listByCollectorId(Long collectorId) {
        return list(new LambdaQueryWrapper<DevSerialPort>()
                .eq(DevSerialPort::getCollectorId, collectorId)
                .orderByAsc(DevSerialPort::getId));
    }

    public void update(Long id, DevSerialPort port) {
        DevSerialPort exist = super.getById(id);
        if (exist == null) {
            throw BusinessException.notFound("串口");
        }
        exist.setPortLabel(port.getPortLabel());
        exist.setBusParam(port.getBusParam());
        exist.setIsActive(port.getIsActive());
        updateById(exist);
    }

    public void update(Long collectorId, Long id, DevSerialPort port) {
        DevSerialPort exist = super.getById(id);
        if (exist == null || !collectorId.equals(exist.getCollectorId())) {
            throw BusinessException.notFound("串口");
        }
        update(id, port);
    }
}
