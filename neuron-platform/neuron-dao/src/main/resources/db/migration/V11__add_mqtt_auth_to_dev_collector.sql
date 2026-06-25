ALTER TABLE dev_collector
    ADD COLUMN mqtt_username      VARCHAR(64)  DEFAULT NULL COMMENT 'MQTT认证用户名',
    ADD COLUMN mqtt_password_hash VARCHAR(256) DEFAULT NULL COMMENT 'MQTT密码BCrypt哈希',
    ADD COLUMN mqtt_tls_enabled   TINYINT      DEFAULT 0  COMMENT '是否启用TLS',
    ADD COLUMN mqtt_broker_host   VARCHAR(128) DEFAULT NULL COMMENT 'MQTT Broker地址',
    ADD COLUMN mqtt_broker_port   INT          DEFAULT 1883 COMMENT 'MQTT Broker端口';
