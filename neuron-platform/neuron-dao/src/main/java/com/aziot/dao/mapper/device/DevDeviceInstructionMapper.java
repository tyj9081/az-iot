package com.aziot.dao.mapper.device;

import com.aziot.dao.entity.device.DevDeviceInstruction;
import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import org.apache.ibatis.annotations.Mapper;
import org.apache.ibatis.annotations.Param;
import java.util.List;

@Mapper
public interface DevDeviceInstructionMapper extends BaseMapper<DevDeviceInstruction> {

    /** 按设备ID查询指令，按sort_order升序 */
    List<DevDeviceInstruction> selectByDeviceId(@Param("deviceId") Long deviceId);

    /** 按设备ID+指令编码查询 */
    DevDeviceInstruction selectByDeviceIdAndCode(
        @Param("deviceId") Long deviceId,
        @Param("instructionCode") String instructionCode
    );

    /** 按设备ID+指令编码查询（排除指定ID） */
    DevDeviceInstruction selectByDeviceIdAndCodeExcludeId(
        @Param("deviceId") Long deviceId,
        @Param("instructionCode") String instructionCode,
        @Param("excludeId") Long excludeId
    );
}
