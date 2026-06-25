CREATE TABLE dev_collector (
    id               BIGINT AUTO_INCREMENT PRIMARY KEY,
    code             VARCHAR(32)  NOT NULL UNIQUE,
    name             VARCHAR(128) NOT NULL,
    type             VARCHAR(32)  NOT NULL DEFAULT 'BC-U101',
    mqtt_client_id   VARCHAR(64)  NOT NULL UNIQUE,
    ip_address       VARCHAR(45)           DEFAULT NULL,
    status           TINYINT      NOT NULL DEFAULT 0 COMMENT '0=离线 1=在线 2=告警',
    firmware_version VARCHAR(32)           DEFAULT NULL,
    last_heartbeat   DATETIME              DEFAULT NULL,
    description      VARCHAR(512)          DEFAULT NULL,
    created_at       DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at       DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
