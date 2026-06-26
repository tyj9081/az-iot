package com.aziot.controller.dto;

import lombok.Data;

@Data
public class CollectorCreateDTO {
    private String code;
    private String name;
    private String type;
    private String mqttClientId;
    private String ipAddress;
    private Integer collectIntervalSec;
    private String firmwareVersion;
    private String description;
}
