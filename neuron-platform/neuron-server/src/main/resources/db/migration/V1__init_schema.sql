-- Flyway 基线迁移: 初始化协议表 + 插入15种工业通信协议
-- 版本: V1__init_schema.sql

-- 协议定义表
CREATE TABLE IF NOT EXISTS dev_protocol (
    id          BIGINT AUTO_INCREMENT PRIMARY KEY,
    code        VARCHAR(32)  NOT NULL COMMENT '协议编码，与采集端枚举严格一致',
    name        VARCHAR(64)  NOT NULL COMMENT '协议名称',
    bus_type    VARCHAR(16)  NOT NULL COMMENT '传输方式: serial / tcp',
    description VARCHAR(255) DEFAULT '' COMMENT '协议说明',
    is_enabled  TINYINT      DEFAULT 1 COMMENT '是否启用: 1=启用, 0=停用',
    created_at  DATETIME     DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME     DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_code (code)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='工业通信协议定义表';

-- 插入 15 种协议，code 与 Rust ProtocolType 枚举严格对应
INSERT INTO dev_protocol (code, name, bus_type, description) VALUES
('MODBUS_RTU',       'Modbus RTU',        'serial', '串口 Modbus，工业现场最通用的协议'),
('MODBUS_TCP',       'Modbus TCP',        'tcp',    '以太网 Modbus'),
('DL_T645_2007',     'DL/T645-2007',      'serial', '国家电网多功能电能表通信协议 2007 版'),
('DL_T645_1997',     'DL/T645-1997',      'serial', '国家电网多功能电能表通信协议 1997 版'),
('IEC_60870_5_104',  'IEC 60870-5-104',   'tcp',    '电力远动通信协议 (TCP)'),
('IEC_60870_5_101',  'IEC 60870-5-101',   'serial', '电力远动通信协议 (串口)'),
('DNP3',             'DNP3',              'tcp',    '分布式网络协议'),
('OPC_UA',           'OPC UA',            'tcp',    '统一架构工业互操作协议'),
('MQTT',             'MQTT',              'tcp',    '物联网消息队列遥测传输'),
('BACNET_IP',        'BACnet/IP',         'tcp',    '楼宇自动化控制网络协议'),
('SNMP_V2C',         'SNMP v2c',          'tcp',    '简单网络管理协议'),
('HTTP_JSON',        'HTTP JSON',         'tcp',    'HTTP REST API 轮询'),
('CAN_BUS',          'CAN Bus',           'serial', '控制器局域网络 (需 SocketCAN)'),
('S7_COMM',          'S7 Communication',  'tcp',    '西门子 S7-300/400/1200/1500 PLC'),
('PROFIBUS_DP',      'PROFIBUS DP',       'serial', '过程现场总线 DP-V0'),
('ETHERNET_IP',      'EtherNet/IP',       'tcp',    '工业以太网 (Rockwell CIP)'),
('FINS_TCP',         'FINS TCP',          'tcp',    '欧姆龙 PLC 通信协议'),
('MITSUBISHI_MC',    'Mitsubishi MC',     'tcp',    '三菱 MELSEC MC 协议');
