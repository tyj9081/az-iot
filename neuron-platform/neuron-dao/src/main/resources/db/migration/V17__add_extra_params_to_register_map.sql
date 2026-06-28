-- V17: 为寄存器点表增加 extra_params JSON 字段, 支持 TCP 协议差异化参数
-- 如 MQTT 的 topic/json_path、OPC UA 的 node_id、SNMP 的 OID、HTTP 的 url/method 等

ALTER TABLE dev_register_map
    ADD COLUMN extra_params JSON DEFAULT NULL COMMENT '协议特定参数(JSON)' AFTER description;
