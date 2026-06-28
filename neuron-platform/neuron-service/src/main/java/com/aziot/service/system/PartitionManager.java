package com.aziot.service.system;

import com.aziot.dao.entity.system.SysConfig;
import com.aziot.dao.mapper.system.SysConfigMapper;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.scheduling.annotation.Scheduled;
import org.springframework.stereotype.Component;

import java.time.LocalDate;
import java.time.format.DateTimeFormatter;

/**
 * MySQL 分区自动管理 — 按月预建分区 + 按配置清理过期分区。
 *
 * 目标表: dev_device_reading
 * 分区策略: RANGE COLUMNS(read_at) → 按月拆分
 * 保留策略: 可配置 data_retention_days (sys_config 表)，默认 365 天
 */
@Slf4j
@Component
@RequiredArgsConstructor
public class PartitionManager {

    private final JdbcTemplate jdbcTemplate;
    private final SysConfigMapper sysConfigMapper;

    private static final DateTimeFormatter YM = DateTimeFormatter.ofPattern("yyyyMM");
    private static final DateTimeFormatter DATE = DateTimeFormatter.ofPattern("yyyy-MM-dd");

    /**
     * 每月 1 日凌晨 3 点 — 为下月预建分区。
     */
    @Scheduled(cron = "0 0 3 1 * ?")
    public void createNextMonthPartition() {
        LocalDate nextMonthFirst = LocalDate.now().plusMonths(1).withDayOfMonth(1);
        String partitionName = "p" + nextMonthFirst.format(YM);
        String lessThanDate = nextMonthFirst.format(DATE);

        // Validate partition name format to prevent injection
        if (!partitionName.matches("p\\d{6}")) {
            log.error("Invalid partition name format: {}", partitionName);
            return;
        }

        String sql = String.format(
            "ALTER TABLE dev_device_reading " +
            "REORGANIZE PARTITION p_future INTO (" +
            "  PARTITION %s VALUES LESS THAN (TO_DAYS('%s'))," +
            "  PARTITION p_future VALUES LESS THAN MAXVALUE" +
            ")",
            partitionName, lessThanDate
        );

        try {
            jdbcTemplate.execute(sql);
            log.info("PartitionManager: created partition {} for < {}", partitionName, lessThanDate);
        } catch (Exception e) {
            log.warn("PartitionManager: create partition {} failed, may already exist: {}", partitionName, e.getMessage());
        }
    }

    /**
     * 每天凌晨 4 点 — 按 data_retention_days 清理过期分区。
     * 默认保留 365 天。
     */
    @Scheduled(cron = "0 0 4 * * ?")
    public void dropExpiredPartitions() {
        int retentionDays = getRetentionDays();
        LocalDate cutoff = LocalDate.now().minusDays(retentionDays);
        log.info("PartitionManager: checking expired partitions, cutoff={}, retentionDays={}", cutoff, retentionDays);

        try {
            // 查询所有分区 (排除 p_future)
            String sql = "SELECT PARTITION_NAME FROM INFORMATION_SCHEMA.PARTITIONS " +
                         "WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'dev_device_reading' " +
                         "AND PARTITION_NAME IS NOT NULL AND PARTITION_NAME != 'p_future' " +
                         "ORDER BY PARTITION_ORDINAL_POSITION";
            var partitions = jdbcTemplate.queryForList(sql, String.class);

            for (String name : partitions) {
                if (!name.startsWith("p") || name.length() < 7) continue;

                // 解析分区名 p202608 → 2026-08-01
                int year = Integer.parseInt(name.substring(1, 5));
                int month = Integer.parseInt(name.substring(5, 7));
                LocalDate partDate = LocalDate.of(year, month, 1).plusMonths(1);

                if (partDate.isBefore(cutoff)) {
                    try {
                        jdbcTemplate.execute("ALTER TABLE dev_device_reading DROP PARTITION " + name);
                        log.info("PartitionManager: dropped expired partition {}", name);
                    } catch (Exception e) {
                        log.warn("PartitionManager: drop partition {} failed: {}", name, e.getMessage());
                    }
                }
            }
        } catch (Exception e) {
            log.error("PartitionManager: drop expired partitions failed", e);
        }
    }

    /**
     * 从 sys_config 表读取 data_retention_days，默认 365。
     */
    private int getRetentionDays() {
        try {
            var wrapper = new LambdaQueryWrapper<SysConfig>()
                .eq(SysConfig::getConfigKey, "data_retention_days");
            SysConfig config = sysConfigMapper.selectOne(wrapper);
            if (config != null) {
                return Integer.parseInt(config.getConfigValue());
            }
        } catch (Exception e) {
            log.warn("PartitionManager: failed to read data_retention_days, using default 365", e);
        }
        return 365;
    }
}
