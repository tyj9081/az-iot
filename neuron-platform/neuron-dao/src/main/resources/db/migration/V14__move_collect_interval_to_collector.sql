-- V14: Move collect_interval_sec from dev_device_model to dev_collector
ALTER TABLE dev_collector ADD COLUMN collect_interval_sec INT NOT NULL DEFAULT 900 COMMENT '采集间隔(秒)' AFTER ip_address;
ALTER TABLE dev_device_model DROP COLUMN collect_interval_sec;
