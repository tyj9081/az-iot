# API 清单

> AZ-IOT 全部 API 端点 | 统一前缀: /api/v1  
> 协议支持: 18 种工业协议 (见 `common.enums.ProtocolType`) | 两端枚举 code 严格对齐

---

## 认证

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | /auth/login | 登录 → {accessToken, refreshToken} |
| POST | /auth/refresh | 刷新 Token |
| POST | /auth/logout | 登出 |
| GET | /auth/me | 当前用户信息 |

## 用户管理

| GET | /users | 分页列表 (keyword 搜索) |
| GET | /users/{id} | 用户详情 |
| POST | /users | 创建 |
| PUT | /users/{id} | 更新 |
| PUT | /users/{id}/password | 修改密码 |
| PUT | /users/{id}/status | 启用/禁用 |

## 角色管理

| GET | /roles | 全部角色列表 |
| GET | /roles/{id} | 角色详情(含权限) |
| POST | /roles | 创建 |
| PUT | /roles/{id} | 更新 |
| DELETE | /roles/{id} | 删除 |
| PUT | /roles/{id}/permissions | 设置权限 {permissionIds:[]} |

## 权限

| GET | /permissions | 权限树 |
| GET | /permissions/menu | 当前用户菜单树 |

## 协议

| GET | /protocols | 全部启用的协议 |

## 品牌

| GET | /manufacturers | 分页 (keyword 搜索) |
| GET | /manufacturers/{id} | 详情 |
| POST | /manufacturers | 创建 |
| PUT | /manufacturers/{id} | 更新 |
| DELETE | /manufacturers/{id} | 软删除 |

## 设备型号

| GET | /device-models | 分页 (manufacturerId/protocolId/keyword 筛选) |
| GET | /device-models/{id} | 详情 (含关联名称) |
| POST | /device-models | 创建 |
| PUT | /device-models/{id} | 更新 |
| DELETE | /device-models/{id} | 删除 |

## 点表

| GET | /device-models/{modelId}/registers | 型号下全部寄存器 (sort_order) |
| POST | /device-models/{modelId}/registers | 单条添加 |
| POST | /device-models/{modelId}/registers/batch | 批量导入 (List body) |
| PUT | /device-models/{modelId}/registers/{id} | 更新 |
| DELETE | /device-models/{modelId}/registers/{id} | 删除 |

## 采集器

| GET | /collectors | 分页 (status/keyword 筛选) |
| GET | /collectors/{id} | 详情 |
| POST | /collectors | 创建 (自动生成 COM1-COM10) |
| PUT | /collectors/{id} | 更新 |
| DELETE | /collectors/{id} | 删除 |

## 串口

| GET | /collectors/{collectorId}/serial-ports | 采集器下全部串口 |
| PUT | /collectors/{collectorId}/serial-ports/{id} | 更新串口参数 |

## 设备

| GET | /devices | 分页 (serialPortId/modelId/status/keyword) |
| GET | /devices/{id} | 详情 (含关联名称) |
| POST | /devices | 创建 → MQTT delta |
| PUT | /devices/{id} | 更新 → MQTT delta |
| DELETE | /devices/{id} | 软删除 → MQTT delta(remove) |
| PUT | /devices/{id}/status | 启用/禁用 |
| GET | /devices/{id}/alarm-config | 告警规则列表 |
| PUT | /devices/{id}/alarm-config/{alarmType}/{sensorCode} | 设置告警规则(10种类型) → MQTT delta |
| DELETE | /devices/{id}/alarm-config/{alarmType}/{sensorCode} | 删除告警规则 → MQTT delta |

## 读数

| GET | /devices/{id}/readings/latest | 最新值 (近1h去重) |
| GET | /devices/{id}/readings/history | 历史 (sensorCode/from/to) |

## 仪表盘

| GET | /dashboard/overview | 概览统计 |
