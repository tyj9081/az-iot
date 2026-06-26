package com.aziot.controller.dto;

import lombok.Data;

@Data
public class DeviceInstructionCreateDTO {
    private Long deviceId;
    private String instructionCode;
    private String instructionName;
    private String instructionType;
    private String funcCode;
    private Integer registerAddress;
    private Integer registerCount;
    private String params;
    private Integer sortOrder;
    private String description;
}
