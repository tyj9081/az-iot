CREATE TABLE sys_config (
    id           BIGINT AUTO_INCREMENT PRIMARY KEY,
    config_key   VARCHAR(64)  NOT NULL UNIQUE,
    config_value VARCHAR(256) NOT NULL,
    description  VARCHAR(256)          DEFAULT NULL,
    updated_at   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO sys_config (config_key, config_value, description) VALUES
('data_retention_days', '90', '读数保留天数'),
('audit_log_retention_days', '180', '审计日志保留天数'),
('default_collect_interval_sec', '600', '全局默认采集间隔(秒)');
