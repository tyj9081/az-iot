CREATE TABLE dev_device_model (
    id                   BIGINT AUTO_INCREMENT PRIMARY KEY,
    manufacturer_id      BIGINT       NOT NULL,
    protocol_id          BIGINT       NOT NULL,
    code                 VARCHAR(64)  NOT NULL UNIQUE,
    name                 VARCHAR(128) NOT NULL,
    collect_interval_sec INT          NOT NULL DEFAULT 900,
    description          VARCHAR(512)          DEFAULT NULL,
    is_enabled           TINYINT      NOT NULL DEFAULT 1,
    created_at           DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at           DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    CONSTRAINT fk_model_manufacturer FOREIGN KEY (manufacturer_id) REFERENCES dev_manufacturer(id),
    CONSTRAINT fk_model_protocol FOREIGN KEY (protocol_id) REFERENCES dev_protocol(id),
    INDEX idx_model_manufacturer (manufacturer_id),
    INDEX idx_model_protocol (protocol_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
