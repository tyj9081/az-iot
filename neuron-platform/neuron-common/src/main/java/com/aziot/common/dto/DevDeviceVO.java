package com.aziot.common.dto;

import lombok.Data;

import java.time.LocalDateTime;

@Data
public class DevDeviceVO {
    private Long id;
    private Long serialPortId;
    private Long collectorId;
    private Long modelId;
    private String code;
    private String name;
    private Integer slaveAddr;
    private Integer collectIntervalSec;
    private String status;
    private String location;
    private String description;
    private LocalDateTime createdAt;
    private LocalDateTime updatedAt;

    private String serialPortName;
    private String collectorName;
    private String modelName;
    private Long protocolId;
    private String protocolName;
}
