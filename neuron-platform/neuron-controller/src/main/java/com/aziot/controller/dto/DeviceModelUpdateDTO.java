package com.aziot.controller.dto;

import lombok.Data;

@Data
public class DeviceModelUpdateDTO {
    private Long manufacturerId;
    private Long protocolId;
    private String code;
    private String name;
    private String description;
}
