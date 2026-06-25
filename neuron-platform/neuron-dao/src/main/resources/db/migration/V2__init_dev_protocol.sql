CREATE TABLE dev_protocol (
    id              BIGINT AUTO_INCREMENT PRIMARY KEY,
    code            VARCHAR(32)  NOT NULL UNIQUE,
    name            VARCHAR(64)  NOT NULL,
    bus_type        VARCHAR(16)  NOT NULL COMMENT 'serial/tcp/udp',
    description     VARCHAR(256)          DEFAULT NULL,
    is_enabled      TINYINT      NOT NULL DEFAULT 1,
    created_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO dev_protocol (code, name, bus_type) VALUES
('MODBUS_RTU', 'Modbus RTU', 'serial'),
('MODBUS_TCP', 'Modbus TCP', 'tcp'),
('DL_T645', 'DL/T645', 'serial'),
('BACNET_IP', 'BACnet/IP', 'tcp'),
('SNMP', 'SNMP', 'tcp'),
('AT_COMMAND', 'AT指令', 'serial');
