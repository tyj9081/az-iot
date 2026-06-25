package com.aziot.controller.dto;

import lombok.Data;

@Data
public class DeviceModelCreateDTO {
    private Long manufacturerId;
    private Long protocolId;
    private String code;
    private String name;
    private Integer collectIntervalSec;
    private String description;
}
