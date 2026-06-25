# AZ-IOT 系统文档

> 最后更新: 2026-06-26

---

## 1. 系统概述

AZ-IOT 是一个物联网数据采集与监控平台，包含：

- **采集器 (Collector)**: Rust 开发，运行在工控机/Windows PC 上，通过 Modbus RTU/TCP 采集设备数据
- **MQTT Broker**: EMQX 5.x，消息中间件，桥接采集器和后端
- **后端 (Server)**: Spring Boot 3.x + MyBatis-Plus，数据存储、配置管理、告警
- **前端 (Web)**: Vue 3 + Element Plus，可视化管理界面

---

## 2. 系统架构

```
┌─────────────┐    MQTT     ┌───────────┐    MQTT     ┌──────────────┐
│  Collector   │ ──────────→ │   EMQX    │ ──────────→ │  neuron-     │
│  (Rust/Win)  │ ←────────── │  Broker   │ ←────────── │  server      │
│              │  config↓    │           │  cmd/pub    │  (Java)      │
└─────────────┘             └───────────┘             └──────┬───────┘
                                                            │
                              MySQL ◄───────────────────────┘
                              Redis                              
```

### 数据流

1. **采集上行**: Collector → EMQX (`neuron/{device_id}/reading`) → `MqttSubscriberService` → MySQL `dev_device_reading`
2. **状态上报**: Collector → EMQX (`neuron/{device_id}/status`)
3. **配置下发**: 前端操作 → 后端 → `ConfigPushService` → EMQX (`neuron/{collector_id}/config/delta`) → Collector
4. **Http 上送备通道**: Collector → Nginx `/api/collector/ingest` → 后端

---

## 3. 部署信息

### 3.1 服务器

| 项目 | 详情 |
|------|------|
| 云平台 | 阿里云 ECS |
| IP | 8.163.61.99 |
| OS | Ubuntu |
| 内存 | 1.6GB |

### 3.2 开放端口

| 端口 | 服务 | 用途 |
|------|------|------|
| 19090 | Nginx | 前端访问入口 |
| 18083 | EMQX Dashboard | MQTT 管理界面 |
| 1883 | EMQX MQTT | 采集器连接 |
| 8080 (内网) | Spring Boot | 后端 API |
| 6379 (内网) | Redis | Token 黑名单/缓存 |
| 3306 (内网) | MySQL | 业务数据 |

### 3.3 部署路径

| 路径 | 内容 |
|------|------|
| `/opt/az-iot/neuron-server.jar` | 后端 fat jar |
| `/opt/neuron/web` | 前端静态文件 (Nginx 19090) |
| `/etc/nginx/conf.d/neuron.conf` | Nginx 配置 |
| `/etc/emqx/emqx.conf` | EMQX 配置 |

---

## 4. 组件配置

### 4.1 Nginx (前端)

```nginx
server {
    listen 19090;
    root /opt/neuron/web;
    index index.html;

    location /api/ {
        proxy_pass http://127.0.0.1:8080/api/;
    }
}
```

### 4.2 EMQX MQTT Broker

- **Dashboard**: http://8.163.61.99:18083 (admin / admin123)
- **MQTT 端口**: tcp://8.163.61.99:1883

#### MQTT 用户

| 用户名 | 密码 | 权限 |
|--------|------|------|
| `neuron-collector` | `collector2024!` | publish `neuron/+/reading`, `neuron/+/status`; subscribe `neuron/+/config/delta` |
| `neuron-server` | `server2024!` | publish `neuron/+/cmd`; subscribe `neuron/+/reading`, `neuron/+/status` |

#### ACL 规则

```
{allow, {username, "neuron-collector"}, publish, ["neuron/+/reading", "neuron/+/status"]}.
{allow, {username, "neuron-collector"}, subscribe, ["neuron/+/config/delta"]}.
{allow, {username, "neuron-server"}, publish, ["neuron/+/cmd"]}.
{allow, {username, "neuron-server"}, subscribe, ["neuron/+/reading", "neuron/+/status"]}.
{deny, all}.
```

### 4.3 后端 (Spring Boot)

**启动命令**:
```bash
java -Xms128m -Xmx200m -XX:+UseSerialGC \
  -jar /opt/az-iot/neuron-server.jar \
  --spring.profiles.active=dev \
  --app.mqtt.broker-url=tcp://localhost:1883 \
  --app.mqtt.username=neuron-server \
  --app.mqtt.password=server2024!
```

**环境变量 (必需)**:
```bash
export DB_URL=jdbc:mysql://localhost:3306/neuron_db?useUnicode=true&characterEncoding=UTF-8&serverTimezone=Asia/Shanghai
export DB_USERNAME=root
export DB_PASSWORD=***
export JWT_SECRET=***
```

### 4.4 Collector (Windows)

配置文件 `config.toml` (与 collector-bin.exe 同目录):

```toml
[mqtt]
broker = "tcp://8.163.61.99:1883"
client_id = "collector-win10-01"
username = "neuron-collector"
password = "collector2024!"
topic_prefix = "neuron"

[fallback]
ws_url = "ws://8.163.61.99:8080/ws/collector"
enabled = true
```

---

## 5. 数据库

- **数据库**: MySQL `neuron_db`
- **root 密码**: `root`
- **Flyway 版本**: 自动管理

### 核心表

| 表名 | 用途 |
|------|------|
| `sys_user` | 用户认证 (BCrypt) |
| `dev_collector` | 采集器注册 |
| `dev_device` | 设备定义 |
| `dev_device_reading` | 采集读数 (分区表) |
| `dev_register_map` | 寄存器映射 |
| `dev_serial_port` | 串口配置 |
| `dev_device_model` | 设备型号 |

---

## 6. 管理维护

### 6.1 后端重启

```bash
# 停掉旧进程
pkill -f neuron-server.jar
# 启动新进程
nohup java -Xms128m -Xmx200m -XX:+UseSerialGC \
  -jar /opt/az-iot/neuron-server.jar \
  --spring.profiles.active=dev \
  --app.mqtt.broker-url=tcp://localhost:1883 \
  --app.mqtt.username=neuron-server \
  --app.mqtt.password=server2024! \
  > /var/log/neuron-server.log 2>&1 &
```

### 6.2 EMQX 管理

```bash
# 查看状态
emqx ctl status
# 查看客户端
emqx ctl clients list
# 查看认证用户
curl -s -u admin:admin123 http://localhost:18083/api/v5/login | ...
```

### 6.3 前端登录

- **地址**: http://8.163.61.99:19090
- **账号**: admin
- **密码**: `123%456/789-admin`

---

## 7. GitHub 仓库

- **Repo**: github.com/tyj9081/az-iot
- **GitHub Actions**: 编译 Windows 采集器
- **Workflow**: `.github/workflows/build-collector-win.yml`

### 本地开发

```bash
# 克隆 (服务器无法直连 GitHub，用 gh api)
cd /opt/az-iot-src
gh api repos/tyj9081/az-iot/zipball/main -o az-iot.zip
unzip -q az-iot.zip

# 编译后端
cd tyj9081-az-iot-*/neuron-platform
mvn package -pl neuron-server -am -DskipTests

# 编译采集器
cd ../neuron-collector
cargo build --release -p collector-bin
```

---

## 8. 已知问题

1. **collector-driver 编译错误**: Rust 依赖版本不兼容(24 errors)，需要修复后再集成 driver
2. **阿里云安全组**: 19090/18083 端口需要手动在阿里云控制台开放
3. **Win10 cross-compile**: 本地 OOM，走 GitHub Actions；workflow 需要 `collector-driver` 修复后 work
