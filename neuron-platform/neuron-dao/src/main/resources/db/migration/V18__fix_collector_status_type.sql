-- V18: 将 dev_collector.status 从 TINYINT 改为 VARCHAR, 与 dev_device 保持一致
-- 原因: Java 代码中使用字符串 'online'/'offline'/'alarm', 与 V16 的 dev_device.status 对齐
-- 同时将现有数据转换: 0→offline, 1→online, 2→alarm

ALTER TABLE dev_collector
    ADD COLUMN status_new VARCHAR(16) NOT NULL DEFAULT 'offline' COMMENT '状态: online / offline / alarm' AFTER status;

UPDATE dev_collector SET status_new = CASE status
    WHEN 1 THEN 'online'
    WHEN 2 THEN 'alarm'
    ELSE 'offline'
END;

ALTER TABLE dev_collector DROP COLUMN status;
ALTER TABLE dev_collector CHANGE COLUMN status_new status VARCHAR(16) NOT NULL DEFAULT 'offline' COMMENT '状态: online / offline / alarm';
