package com.aziot.controller.dto;

import lombok.Data;

import java.math.BigDecimal;

@Data
public class RegisterMapDTO {
    private String sensorCode;
    private String sensorName;
    private Integer registerAddress;
    private Integer registerCount;
    private String dataType;
    private String byteOrder;
    private String funcCode;
    private BigDecimal coefficient;
    private BigDecimal offsetVal;
    private String unit;
    private String rw;
    private Integer sortOrder;
    private String description;
}
