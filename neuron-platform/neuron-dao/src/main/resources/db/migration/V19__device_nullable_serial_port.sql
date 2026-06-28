-- V19: 允许 TCP 协议设备不绑定串口
-- TCP/MQTT/OPC UA 等协议通过 IP 通信, 不需要物理串口

ALTER TABLE dev_device
    MODIFY COLUMN serial_port_id BIGINT NULL COMMENT '串口ID(TCP协议可为NULL)';
