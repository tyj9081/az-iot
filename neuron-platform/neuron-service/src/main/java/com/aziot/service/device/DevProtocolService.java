package com.aziot.service.device;

import com.aziot.dao.entity.device.DevProtocol;
import com.aziot.dao.mapper.device.DevProtocolMapper;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;
import org.springframework.stereotype.Service;

import java.util.List;

@Service
public class DevProtocolService extends ServiceImpl<DevProtocolMapper, DevProtocol> {

    public List<DevProtocol> listAll() {
        LambdaQueryWrapper<DevProtocol> qw = new LambdaQueryWrapper<>();
        qw.eq(DevProtocol::getIsEnabled, 1);
        qw.orderByAsc(DevProtocol::getId);
        return list(qw);
    }
}
