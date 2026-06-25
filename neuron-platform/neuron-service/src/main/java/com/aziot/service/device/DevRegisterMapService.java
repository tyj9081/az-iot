package com.aziot.service.device;

import com.aziot.common.exception.BusinessException;
import com.aziot.dao.entity.device.DevRegisterMap;
import com.aziot.dao.mapper.device.DevRegisterMapMapper;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;

@Service
public class DevRegisterMapService extends ServiceImpl<DevRegisterMapMapper, DevRegisterMap> {

    public List<DevRegisterMap> listByModelId(Long modelId) {
        LambdaQueryWrapper<DevRegisterMap> qw = new LambdaQueryWrapper<>();
        qw.eq(DevRegisterMap::getModelId, modelId);
        qw.orderByAsc(DevRegisterMap::getSortOrder);
        return list(qw);
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
            DevRegisterMap dup = getOne(new LambdaQueryWrapper<DevRegisterMap>()
                    .eq(DevRegisterMap::getModelId, existing.getModelId())
                    .eq(DevRegisterMap::getSensorCode, register.getSensorCode())
                    .ne(DevRegisterMap::getId, id));
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
        // 清除旧数据
        remove(new LambdaQueryWrapper<DevRegisterMap>()
                .eq(DevRegisterMap::getModelId, modelId));
        // 批量写入新数据
        for (DevRegisterMap item : list) {
            item.setModelId(modelId);
            save(item);
        }
    }

    private boolean existsByModelIdAndSensorCode(Long modelId, String sensorCode) {
        return getOne(new LambdaQueryWrapper<DevRegisterMap>()
                .eq(DevRegisterMap::getModelId, modelId)
                .eq(DevRegisterMap::getSensorCode, sensorCode)) != null;
    }
}
