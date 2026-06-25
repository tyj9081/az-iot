CREATE TABLE dev_serial_port (
    id           BIGINT AUTO_INCREMENT PRIMARY KEY,
    collector_id BIGINT       NOT NULL,
    port_name    VARCHAR(16)  NOT NULL,
    port_label   VARCHAR(64)           DEFAULT NULL,
    bus_type     VARCHAR(16)  NOT NULL DEFAULT 'serial',
    bus_param    JSON         NOT NULL,
    port_type    VARCHAR(16)  NOT NULL DEFAULT 'device' COMMENT 'device/io_board/sms_modem',
    is_active    TINYINT      NOT NULL DEFAULT 1,
    created_at   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_collector_port (collector_id, port_name),
    CONSTRAINT fk_port_collector FOREIGN KEY (collector_id) REFERENCES dev_collector(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
