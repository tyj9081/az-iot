CREATE TABLE sys_user_role (
    id              BIGINT AUTO_INCREMENT PRIMARY KEY,
    user_id         BIGINT       NOT NULL,
    role_id         BIGINT       NOT NULL,
    created_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_user_role (user_id, role_id),
    INDEX idx_user_role_user (user_id),
    INDEX idx_user_role_role (role_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO sys_user_role (user_id, role_id) VALUES (1, 1);
