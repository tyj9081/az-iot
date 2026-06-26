package com.aziot.service.device;

import com.aziot.common.exception.BusinessException;
import com.aziot.dao.entity.device.DevDeviceInstruction;
import com.aziot.dao.mapper.device.DevDeviceInstructionMapper;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;

@Service
public class DevDeviceInstructionService extends ServiceImpl<DevDeviceInstructionMapper, DevDeviceInstruction> {

    public List<DevDeviceInstruction> listByDeviceId(Long deviceId) {
        return list(new LambdaQueryWrapper<DevDeviceInstruction>()
                .eq(DevDeviceInstruction::getDeviceId, deviceId)
                .orderByAsc(DevDeviceInstruction::getSortOrder));
    }

    @Transactional
    public void create(DevDeviceInstruction instr) {
        if (existsByCode(instr.getDeviceId(), instr.getInstructionCode())) {
            throw new BusinessException(409, "该设备下指令编码已存在");
        }
        save(instr);
    }

    @Transactional
    public void update(Long id, DevDeviceInstruction instr) {
        DevDeviceInstruction exist = getById(id);
        if (exist == null) {
            throw BusinessException.notFound("设备指令");
        }
        DevDeviceInstruction dup = getOne(new LambdaQueryWrapper<DevDeviceInstruction>()
                .eq(DevDeviceInstruction::getDeviceId, exist.getDeviceId())
                .eq(DevDeviceInstruction::getInstructionCode, instr.getInstructionCode())
                .ne(DevDeviceInstruction::getId, id));
        if (dup != null) {
            throw new BusinessException(409, "该设备下指令编码已存在");
        }
        instr.setId(id);
        updateById(instr);
    }

    @Transactional
    public void delete(Long id) {
        if (getById(id) == null) {
            throw BusinessException.notFound("设备指令");
        }
        removeById(id);
    }

    private boolean existsByCode(Long deviceId, String code) {
        return getOne(new LambdaQueryWrapper<DevDeviceInstruction>()
                .eq(DevDeviceInstruction::getDeviceId, deviceId)
                .eq(DevDeviceInstruction::getInstructionCode, code)) != null;
    }
}
