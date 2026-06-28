package com.aziot.dao.mapper.collector;

import com.aziot.dao.entity.collector.DevCollector;
import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import org.apache.ibatis.annotations.Mapper;
import org.apache.ibatis.annotations.Param;

@Mapper
public interface DevCollectorMapper extends BaseMapper<DevCollector> {

    /** 分页条件查询采集器 */
    Page<DevCollector> selectPageByCondition(
        Page<DevCollector> page,
        @Param("status") String status,
        @Param("keyword") String keyword
    );

    /** 按编码查询（排除指定ID） */
    DevCollector selectByCodeExcludeId(@Param("code") String code, @Param("excludeId") Long excludeId);

    /** 按编码查询 */
    DevCollector selectByCode(@Param("code") String code);

    /** 按MQTT用户名查询 */
    DevCollector selectByMqttUsername(@Param("mqttUsername") String mqttUsername);
}
