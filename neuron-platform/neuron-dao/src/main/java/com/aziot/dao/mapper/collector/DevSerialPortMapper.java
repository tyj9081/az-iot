package com.aziot.dao.mapper.collector;

import com.aziot.dao.entity.collector.DevSerialPort;
import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import org.apache.ibatis.annotations.Mapper;
import org.apache.ibatis.annotations.Param;
import java.util.List;

@Mapper
public interface DevSerialPortMapper extends BaseMapper<DevSerialPort> {

    /** 按采集器ID查询串口列表，按ID升序 */
    List<DevSerialPort> selectByCollectorId(@Param("collectorId") Long collectorId);

    /** 统计指定采集器下的串口数 */
    long countByCollectorId(@Param("collectorId") Long collectorId);
}
