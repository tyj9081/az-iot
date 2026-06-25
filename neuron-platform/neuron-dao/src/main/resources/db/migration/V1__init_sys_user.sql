CREATE TABLE sys_user (
    id              BIGINT AUTO_INCREMENT PRIMARY KEY,
    username        VARCHAR(64)  NOT NULL UNIQUE,
    password_hash   VARCHAR(256) NOT NULL,
    nickname        VARCHAR(128) NOT NULL DEFAULT '',
    email           VARCHAR(128)          DEFAULT NULL,
    phone           VARCHAR(32)           DEFAULT NULL,
    avatar_url      VARCHAR(512)          DEFAULT NULL,
    status          TINYINT      NOT NULL DEFAULT 1 COMMENT '1=启用 0=禁用',
    is_deleted      TINYINT      NOT NULL DEFAULT 0,
    last_login_time DATETIME              DEFAULT NULL,
    created_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 初始管理员: admin / admin123 (BCrypt)
INSERT INTO sys_user (username, password_hash, nickname, status) VALUES
('admin', '$2a$10$N.zmdr9k7uOCQb376NoUnuTJ8iAt6Z5EHsM8lE9lBOsl7iKTVKIUi', '超级管理员', 1);
