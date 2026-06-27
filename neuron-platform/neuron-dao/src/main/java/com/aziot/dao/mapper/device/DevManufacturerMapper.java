package com.aziot.dao.mapper.device;

import com.aziot.dao.entity.device.DevManufacturer;
import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import com.baomidou.mybatisplus.core.metadata.IPage;
import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import org.apache.ibatis.annotations.Mapper;
import org.apache.ibatis.annotations.Param;

@Mapper
public interface DevManufacturerMapper extends BaseMapper<DevManufacturer> {

    /** 分页查询制造商，支持关键词模糊搜索 */
    IPage<DevManufacturer> selectPageByKeyword(Page<DevManufacturer> page, @Param("keyword") String keyword);

    /** 按编码查询（排除指定ID） */
    DevManufacturer selectByCodeExcludeId(@Param("code") String code, @Param("excludeId") Long excludeId);

    /** 按编码查询 */
    DevManufacturer selectByCode(@Param("code") String code);

    /** 统计指定制造商下的型号数量 */
    long countByManufacturerId(@Param("manufacturerId") Long manufacturerId);
}
