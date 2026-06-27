package com.aziot.dao.mapper.device;

import com.aziot.dao.entity.device.DevProtocol;
import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import org.apache.ibatis.annotations.Mapper;

import java.util.List;

@Mapper
public interface DevProtocolMapper extends BaseMapper<DevProtocol> {

    /** 查询所有启用的协议，按ID升序 */
    List<DevProtocol> selectAllEnabled();
}
