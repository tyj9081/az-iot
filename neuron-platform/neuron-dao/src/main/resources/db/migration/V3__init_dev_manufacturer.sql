CREATE TABLE dev_manufacturer (
    id              BIGINT AUTO_INCREMENT PRIMARY KEY,
    code            VARCHAR(32)  NOT NULL UNIQUE,
    name            VARCHAR(128) NOT NULL,
    country         VARCHAR(64)           DEFAULT NULL,
    website         VARCHAR(256)          DEFAULT NULL,
    description     VARCHAR(512)          DEFAULT NULL,
    is_deleted      TINYINT      NOT NULL DEFAULT 0,
    created_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
