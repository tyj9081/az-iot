package com.aziot.controller.dto;

import lombok.Data;

@Data
public class DeviceCreateDTO {
    private Long serialPortId;
    private Long modelId;
    private String code;
    private String name;
    private Integer slaveAddr;
    private Integer collectIntervalSec;
    private String location;
    private String description;
}
