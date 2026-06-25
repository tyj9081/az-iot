# AZ-IOT

面向工业园区/楼宇的能源管理物联网平台。

```
采集端数据获取 → 服务端存储加工 → 前端展示监控
```

## 技术栈

| 层 | 技术 |
|---|------|
| 采集端 | Rust (tokio + rumqttc + tokio-modbus) |
| 服务端 | Java 17 (Spring Boot 3.2 + MyBatis-Plus) |
| 前端 | Vue 3 (Vite + Element Plus + ECharts + Pinia) |
| 消息 | EMQX 5.x (MQTT Broker) |
| 数据库 | MySQL 8.0 + Redis 7.x |
| 采集终端 | BC-U101 (x86 工控机, 2×RS232 + 6×RS485 + 6DI + 2DO + 短信猫) |

## 项目结构

```
az-iot/
├── neuron-platform/          # Java 服务端 (Maven 多模块)
├── neuron-collector/         # Rust 采集端 (Cargo Workspace)
├── neuron-web/               # Vue 3 前端 SPA
├── docs/                     # 项目文档
│   └── 开发总纲.md
└── README.md
```

## 架构

```
neuron-web ──HTTP──→ neuron-server ──MQTT──→ neuron-collector ──→ BC-U101 → 设备
```

## 核心能力 (MVP)

- 品牌/型号/协议/点表管理，支持 Excel 导入寄存器映射
- BC-U101 采集器注册 + 串口管理 (COM1-COM10)
- 设备绑定型号+串口+从站地址 → 一键 MQTT 下发
- 双通道数据上报: 实时(WebSocket) + 历史(MySQL 分区)
- 点表驱动通用渲染器 (数值卡片/状态灯/趋势图)
- JWT 认证 + RBAC + 审计日志

## 开发规范

详见 [docs/开发总纲.md](docs/开发总纲.md)
