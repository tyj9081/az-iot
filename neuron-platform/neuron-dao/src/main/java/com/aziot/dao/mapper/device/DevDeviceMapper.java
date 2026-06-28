package com.aziot.dao.mapper.device;

import com.aziot.dao.entity.device.DevDevice;
import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import org.apache.ibatis.annotations.Mapper;
import org.apache.ibatis.annotations.Param;

@Mapper
public interface DevDeviceMapper extends BaseMapper<DevDevice> {

    /** 分页条件查询设备 */
    Page<DevDevice> selectPageByCondition(
        Page<DevDevice> page,
        @Param("serialPortId") Long serialPortId,
        @Param("portIds") java.util.List<Long> portIds,
        @Param("modelId") Long modelId,
        @Param("status") String status,
        @Param("keyword") String keyword
    );

    /** 按编码查询（排除指定ID） */
    DevDevice selectByCodeExcludeId(@Param("code") String code, @Param("excludeId") Long excludeId);

    /** 按编码查询 */
    DevDevice selectByCode(@Param("code") String code);

    /** 统计指定型号下的设备数 */
    long countByModelId(@Param("modelId") Long modelId);

    /** 统计在线设备数 */
    long countByStatus(@Param("status") String status);

    /** 按采集器ID查所有设备（通过关联的串口） */
    @org.apache.ibatis.annotations.Select("SELECT d.* FROM dev_device d INNER JOIN dev_serial_port p ON d.serial_port_id = p.id WHERE p.collector_id = #{collectorId}")
    java.util.List<DevDevice> selectByCollectorId(@Param("collectorId") Long collectorId);
}
