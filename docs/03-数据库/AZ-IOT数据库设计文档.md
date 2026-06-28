# AZ-IOT 数据库设计文档

> 数据库名: `neuron_db` | 引擎: MySQL (InnoDB) | 字符集: `utf8mb4_unicode_ci`
> ~~迁移工具: Flyway~~ | 最后更新: 2026-06-28

---

## 目录

1. [ER 关系图](#er-关系图)
2. [系统模块 (sys_)](#系统模块)
   - [sys_user — 用户](#sys_user)
   - [sys_role — 角色](#sys_role)
   - [sys_permission — 权限](#sys_permission)
   - [sys_user_role — 用户角色关联](#sys_user_role)
   - [sys_role_permission — 角色权限关联](#sys_role_permission)
   - [sys_audit_log — 审计日志](#sys_audit_log)
   - [sys_config — 系统配置](#sys_config)
3. [设备模块 (dev_)](#设备模块)
   - [dev_protocol — 通信协议](#dev_protocol)
   - [dev_manufacturer — 设备厂商](#dev_manufacturer)
   - [dev_device_model — 设备型号](#dev_device_model)
   - [dev_register_map — 寄存器映射](#dev_register_map)
   - [dev_collector — 采集器](#dev_collector)
   - [dev_serial_port — 串口配置](#dev_serial_port)
   - [dev_device — 物理设备](#dev_device)
   - [dev_device_reading — 设备读数(分区)](#dev_device_reading)
   - [dev_device_alarm_config — 设备告警配置](#dev_device_alarm_config)
   - [dev_device_instruction — 设备指令](#dev_device_instruction)
4. [初始化数据](#初始化数据)
5. [关键设计决策](#关键设计决策)

---

## ER 关系图

```
┌──────────────┐        ┌────────────────┐        ┌──────────────┐
│   sys_user    │──────▶│ sys_user_role   │◀──────│   sys_role    │
│             1 : n     │                n : 1    │               │
└──────────────┘        └────────────────┘        └──────┬───────┘
                                                         │
                                                         │ 1 : n
                                                         ▼
                                                 ┌────────────────────┐
                                                 │ sys_role_permission│
                                                 └────────┬───────────┘
                                                          │ n : 1
                                                          ▼
                                                  ┌────────────────┐
                                                  │ sys_permission │
                                                  └────────────────┘

┌────────────────┐        ┌──────────────────────┐
│ dev_protocol   │──────▶│ dev_device_model      │
│              1 : n     │                       │
└────────────────┘       └──────────┬────────────┘
                          │         │         │
┌────────────────┐        │ 1 : n   │ 1 : n   │ 1 : n
│dev_manufacturer│────────┘         │         │
└────────────────┘                  ▼         ▼
                          ┌──────────────┐  ┌──────────────────┐
                          │dev_register_map│  │dev_device_instruction│
                          └──────────────┘  └──────────────────┘

┌───────────────┐      ┌────────────────┐      ┌──────────────┐
│ dev_collector │─────▶│dev_serial_port │─────▶│  dev_device  │
│             1 : n    │              1 : n    │              │
└──────┬───────┘      └────────────────┘      └──────┬───────┘
       │                                             │
       │ MQTT 双向通信                                 │ 1 : n
       │ (无外键，逻辑关联)                              │
       ▼                                             ▼
┌──────────────┐                           ┌──────────────────────┐
│  Rust 采集端 │                           │ dev_device_reading    │ (分区表)
│  (外部系统)  │                           │ dev_device_alarm_config│
└──────────────┘                           └──────────────────────┘
```

---

## 系统模块

### sys_user

> 用户表 — 平台用户认证与基本信息

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| username | VARCHAR(64) | UNIQUE NOT NULL | | 登录用户名 |
| password_hash | VARCHAR(256) | NOT NULL | | BCrypt 密码哈希 |
| nickname | VARCHAR(128) | NOT NULL | '' | 昵称/姓名 (Entity: `real_name`) |
| email | VARCHAR(128) | | NULL | 邮箱 |
| phone | VARCHAR(32) | | NULL | 手机号 |
| avatar_url | VARCHAR(512) | | NULL | 头像URL |
| status | TINYINT | NOT NULL | 1 | 1=启用 0=禁用 |
| is_deleted | TINYINT | NOT NULL | 0 | 逻辑删除 (@TableLogic) |
| last_login_time | DATETIME | | NULL | 最后登录时间 |
| created_at | DATETIME | NOT NULL | NOW() | |
| updated_at | DATETIME | NOT NULL | NOW() ON UPDATE | |

> ⚠️ **已知问题**: SQL 注释写 `admin / admin123`，但实际 BCrypt hash 匹配密码 `123456`。

---

### sys_role

> 角色表 — RBAC 角色定义

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| role_code | VARCHAR(64) | UNIQUE NOT NULL | | 角色编码 |
| role_name | VARCHAR(128) | NOT NULL | | 角色名称 |
| description | VARCHAR(256) | | NULL | 角色描述 |
| sort_order | INT | NOT NULL | 0 | 排序号 |
| status | TINYINT | NOT NULL | 1 | 1=启用 0=禁用 |
| is_deleted | TINYINT | NOT NULL | 0 | 逻辑删除 (@TableLogic) |
| created_at | DATETIME | NOT NULL | NOW() | |
| updated_at | DATETIME | NOT NULL | NOW() ON UPDATE | |

> ⚠️ **已知债务**: Java 实体 `SysRole.status` 为 `String`，但 DB 为 `TINYINT`。当前无写操作触发此字段，但后续若改 VARCHAR 需统一迁移。

---

### sys_permission

> 权限表 — 菜单/按钮/API 权限树

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| parent_id | BIGINT | NOT NULL | 0 | 父权限ID，0=根节点 |
| perm_code | VARCHAR(128) | UNIQUE NOT NULL | | 权限编码 |
| perm_name | VARCHAR(128) | NOT NULL | | 权限名称 |
| perm_type | VARCHAR(16) | NOT NULL | 'menu' | menu / button / api |
| path | VARCHAR(256) | | NULL | 路由路径/API路径 |
| icon | VARCHAR(64) | | NULL | 图标 |
| sort_order | INT | NOT NULL | 0 | 排序号 |
| status | TINYINT | NOT NULL | 1 | 1=启用 0=停用 |
| created_at | DATETIME | NOT NULL | NOW() | |
| updated_at | DATETIME | NOT NULL | NOW() ON UPDATE | |

> ⚠️ **已知债务**: Java 实体 `SysPermission.status` 为 `String`，DB 为 `TINYINT`。同 sys_role。

---

### sys_user_role

> 用户角色关联表 — 多对多中间表

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| user_id | BIGINT | NOT NULL | | FK → sys_user.id |
| role_id | BIGINT | NOT NULL | | FK → sys_role.id |
| created_at | DATETIME | NOT NULL | NOW() | |

**索引**:
- `UNIQUE KEY uk_user_role (user_id, role_id)`
- `INDEX idx_user_role_user (user_id)`
- `INDEX idx_user_role_role (role_id)`

> 注意: 此表未创建显式外键约束。

---

### sys_role_permission

> 角色权限关联表 — 多对多中间表

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| role_id | BIGINT | NOT NULL | | FK → sys_role.id |
| permission_id | BIGINT | NOT NULL | | FK → sys_permission.id |
| created_at | DATETIME | NOT NULL | NOW() | |

**索引**:
- `UNIQUE KEY uk_role_perm (role_id, permission_id)`
- `INDEX idx_role_perm_role (role_id)`
- `INDEX idx_role_perm_perm (permission_id)`

---

### sys_audit_log

> 审计日志表 — 操作记录与追溯

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| module | VARCHAR(64) | NOT NULL | | 模块名 |
| action | VARCHAR(128) | NOT NULL | | 操作描述 |
| operator_id | BIGINT | | NULL | 操作人ID |
| operator_name | VARCHAR(64) | | NULL | 操作人用户名 |
| request_ip | VARCHAR(64) | | NULL | 请求IP |
| request_method | VARCHAR(16) | | NULL | HTTP Method |
| request_url | VARCHAR(256) | | NULL | 请求URL |
| request_params | TEXT | | NULL | 请求参数 |
| response_result | TEXT | | NULL | 响应结果 |
| cost_ms | INT | | 0 | 耗时(ms) |
| status | TINYINT | | 1 | 1=成功 0=失败 |
| created_at | DATETIME | NOT NULL | NOW() | |

**索引**:
- `INDEX idx_audit_log_time (created_at)`
- `INDEX idx_audit_log_operator (operator_id)`

> 默认保留 180 天 (sys_config: `audit_log_retention_days=180`)。
> ⚠️ **已知债务**: Java 实体 `SysAuditLog.status` 为 `String`，DB 为 `TINYINT`。写入依赖 DB 默认值，后续若改 VARCHAR 需统一迁移。

---

### sys_config

> 系统配置表 — 运行时配置键值对

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| config_key | VARCHAR(64) | UNIQUE NOT NULL | | 配置键 |
| config_value | VARCHAR(256) | NOT NULL | | 配置值 |
| description | VARCHAR(256) | | NULL | 配置说明 |
| updated_at | DATETIME | NOT NULL | NOW() ON UPDATE | |

---

## 设备模块

### dev_protocol

> 通信协议定义表 — 与 Rust 采集端 `ProtocolType` 枚举严格对齐

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| code | VARCHAR(32) | UNIQUE NOT NULL | | 协议编码 (与Rust枚举一致) |
| name | VARCHAR(64) | NOT NULL | | 协议名称 |
| bus_type | VARCHAR(16) | NOT NULL | | serial / tcp |
| description | VARCHAR(256) | | NULL | 协议说明 |
| is_enabled | TINYINT | NOT NULL | 1 | 1=启用 0=停用 |
| created_at | DATETIME | NOT NULL | NOW() | |
| updated_at | DATETIME | NOT NULL | NOW() ON UPDATE | |

---

### dev_manufacturer

> 设备厂商表

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| code | VARCHAR(32) | UNIQUE NOT NULL | | 厂商编码 |
| name | VARCHAR(128) | NOT NULL | | 厂商名称 |
| country | VARCHAR(64) | | NULL | 国家 |
| website | VARCHAR(256) | | NULL | 官网 |
| description | VARCHAR(512) | | NULL | 厂商描述 |
| is_deleted | TINYINT | NOT NULL | 0 | 逻辑删除 (@TableLogic) |
| created_at | DATETIME | NOT NULL | NOW() | |
| updated_at | DATETIME | NOT NULL | NOW() ON UPDATE | |

---

### dev_device_model

> 设备型号表 — 设备的模板定义

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| manufacturer_id | BIGINT | NOT NULL | | FK → dev_manufacturer.id |
| protocol_id | BIGINT | NOT NULL | | FK → dev_protocol.id |
| code | VARCHAR(64) | UNIQUE NOT NULL | | 型号编码 |
| name | VARCHAR(128) | NOT NULL | | 型号名称 |
| description | VARCHAR(512) | | NULL | 型号描述 |
| is_enabled | TINYINT | NOT NULL | 1 | 1=启用 0=停用 |
| created_at | DATETIME | NOT NULL | NOW() | |
| updated_at | DATETIME | NOT NULL | NOW() ON UPDATE | |

**外键与索引**:
- `CONSTRAINT fk_model_manufacturer FOREIGN KEY (manufacturer_id) REFERENCES dev_manufacturer(id)`
- `CONSTRAINT fk_model_protocol FOREIGN KEY (protocol_id) REFERENCES dev_protocol(id)`
- `INDEX idx_model_manufacturer (manufacturer_id)`
- `INDEX idx_model_protocol (protocol_id)`

> V14 迁移将 `collect_interval_sec` 移至 `dev_collector` 表。

---

### dev_register_map

> 寄存器映射表 — 设备型号内每个采集点的寄存器定义

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| model_id | BIGINT | NOT NULL | | FK → dev_device_model.id (CASCADE) |
| sensor_code | VARCHAR(64) | NOT NULL | | 传感器编码 |
| sensor_name | VARCHAR(128) | NOT NULL | | 传感器名称 |
| register_address | INT | NOT NULL | | 寄存器起始地址 |
| register_count | INT | NOT NULL | 1 | 寄存器数量 |
| data_type | VARCHAR(16) | NOT NULL | 'uint16' | uint16/int16/uint32/int32/float32/float64/string/bool |
| byte_order | VARCHAR(8) | NOT NULL | 'ABCD' | ABCD/BADC/CDAB/DCBA |
| func_code | VARCHAR(8) | NOT NULL | '0x03' | 功能码 (0x01~0x04) |
| coefficient | DECIMAL(12,6) | NOT NULL | 1.0 | 线性系数 (y = kx + b) |
| offset_val | DECIMAL(12,6) | NOT NULL | 0.0 | 线性偏移 |
| unit | VARCHAR(16) | NOT NULL | '' | 单位 (V/A/kW/kWh/℃) |
| rw | VARCHAR(8) | NOT NULL | 'R' | R=只读 W=只写 RW=读写 |
| sort_order | INT | NOT NULL | 0 | 排序号 |
| description | VARCHAR(256) | | NULL | 说明 |
| extra_params | JSON | | NULL | 协议特定参数(MQTT topic/OPC UA node_id/SNMP OID等, V17) |
| created_at | DATETIME | NOT NULL | NOW() | |
| updated_at | DATETIME | NOT NULL | NOW() ON UPDATE | |

**索引与约束**:
- `UNIQUE KEY uk_model_sensor (model_id, sensor_code)` — 同一型号下传感器编码唯一
- `CONSTRAINT fk_regmap_model FOREIGN KEY (model_id) REFERENCES dev_device_model(id) ON DELETE CASCADE`

---

### dev_collector

> 采集器表 — 现场边缘网关 (BC-U101)

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| code | VARCHAR(32) | UNIQUE NOT NULL | | 采集器编码 |
| name | VARCHAR(128) | NOT NULL | | 采集器名称 |
| type | VARCHAR(32) | NOT NULL | 'BC-U101' | 采集器型号 |
| mqtt_client_id | VARCHAR(64) | UNIQUE NOT NULL | | MQTT Client ID |
| ip_address | VARCHAR(45) | | NULL | 采集器IP |
| collect_interval_sec | INT | NOT NULL | 900 | 采集间隔(秒) (V14迁入) |
| status | VARCHAR(16) | NOT NULL | 'offline' | offline / online / alarm (V18: TINYINT→VARCHAR) |
| firmware_version | VARCHAR(32) | | NULL | 固件版本 |
| last_heartbeat | DATETIME | | NULL | 最后心跳时间 |
| description | VARCHAR(512) | | NULL | 描述 |
| **以下字段由 V11 迁入** | | | | |
| mqtt_username | VARCHAR(64) | | NULL | MQTT 认证用户名 |
| mqtt_password_hash | VARCHAR(256) | | NULL | MQTT 密码 BCrypt 哈希 |
| mqtt_tls_enabled | TINYINT | | 0 | 是否启用 TLS |
| mqtt_broker_host | VARCHAR(128) | | NULL | MQTT Broker 地址 |
| mqtt_broker_port | INT | | 1883 | MQTT Broker 端口 |
| created_at | DATETIME | NOT NULL | NOW() | |
| updated_at | DATETIME | NOT NULL | NOW() ON UPDATE | |

> ⚠️ V14 之前 `collect_interval_sec` 可能存在旧版本未执行迁移的风险。
> **V18**: `status` 已从 `TINYINT(0/1/2)` 改为 `VARCHAR(16) (offline/online/alarm)`，与 `dev_device.status` 统一。

---

### dev_serial_port

> 串口配置表 — 采集器下挂的物理通信端口

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| collector_id | BIGINT | NOT NULL | | FK → dev_collector.id (CASCADE) |
| port_name | VARCHAR(16) | NOT NULL | | 端口名称 (COM1/ttyUSB0) |
| port_label | VARCHAR(64) | | NULL | 端口标签 |
| bus_type | VARCHAR(16) | NOT NULL | 'serial' | 总线类型 |
| bus_param | JSON | NOT NULL | | 总线参数 (波特率/校验位等) |
| port_type | VARCHAR(16) | NOT NULL | 'device' | device / io_board / sms_modem |
| is_active | TINYINT | NOT NULL | 1 | 1=启用 0=停用 |
| created_at | DATETIME | NOT NULL | NOW() | |
| updated_at | DATETIME | NOT NULL | NOW() ON UPDATE | |

**索引与约束**:
- `UNIQUE KEY uk_collector_port (collector_id, port_name)` — 同一采集器下端口名唯一
- `CONSTRAINT fk_port_collector FOREIGN KEY (collector_id) REFERENCES dev_collector(id) ON DELETE CASCADE`

---

### dev_device

> 物理设备表 — 实际挂载在串口下的计量/控制设备

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| serial_port_id | BIGINT | | **NULL** | FK → dev_serial_port.id (TCP协议可为NULL, V19) |
| model_id | BIGINT | NOT NULL | | FK → dev_device_model.id |
| code | VARCHAR(64) | UNIQUE NOT NULL | | 设备编号 |
| name | VARCHAR(128) | NOT NULL | | 设备名称 |
| slave_addr | INT | NOT NULL | 1 | Modbus 从站地址 |
| collect_interval_sec | INT | | NULL | 采集间隔(秒)，NULL=使用采集器级别 |
| status | VARCHAR(16) | NOT NULL | 'offline' | offline / online / alarm / disabled (V16: TINYINT→VARCHAR) |
| location | VARCHAR(256) | | NULL | 安装位置 |
| description | VARCHAR(512) | | NULL | 设备描述 |
| is_deleted | TINYINT | NOT NULL | 0 | 逻辑删除 (@TableLogic) |
| created_at | DATETIME | NOT NULL | NOW() | |
| updated_at | DATETIME | NOT NULL | NOW() ON UPDATE | |

**外键与索引**:
- `CONSTRAINT fk_device_port FOREIGN KEY (serial_port_id) REFERENCES dev_serial_port(id)`
- `CONSTRAINT fk_device_model FOREIGN KEY (model_id) REFERENCES dev_device_model(id)`
- `INDEX idx_device_port (serial_port_id)`
- `INDEX idx_device_model (model_id)`

> **V16**: `status` 已从 `TINYINT(0/1/2/3)` 改为 `VARCHAR(16) (offline/online/alarm/disabled)`。

---

### dev_device_reading

> 设备读数表 — 高吞吐实时数据，**按月 RANGE 分区**

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK 第一部分 | AUTO | 主键 |
| device_id | BIGINT | NOT NULL | | 设备ID |
| register_id | BIGINT | NOT NULL | | 寄存器映射ID |
| sensor_code | VARCHAR(64) | NOT NULL | | 传感器编码 (冗余，加速查询) |
| value | DECIMAL(20,6) | | NULL | 瞬时值 |
| avg | DECIMAL(20,6) | | NULL | 窗口内平均值 |
| max | DECIMAL(20,6) | | NULL | 窗口内最大值 |
| min | DECIMAL(20,6) | | NULL | 窗口内最小值 |
| sample_count | INT | NOT NULL | 1 | 窗口内采样数 |
| quality | TINYINT | NOT NULL | 0 | 0=正常 1=超时 2=异常 3=传感器故障 |
| read_at | DATETIME | PK 第二部分 | | 读数时间 |
| window_start | DATETIME | | NULL | 聚合窗口起始 |
| window_end | DATETIME | | NULL | 聚合窗口结束 |
| created_at | DATETIME | NOT NULL | NOW() | |

**复合主键**: `PRIMARY KEY (id, read_at)` — 配合分区键
**索引**:
- `INDEX idx_device_time (device_id, read_at)`
- `INDEX idx_register_time (register_id, read_at)`
- `INDEX idx_device_register (device_id, register_id, read_at)`

**分区策略 (按月 RANGE)**:
```
PARTITION BY RANGE (TO_DAYS(read_at)) (
    PARTITION p202606 VALUES LESS THAN (TO_DAYS('2026-07-01')),
    PARTITION p202607 VALUES LESS THAN (TO_DAYS('2026-08-01')),
    PARTITION p202608 VALUES LESS THAN (TO_DAYS('2026-09-01')),
    PARTITION p_future VALUES LESS THAN MAXVALUE
)
```

> ⚠️ **运维提醒**: 每月需新增下月分区 + 按 `data_retention_days`(默认90天) 清理过期分区。
> 默认保留 90 天 (sys_config: `data_retention_days=90`)。
> ⚠️ **已知债务**: Java 实体 `DevDeviceReading.quality` 为 `String`，DB 为 `TINYINT`。当前写 `"0"` 由 MySQL 隐式转换，后续若改 VARCHAR 需统一迁移。

---

### dev_device_alarm_config

> 设备告警配置表 — 设备级别的告警规则定义

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| device_id | BIGINT | NOT NULL | | FK → dev_device.id |
| sensor_code | VARCHAR(64) | NOT NULL | | 采集点编码 |
| alarm_enabled | TINYINT | NOT NULL | 1 | 1=启用 0=停用 |
| alarm_type | VARCHAR(32) | NOT NULL | 'limit_upper' | 告警类型 (见下方) |
| params | JSON | NOT NULL | | 告警参数 (按类型不同) |
| alarm_level | VARCHAR(16) | NOT NULL | 'warning' | info / warning / critical |
| description | VARCHAR(256) | | NULL | 告警描述 |
| created_at | DATETIME | NOT NULL | NOW() | |
| updated_at | DATETIME | NOT NULL | NOW() ON UPDATE | |

**告警类型** (alarm_type):
| 类型值 | params JSON 示例 | 说明 |
|--------|------------------|------|
| limit_upper | `{"threshold": 100}` | 上限告警 |
| limit_lower | `{"threshold": 0}` | 下限告警 |
| range   | `{"min": 0, "max": 100}` | 范围告警 |
| rate_change | `{"rate": 10, "unit": "percent", "window_sec": 300}` | 变化率告警 |
| communication | `{"timeout_sec": 60}` | 通讯超时告警 |

**索引与约束**:
- `UNIQUE KEY uk_device_sensor_type (device_id, sensor_code, alarm_type)`
- `INDEX idx_alarm_device (device_id)`

> 此表无外键约束，但 `device_id` 逻辑关联 `dev_device.id`。

---

### dev_device_instruction

> 设备指令表 — 设备支持的读写/控制指令集

| 字段 | 类型 | 键 | 默认值 | 说明 |
|------|------|------|--------|------|
| id | BIGINT | PK AUTO | | 主键 |
| device_id | BIGINT | NOT NULL | | FK → dev_device.id (CASCADE) |
| instruction_code | VARCHAR(64) | NOT NULL | | 指令编码 |
| instruction_name | VARCHAR(128) | NOT NULL | | 指令名称 |
| instruction_type | VARCHAR(32) | NOT NULL | 'READ' | READ/WRITE/CONTROL/CONFIG |
| func_code | VARCHAR(16) | | '0x03' | Modbus 功能码 |
| register_address | INT | | 0 | 寄存器地址 |
| register_count | INT | | 1 | 寄存器数量 |
| params | JSON | | NULL | 额外参数 (dataType,byteOrder等) |
| sort_order | INT | NOT NULL | 0 | 排序号 |
| description | VARCHAR(512) | | NULL | 指令说明 |
| is_enabled | TINYINT | NOT NULL | 1 | 1=启用 0=停用 |
| created_at | DATETIME | NOT NULL | NOW() | |
| updated_at | DATETIME | NOT NULL | NOW() ON UPDATE | |

**索引与约束**:
- `CONSTRAINT fk_instr_device FOREIGN KEY (device_id) REFERENCES dev_device(id) ON DELETE CASCADE`
- `INDEX idx_instr_device (device_id)`

---

## 初始化数据

### 默认用户
| username | password | nickname | 角色 |
|----------|----------|----------|------|
| admin | ~~admin123~~ **123456** | 超级管理员 | admin (管理员) |

> ⚠️ **已知 BUG**: SQL 迁移文件注释写 `admin123`，实际 BCrypt hash 匹配 `123456`。

### 预置角色
| role_code | role_name | sort_order |
|-----------|-----------|------------|
| admin | 管理员 | 1 |
| operator | 运维人员 | 2 |
| viewer | 查看者 | 3 |

### 系统配置
| config_key | config_value | 说明 |
|------------|-------------|------|
| data_retention_days | 90 | 读数保留天数 |
| audit_log_retention_days | 180 | 审计日志保留天数 |
| default_collect_interval_sec | 600 | 全局默认采集间隔(秒) |

### 预置协议 (18种)

| code | bus_type | 状态 |
|------|----------|------|
| MODBUS_RTU | serial | ✅ 启用 |
| MODBUS_TCP | tcp | ✅ 启用 |
| DL_T645_2007 | serial | ✅ 启用 |
| DL_T645_1997 | serial | ✅ 启用 |
| IEC_60870_5_101 | serial | ✅ 启用 |
| IEC_60870_5_104 | tcp | ✅ 启用 |
| DNP3 | tcp | ✅ 启用 |
| OPC_UA | tcp | ✅ 启用 |
| MQTT | tcp | ✅ 启用 |
| BACNET_IP | tcp | ✅ 启用 |
| SNMP_V2C | tcp | ✅ 启用 |
| HTTP_JSON | tcp | ✅ 启用 |
| CAN_BUS | serial | ✅ 启用 |
| S7_COMM | tcp | ✅ 启用 |
| PROFIBUS_DP | serial | ✅ 启用 |
| ETHERNET_IP | tcp | ✅ 启用 |
| FINS_TCP | tcp | ✅ 启用 |
| MITSUBISHI_MC | tcp | ✅ 启用 |

> V16 迁移将旧编码 `DL_T645`、`SNMP`、`AT_COMMAND` 设为停用(`is_enabled=0`)。

---

## 关键设计决策

| # | 决策 | 说明 |
|---|------|------|
| ADR-01 | dev_device_reading 按月分区 | 高吞吐场景，按 TO_DAYS 做 RANGE 分区，便于按月淘汰 |
| ADR-02 | 复合主键 (id, read_at) | 读数表分区键必须在主键中 |
| ADR-03 | sensor_code 冗余 | 避免 JOIN dev_register_map 加速查询 |
| ADR-04 | dev_device_model 不再管理采集间隔 | V14 将 `collect_interval_sec` 移至 `dev_collector`，采集器级别统一控制 |
| ADR-05 | dev_device.status V16 改为 VARCHAR | 从 `TINYINT` 改为可读字符串 `offline/online/alarm/disabled` |
| ADR-06 | sys_user_role / sys_role_permission 无外键 | 中间表未创建显式 FK，减少锁竞争 |
| ADR-07 | dev_collector MQTT 认证分离 | V11 为每个采集器独立存储 MQTT 凭证，支持 TLS |
| ADR-08 | 协议编码与 Rust 枚举编译期对齐 | dev_protocol.code 在 V16 中与采集端 ProtocolType 严格一致 |
| ADR-09 | TCP 协议使用 extra_params JSON | dev_register_map 新增加 extra_params 字段，支持 MQTT topic/node_id、OPC UA node_id、SNMP OID 等协议差异化参数 |
| ADR-10 | collector.status V18 改为 VARCHAR | 与 dev_device.status (V16) 保持一致，统一使用可读字符串 `offline/online/alarm`，采集器侧（Rust）MQTT payload 即报告字符串，无需中间转换 |
| ADR-11 | sys_role/sys_permission/sys_auditLog/reading.quality 待统一 | 4 处 Java String↔DB TINYINT 不一致。quality 已修复 (Java Integer→TINYINT)，剩余 3 处当前无运行时错误但为设计债务，后续应走 V20 迁移统一 |

---

## 已知债务（待迁移）

| # | 表 | 列 | DB 类型 | Java 类型 | 风险 | 建议 |
|---|-----|------|---------|----------|------|------|
| 1 | sys_role | status | TINYINT | String | 代码只读，读到 "1"/"0" | V20: TINYINT→VARCHAR(16) `enabled`/`disabled` |
| 2 | sys_permission | status | TINYINT | String | 同上 | 同上 |
| 3 | sys_audit_log | status | TINYINT | String | 插入用 DB 默认值 | V20: TINYINT→VARCHAR(16) `success`/`failure` |
| 4 | dev_device_reading | quality | TINYINT | Integer | ✅ 已修复 (P1-011) | — |

---

## 版本记录

| 日期 | 迁移文件 | 变更摘要 |
|------|---------|---------|
| 2026-06-23 | V1~V1.5 | 初始化系统模块 (用户/角色/权限/审计) |
| 2026-06-23 | V2~V5 | 初始化设备模块 (协议/厂商/型号/寄存器) |
| 2026-06-23 | V6~V9 | 初始化采集链路 (采集器/串口/设备/读数分区) |
| 2026-06-23 | V10 | 系统配置表 |
| 2026-06-24 | V11 | 采集器增加 MQTT 认证字段 |
| 2026-06-25 | V13 | 设备告警配置表 |
| 2026-06-25 | V14 | 采集间隔从型号移至采集器 |
| 2026-06-26 | V15 | 设备指令表 |
| 2026-06-27 | V16 | 设备状态改为 VARCHAR + 协议编码对齐 |
| 2026-06-28 | V17 | register_map 增加 extra_params JSON 列 (TCP 协议支持) |
| 2026-06-28 | V18 | collector.status TINYINT → VARCHAR (与 device 对齐) |
| 2026-06-28 | V19 | device.serial_port_id 改为 NULLABLE (TCP协议设备无需串口) |

> **V12 已跳过**。**V18** 因 Flyway 未自动执行需手动补跑迁移 SQL；**V19** 需同步确认已应用。
