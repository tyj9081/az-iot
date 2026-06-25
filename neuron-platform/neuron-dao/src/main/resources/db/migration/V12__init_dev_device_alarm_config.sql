CREATE TABLE dev_device_alarm_config (
    id              BIGINT AUTO_INCREMENT PRIMARY KEY,
    device_id       BIGINT       NOT NULL COMMENT '设备ID',
    sensor_code     VARCHAR(64)  NOT NULL COMMENT '采集点编码',
    alarm_enabled   TINYINT      NOT NULL DEFAULT 1 COMMENT '是否启用告警: 1=启用, 0=禁用',
    min_value       DECIMAL(20,6)         DEFAULT NULL COMMENT '下限阈值 (NULL=不检查下限)',
    max_value       DECIMAL(20,6)         DEFAULT NULL COMMENT '上限阈值 (NULL=不检查上限)',
    hysteresis      DECIMAL(20,6)         DEFAULT 0 COMMENT '回滞值: 恢复需偏离阈值此值',
    delay_count     INT          NOT NULL DEFAULT 1 COMMENT '连续N次越限才触发告警(防抖)',
    alarm_level     VARCHAR(16)  NOT NULL DEFAULT 'warning' COMMENT '告警等级: info/warning/critical',
    description     VARCHAR(256)          DEFAULT NULL,
    created_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_device_sensor (device_id, sensor_code),
    INDEX idx_alarm_device (device_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
