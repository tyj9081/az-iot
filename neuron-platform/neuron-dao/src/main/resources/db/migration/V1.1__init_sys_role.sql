CREATE TABLE sys_role (
    id              BIGINT AUTO_INCREMENT PRIMARY KEY,
    role_code       VARCHAR(64)  NOT NULL UNIQUE,
    role_name       VARCHAR(128) NOT NULL,
    description     VARCHAR(256)          DEFAULT NULL,
    sort_order      INT          NOT NULL DEFAULT 0,
    status          TINYINT      NOT NULL DEFAULT 1,
    is_deleted      TINYINT      NOT NULL DEFAULT 0,
    created_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO sys_role (role_code, role_name, sort_order) VALUES
('admin', '管理员', 1),
('operator', '运维人员', 2),
('viewer', '查看者', 3);
