CREATE TABLE dev_device_reading (
    id             BIGINT AUTO_INCREMENT,
    device_id      BIGINT       NOT NULL,
    register_id    BIGINT       NOT NULL,
    sensor_code    VARCHAR(64)  NOT NULL,
    value          DECIMAL(20,6)         DEFAULT NULL,
    avg            DECIMAL(20,6)         DEFAULT NULL,
    max            DECIMAL(20,6)         DEFAULT NULL,
    min            DECIMAL(20,6)         DEFAULT NULL,
    sample_count   INT          NOT NULL DEFAULT 1,
    quality        TINYINT      NOT NULL DEFAULT 0 COMMENT '0=正常 1=超时 2=异常 3=传感器故障',
    read_at        DATETIME     NOT NULL,
    window_start   DATETIME              DEFAULT NULL,
    window_end     DATETIME              DEFAULT NULL,
    created_at     DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id, read_at),
    INDEX idx_device_time (device_id, read_at),
    INDEX idx_register_time (register_id, read_at),
    INDEX idx_device_register (device_id, register_id, read_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
PARTITION BY RANGE (TO_DAYS(read_at)) (
    PARTITION p202606 VALUES LESS THAN (TO_DAYS('2026-07-01')),
    PARTITION p202607 VALUES LESS THAN (TO_DAYS('2026-08-01')),
    PARTITION p202608 VALUES LESS THAN (TO_DAYS('2026-09-01')),
    PARTITION p_future VALUES LESS THAN MAXVALUE
);
