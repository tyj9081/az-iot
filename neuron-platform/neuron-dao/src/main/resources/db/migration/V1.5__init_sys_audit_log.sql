CREATE TABLE sys_audit_log (
    id              BIGINT AUTO_INCREMENT PRIMARY KEY,
    module          VARCHAR(64)  NOT NULL,
    action          VARCHAR(128) NOT NULL,
    operator_id     BIGINT                DEFAULT NULL,
    operator_name   VARCHAR(64)           DEFAULT NULL,
    request_ip      VARCHAR(64)           DEFAULT NULL,
    request_method  VARCHAR(16)           DEFAULT NULL,
    request_url     VARCHAR(256)          DEFAULT NULL,
    request_params  TEXT                  DEFAULT NULL,
    response_result TEXT                  DEFAULT NULL,
    cost_ms         INT                   DEFAULT 0,
    status          TINYINT              DEFAULT 1 COMMENT '1=成功 0=失败',
    created_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_audit_log_time (created_at),
    INDEX idx_audit_log_operator (operator_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
