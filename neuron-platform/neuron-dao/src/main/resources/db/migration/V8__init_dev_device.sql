CREATE TABLE dev_device (
    id                   BIGINT AUTO_INCREMENT PRIMARY KEY,
    serial_port_id       BIGINT       NOT NULL,
    model_id             BIGINT       NOT NULL,
    code                 VARCHAR(64)  NOT NULL UNIQUE,
    name                 VARCHAR(128) NOT NULL,
    slave_addr           INT          NOT NULL DEFAULT 1,
    collect_interval_sec INT                   DEFAULT NULL,
    status               TINYINT      NOT NULL DEFAULT 0 COMMENT '0=离线 1=在线 2=告警 3=停用',
    location             VARCHAR(256)          DEFAULT NULL,
    description          VARCHAR(512)          DEFAULT NULL,
    is_deleted           TINYINT      NOT NULL DEFAULT 0,
    created_at           DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at           DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    CONSTRAINT fk_device_port FOREIGN KEY (serial_port_id) REFERENCES dev_serial_port(id),
    CONSTRAINT fk_device_model FOREIGN KEY (model_id) REFERENCES dev_device_model(id),
    INDEX idx_device_port (serial_port_id),
    INDEX idx_device_model (model_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
