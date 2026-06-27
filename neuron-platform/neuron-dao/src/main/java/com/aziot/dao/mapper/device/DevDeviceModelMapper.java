package com.aziot.dao.mapper.device;

import com.aziot.dao.entity.device.DevDeviceModel;
import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import com.baomidou.mybatisplus.core.metadata.IPage;
import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import org.apache.ibatis.annotations.Mapper;
import org.apache.ibatis.annotations.Param;

@Mapper
public interface DevDeviceModelMapper extends BaseMapper<DevDeviceModel> {

    /** 分页条件查询型号 */
    IPage<DevDeviceModel> selectPageByCondition(
        Page<DevDeviceModel> page,
        @Param("manufacturerId") Long manufacturerId,
        @Param("protocolId") Long protocolId,
        @Param("keyword") String keyword
    );

    /** 按编码查询（排除指定ID） */
    DevDeviceModel selectByCodeExcludeId(@Param("code") String code, @Param("excludeId") Long excludeId);

    /** 按编码查询 */
    DevDeviceModel selectByCode(@Param("code") String code);

    /** 统计指定型号下的设备数 */
    long countByModelId(@Param("modelId") Long modelId);
}
