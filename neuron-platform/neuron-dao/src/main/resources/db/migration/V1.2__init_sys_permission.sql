CREATE TABLE sys_permission (
    id              BIGINT AUTO_INCREMENT PRIMARY KEY,
    parent_id       BIGINT       NOT NULL DEFAULT 0,
    perm_code       VARCHAR(128) NOT NULL UNIQUE,
    perm_name       VARCHAR(128) NOT NULL,
    perm_type       VARCHAR(16)  NOT NULL DEFAULT 'menu' COMMENT 'menu/button/api',
    path            VARCHAR(256)          DEFAULT NULL,
    icon            VARCHAR(64)           DEFAULT NULL,
    sort_order      INT          NOT NULL DEFAULT 0,
    status          TINYINT      NOT NULL DEFAULT 1,
    created_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
