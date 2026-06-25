CREATE TABLE sys_role_permission (
    id              BIGINT AUTO_INCREMENT PRIMARY KEY,
    role_id         BIGINT       NOT NULL,
    permission_id   BIGINT       NOT NULL,
    created_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_role_perm (role_id, permission_id),
    INDEX idx_role_perm_role (role_id),
    INDEX idx_role_perm_perm (permission_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
