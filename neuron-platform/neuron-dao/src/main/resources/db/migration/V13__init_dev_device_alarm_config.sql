DROP TABLE IF EXISTS dev_device_alarm_config;
CREATE TABLE dev_device_alarm_config (
    id              BIGINT AUTO_INCREMENT PRIMARY KEY,
    device_id       BIGINT       NOT NULL COMMENT '设备ID',
    sensor_code     VARCHAR(64)  NOT NULL COMMENT '采集点编码',
    alarm_enabled   TINYINT      NOT NULL DEFAULT 1 COMMENT '是否启用告警',
    alarm_type      VARCHAR(32)  NOT NULL DEFAULT 'limit_upper' COMMENT '告警类型',
    params          JSON         NOT NULL COMMENT '告警参数(按类型不同)',
    alarm_level     VARCHAR(16)  NOT NULL DEFAULT 'warning' COMMENT '告警等级: info/warning/critical',
    description     VARCHAR(256)          DEFAULT NULL COMMENT '告警描述',
    created_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_device_sensor_type (device_id, sensor_code, alarm_type),
    INDEX idx_alarm_device (device_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
