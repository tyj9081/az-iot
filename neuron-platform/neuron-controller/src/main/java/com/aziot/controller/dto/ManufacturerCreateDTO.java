package com.aziot.controller.dto;

import lombok.Data;

@Data
public class ManufacturerCreateDTO {
    private String code;
    private String name;
    private String country;
    private String website;
    private String description;
}
