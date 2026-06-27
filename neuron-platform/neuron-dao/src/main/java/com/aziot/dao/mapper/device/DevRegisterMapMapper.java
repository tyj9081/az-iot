package com.aziot.dao.mapper.device;

import com.aziot.dao.entity.device.DevRegisterMap;
import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import org.apache.ibatis.annotations.Mapper;
import org.apache.ibatis.annotations.Param;

import java.util.List;

@Mapper
public interface DevRegisterMapMapper extends BaseMapper<DevRegisterMap> {

    /** 按型号ID查询点表，按sort_order升序 */
    List<DevRegisterMap> selectByModelId(@Param("modelId") Long modelId);

    /** 按型号ID+传感器编码查询 */
    DevRegisterMap selectByModelIdAndSensorCode(@Param("modelId") Long modelId, @Param("sensorCode") String sensorCode);

    /** 按型号ID+传感器编码查询（排除指定ID） */
    DevRegisterMap selectByModelIdAndSensorCodeExcludeId(@Param("modelId") Long modelId, @Param("sensorCode") String sensorCode, @Param("excludeId") Long excludeId);

    /** 删除指定型号下所有点表 */
    int deleteByModelId(@Param("modelId") Long modelId);

    /** 统计指定型号下的点表数 */
    long countByModelId(@Param("modelId") Long modelId);

    /** 按传感器编码查询 */
    DevRegisterMap selectBySensorCode(@Param("sensorCode") String sensorCode);
}
