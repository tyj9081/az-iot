-- V15: Create dev_device_instruction table
CREATE TABLE dev_device_instruction (
    id                BIGINT AUTO_INCREMENT PRIMARY KEY,
    device_id         BIGINT       NOT NULL,
    instruction_code  VARCHAR(64)  NOT NULL,
    instruction_name  VARCHAR(128) NOT NULL,
    instruction_type  VARCHAR(32)  NOT NULL DEFAULT 'READ' COMMENT 'READ/WRITE/CONTROL/CONFIG',
    func_code         VARCHAR(16)           DEFAULT '0x03' COMMENT 'e.g. 0x03=ReadHoldingReg',
    register_address  INT                   DEFAULT 0,
    register_count    INT                   DEFAULT 1,
    params            JSON                  DEFAULT NULL COMMENT 'extra params (dataType,byteOrder,etc.)',
    sort_order        INT          NOT NULL DEFAULT 0,
    description       VARCHAR(512)          DEFAULT NULL,
    is_enabled        TINYINT      NOT NULL DEFAULT 1,
    created_at        DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at        DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    CONSTRAINT fk_instr_device FOREIGN KEY (device_id) REFERENCES dev_device(id) ON DELETE CASCADE,
    INDEX idx_instr_device (device_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
