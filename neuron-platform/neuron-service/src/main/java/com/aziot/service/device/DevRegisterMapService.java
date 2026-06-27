package com.aziot.service.device;

import com.aziot.common.exception.BusinessException;
import com.aziot.dao.entity.device.DevRegisterMap;
import com.aziot.dao.mapper.device.DevRegisterMapMapper;
import com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;

@Service
public class DevRegisterMapService extends ServiceImpl<DevRegisterMapMapper, DevRegisterMap> {

    public List<DevRegisterMap> listByModelId(Long modelId) {
        return baseMapper.selectByModelId(modelId);
    }

    public DevRegisterMap getById(Long id) {
        DevRegisterMap register = super.getById(id);
        if (register == null) {
            throw BusinessException.notFound("点表寄存器");
        }
        return register;
    }

    @Transactional
    public void create(DevRegisterMap register) {
        if (existsByModelIdAndSensorCode(register.getModelId(), register.getSensorCode())) {
            throw new BusinessException(409, "该型号下传感器编码已存在");
        }
        save(register);
    }

    @Transactional
    public void update(Long id, DevRegisterMap register) {
        DevRegisterMap existing = getById(id);
        if (!existing.getSensorCode().equals(register.getSensorCode())) {
            DevRegisterMap dup = baseMapper.selectByModelIdAndSensorCodeExcludeId(
                existing.getModelId(), register.getSensorCode(), id);
            if (dup != null) {
                throw new BusinessException(409, "该型号下传感器编码已存在");
            }
        }
        register.setId(id);
        updateById(register);
    }

    @Transactional
    public void delete(Long id) {
        getById(id);
        removeById(id);
    }

    @Transactional
    public void batchCreate(Long modelId, List<DevRegisterMap> list) {
        baseMapper.deleteByModelId(modelId);
        for (DevRegisterMap item : list) {
            item.setModelId(modelId);
            save(item);
        }
    }

    private boolean existsByModelIdAndSensorCode(Long modelId, String sensorCode) {
        return baseMapper.selectByModelIdAndSensorCode(modelId, sensorCode) != null;
    }
}
