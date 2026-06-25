package com.aziot.service.device;

import com.aziot.common.exception.BusinessException;
import com.aziot.dao.entity.device.DevDeviceModel;
import com.aziot.dao.entity.device.DevManufacturer;
import com.aziot.dao.mapper.device.DevDeviceModelMapper;
import com.aziot.dao.mapper.device.DevManufacturerMapper;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

@Service
public class DevManufacturerService extends ServiceImpl<DevManufacturerMapper, DevManufacturer> {

    private final DevDeviceModelMapper devDeviceModelMapper;

    public DevManufacturerService(DevDeviceModelMapper devDeviceModelMapper) {
        this.devDeviceModelMapper = devDeviceModelMapper;
    }

    public Page<DevManufacturer> page(int page, int pageSize, String keyword) {
        LambdaQueryWrapper<DevManufacturer> qw = new LambdaQueryWrapper<>();
        if (keyword != null && !keyword.isBlank()) {
            qw.and(w -> w.like(DevManufacturer::getName, keyword)
                    .or().like(DevManufacturer::getCode, keyword));
        }
        qw.orderByAsc(DevManufacturer::getId);
        return page(new Page<>(page, pageSize), qw);
    }

    public DevManufacturer getById(Long id) {
        DevManufacturer manufacturer = super.getById(id);
        if (manufacturer == null) {
            throw BusinessException.notFound("制造商");
        }
        return manufacturer;
    }

    @Transactional
    public void create(DevManufacturer manufacturer) {
        if (existsByCode(manufacturer.getCode())) {
            throw new BusinessException(409, "制造商编码已存在");
        }
        save(manufacturer);
    }

    @Transactional
    public void update(Long id, DevManufacturer manufacturer) {
        getById(id);
        DevManufacturer exist = getOne(new LambdaQueryWrapper<DevManufacturer>()
                .eq(DevManufacturer::getCode, manufacturer.getCode())
                .ne(DevManufacturer::getId, id));
        if (exist != null) {
            throw new BusinessException(409, "制造商编码已存在");
        }
        manufacturer.setId(id);
        updateById(manufacturer);
    }

    @Transactional
    public void delete(Long id) {
        getById(id);
        long count = devDeviceModelMapper.selectCount(
                new LambdaQueryWrapper<DevDeviceModel>()
                        .eq(DevDeviceModel::getManufacturerId, id));
        if (count > 0) {
            throw new BusinessException("该制造商下存在关联型号，无法删除");
        }
        removeById(id);
    }

    private boolean existsByCode(String code) {
        return getOne(new LambdaQueryWrapper<DevManufacturer>()
                .eq(DevManufacturer::getCode, code)) != null;
    }
}
