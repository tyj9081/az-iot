package com.aziot.service.device;

import com.aziot.common.exception.BusinessException;
import com.aziot.common.dto.DeviceModelVO;
import com.aziot.dao.entity.device.DevDevice;
import com.aziot.dao.entity.device.DevDeviceModel;
import com.aziot.dao.entity.device.DevManufacturer;
import com.aziot.dao.entity.device.DevProtocol;
import com.aziot.dao.entity.device.DevRegisterMap;
import com.aziot.dao.mapper.device.DevDeviceMapper;
import com.aziot.dao.mapper.device.DevDeviceModelMapper;
import com.aziot.dao.mapper.device.DevManufacturerMapper;
import com.aziot.dao.mapper.device.DevProtocolMapper;
import com.aziot.dao.mapper.device.DevRegisterMapMapper;
import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

@Service
public class DevDeviceModelService extends ServiceImpl<DevDeviceModelMapper, DevDeviceModel> {

    private final DevManufacturerMapper devManufacturerMapper;
    private final DevProtocolMapper devProtocolMapper;
    private final DevDeviceMapper devDeviceMapper;
    private final DevRegisterMapMapper devRegisterMapMapper;

    public DevDeviceModelService(DevManufacturerMapper devManufacturerMapper,
                                 DevProtocolMapper devProtocolMapper,
                                 DevDeviceMapper devDeviceMapper,
                                 DevRegisterMapMapper devRegisterMapMapper) {
        this.devManufacturerMapper = devManufacturerMapper;
        this.devProtocolMapper = devProtocolMapper;
        this.devDeviceMapper = devDeviceMapper;
        this.devRegisterMapMapper = devRegisterMapMapper;
    }

    public Page<DevDeviceModel> page(int page, int pageSize, Long manufacturerId, Long protocolId, String keyword) {
        return baseMapper.selectPageByCondition(new Page<>(page, pageSize), manufacturerId, protocolId, keyword);
    }

    public DeviceModelVO getById(Long id) {
        DevDeviceModel model = super.getById(id);
        if (model == null) {
            throw BusinessException.notFound("设备型号");
        }
        DeviceModelVO vo = new DeviceModelVO();
        copyProperties(model, vo);

        DevManufacturer manufacturer = devManufacturerMapper.selectById(model.getManufacturerId());
        if (manufacturer != null) {
            vo.setManufacturerName(manufacturer.getName());
        }
        DevProtocol protocol = devProtocolMapper.selectById(model.getProtocolId());
        if (protocol != null) {
            vo.setProtocolName(protocol.getName());
        }
        return vo;
    }

    @Transactional
    public void create(DevDeviceModel model) {
        if (existsByCode(model.getCode())) {
            throw new BusinessException(409, "型号编码已存在");
        }
        save(model);
    }

    @Transactional
    public void update(Long id, DevDeviceModel model) {
        getById(id);
        DevDeviceModel exist = baseMapper.selectByCodeExcludeId(model.getCode(), id);
        if (exist != null) {
            throw new BusinessException(409, "型号编码已存在");
        }
        model.setId(id);
        updateById(model);
    }

    @Transactional
    public void delete(Long id) {
        getById(id);
        long deviceCount = devDeviceMapper.countByModelId(id);
        if (deviceCount > 0) {
            throw new BusinessException("该型号下存在关联设备，无法删除");
        }
        long registerCount = devRegisterMapMapper.countByModelId(id);
        if (registerCount > 0) {
            throw new BusinessException("该型号下存在点表寄存器，无法删除");
        }
        removeById(id);
    }

    private boolean existsByCode(String code) {
        return baseMapper.selectByCode(code) != null;
    }

    private void copyProperties(DevDeviceModel src, DeviceModelVO dst) {
        dst.setId(src.getId());
        dst.setManufacturerId(src.getManufacturerId());
        dst.setProtocolId(src.getProtocolId());
        dst.setCode(src.getCode());
        dst.setName(src.getName());
        dst.setDescription(src.getDescription());
        dst.setIsEnabled(src.getIsEnabled());
        dst.setCreatedAt(src.getCreatedAt());
        dst.setUpdatedAt(src.getUpdatedAt());
    }
}
