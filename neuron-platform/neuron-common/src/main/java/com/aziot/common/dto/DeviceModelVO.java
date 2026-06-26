package com.aziot.common.dto;

import lombok.Data;

import java.time.LocalDateTime;

@Data
public class DeviceModelVO {
    private Long id;
    private Long manufacturerId;
    private Long protocolId;
    private String code;
    private String name;
    private String description;
    private Integer isEnabled;
    private LocalDateTime createdAt;
    private LocalDateTime updatedAt;
    private String manufacturerName;
    private String protocolName;
}
