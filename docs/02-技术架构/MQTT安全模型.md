# MQTT 安全模型

> AZ-IOT 服务端 ←→ 采集端 | ADR-07

---

## 威胁分析

| 威胁 | 风险 | 对策 |
|------|------|------|
| 裸 TCP 嗅探 | 高 — 明文传输所有读数和控制指令 | TLS 1.2+ 加密传输 |
| 未授权连接 | 高 — 任何人可注入伪造数据 | 用户名+密码认证 + EMQX ACL |
| 采集器冒名顶替 | 高 — 伪造 client_id 上报假数据 | 独立凭证绑定 + ACL 隔离 |
| 权限越界 | 中 — 采集器可订阅其他采集器配置 | ACL: 只允许 `neuron/{own_id}/` |
| 凭证泄露 | 中 — 密码明文暴露 | BCrypt 哈希存储 + 定期轮换 |
| 重放攻击 | 低 — QoS 1 消息重复投递 | MQTT 消息去重 (EMQX 内置) |

---

## 三层安全体系

```
┌──────────────────────────────────────┐
│  采集端 (Rust)                       │
│  ┌─ TLS 1.2+ ──────────────────┐     │
│  │  证书: CA 签发 或 自签       │     │
│  └─────────────────────────────┘     │
│  ┌─ 认证 ──────────────────────┐     │
│  │  username + password         │     │
│  │  (服务端注册时生成)           │     │
│  └─────────────────────────────┘     │
└──────────────┬───────────────────────┘
               │ MQTT over TLS (port 8883)
        ┌──────▼──────┐
        │    EMQX     │
        │  ┌────────┐ │
        │  │TLS终结 │ │ ← 证书校验
        │  ├────────┤ │
        │  │HTTP认证│ │ ← POST → neuron-server /api/v1/mqtt/auth
        │  ├────────┤ │     验证 username/password
        │  │ ACL    │ │ ← 内置规则引擎
        │  └────────┘ │     neuron/{client_id}/... 
        └──────┬──────┘
               │
┌──────────────▼───────────────────────┐
│  服务端 (Java)                       │
│  ┌─ MQTT Auth Callback ────────┐     │
│  │  EMQX 收到 CONNECT → HTTP   │     │
│  │  POST /mqtt/auth → 查 DB   │     │
│  │  验证凭证 → 返回 允许/拒绝   │     │
│  └─────────────────────────────┘     │
│  独立管理凭证连接 EMQX               │
└──────────────────────────────────────┘
```

---

## 凭证生命周期

### 1. 注册时生成

```java
// DevCollectorService.create():
String username = "col-" + RandomStringUtils.randomAlphanumeric(8);
String rawPassword = RandomStringUtils.randomAlphanumeric(16);
String passwordHash = passwordEncoder.encode(rawPassword);

collector.setMqttUsername(username);
collector.setMqttPasswordHash(passwordHash);
// rawPassword 仅在创建响应中返回一次——不存储明文
```

### 2. 部署时配置

运维人员在 Web 界面创建采集器后，看到一次性显示的凭证 → SSH 到 BC-U101 配置:

```toml
# config.toml
[mqtt]
broker = "emqx.example.com"
port = 8883
client_id = "BC-U101-001"
username = "col-Ab3dEf7x"
password = "K9m2Wp5Rq8NtL4vH"
tls_enabled = true
tls_ca_cert = "/etc/neuron/ca.crt"
```

### 3. 凭证轮换

Web 界面提供"重新生成凭证"按钮 → 新密码 → 服务端更新 `mqtt_password_hash` → 运维手动更新采集端 config.toml → 重启采集端

### 4. 吊销

删除/禁用采集器 → 服务端通过 EMQX HTTP API 踢出该 client_id → 凭证标记为失效

---

## EMQX ACL 规则

```erlang
%% 每个采集器只能在自己的 namespace 下操作
{allow, {username, "col-{id}"}, publish, ["neuron/{client_id}/#"]}.
{allow, {username, "col-{id}"}, subscribe, ["neuron/{client_id}/config/#", "neuron/{client_id}/cmd"]}.

%% 服务端管理账号可操作所有 namespace
{allow, {username, "neuron-server"}, publish, ["neuron/+/config/#", "neuron/+/cmd"]}.
{allow, {username, "neuron-server"}, subscribe, ["neuron/+/#"]}.

%% 默认拒绝
{deny, all, publish, ["#"]}.
{deny, all, subscribe, ["#"]}.
```

---

## config.toml 模板

```toml
[mqtt]
broker = "localhost"
port = 1883

# 认证 (首次部署时由运维从 Web 界面复制)
username = ""
password = ""

# TLS (生产环境必须启用)
tls_enabled = false
tls_ca_cert = ""     # CA 证书路径, 如 /etc/neuron/ca.crt
tls_client_cert = "" # 客户端证书 (双向认证时使用)
tls_client_key = ""  # 客户端私钥

keepalive = 60
qos = 1
```

---

## 内网环境降级策略

如果 BC-U101 部署在内网且网络已物理隔离，可接受降级:

| 环境 | TLS | 凭证认证 | ACL |
|------|-----|---------|-----|
| 公网/混合云 | **强制** | **强制** | **强制** |
| 内网物理隔离 | 可选 | 强制 | 强制 |
| 开发环境 | 关闭 | 关闭 | 关闭 |

- 内网环境跳过 TLS 不意味着跳过认证——即使局域网内，用户名+密码认证也必须开启
- EMQX 的 HTTP Auth 回调指向服务端的 `/api/v1/mqtt/auth` 端点，该端点不经过 JWT Filter (需要单独放行)<｜end▁of▁thinking｜>

<｜｜DSML｜｜tool_calls>
<｜｜DSML｜｜invoke name="Write">
<｜｜DSML｜｜parameter name="content" string="true"># MQTT 安全模型

> AZ-IOT 服务端 ←→ 采集端 | ADR-07 | 2026-06-26

---

## 威胁矩阵

| 威胁 | 风险等级 | 对策 |
|------|---------|------|
| 裸 TCP 嗅探 (明文读数/指令) | 高 | TLS 1.2+ 加密 |
| 未授权连接 (注入伪造数据) | 高 | 用户名+密码认证 + EMQX ACL |
| 采集器冒名 (伪造 client_id) | 高 | 独立凭证绑定 + ACL namespace 隔离 |
| 权限越界 (订阅他人配置) | 中 | ACL: 仅允许 `neuron/{own_id}/` |
| 凭证泄露 (密码明文) | 中 | BCrypt 哈希存储 + 定期轮换 |
| 重放攻击 (QoS 1 重复) | 低 | EMQX 消息去重 (内置) |

---

## 三层体系

```
采集端 ──TLS 1.2+──→ EMQX ──TLS 1.2+──→ 服务端
          ↑                      ↑
    客户端证书(可选)        HTTP Auth 回调
                           POST /api/v1/mqtt/auth
                           验证 username/password
```

### 传输层 — TLS
- EMQX 开启 TLS 监听 (端口 8883)
- 内网物理隔离可降级跳过 TLS，但**认证和 ACL 不可跳过**

### 认证层 — 独立凭证

| 凭证 | 生成 | 存储 |
|------|------|------|
| mqtt_username | 服务端注册采集器时自动生成 `col-{8位随机}` | dev_collector 表明文 |
| mqtt_password | 随机 16 位, 创建响应中**仅返回一次** | dev_collector 表 BCrypt 哈希 |
| 服务端管理凭证 | 环境变量 `MQTT_ADMIN_USER` / `MQTT_ADMIN_PASS` | 不入表 |

### 授权层 — EMQX ACL

| 角色 | 允许 PUB | 允许 SUB |
|------|---------|---------|
| 采集器 (id=xxx) | `neuron/xxx/reading`, `latest`, `heartbeat`, `alarm`, `config/ack` | `neuron/xxx/config/#`, `cmd` |
| 服务端 | `neuron/+/config/#`, `cmd` | `neuron/+/#` (全部) |
| **默认** | **全部拒绝** | **全部拒绝** |

---

## 凭证生命周期

1. **生成**: 创建采集器 → 服务端 `RandomStringUtils` 生成 → password 仅一次返回 → 存 hash
2. **部署**: 运维人员 Web 界面复制凭证 → SSH 到 BC-U101 → 编辑 `config.toml`
3. **轮换**: Web "重置凭证" → 服务端更新 hash → 运维手动更新 config.toml → 重启采集端
4. **吊销**: 禁用/删除采集器 → EMQX HTTP API 踢出 client_id

---

## config.toml 模板

```toml
[mqtt]
broker = "localhost"
port = 1883
username = ""       # 运维从 Web 复制
password = ""       # 一次性显示
tls_enabled = false
tls_ca_cert = ""    # CA 证书路径
keepalive = 60
```

---

## 环境降级策略

| 环境 | TLS | 凭证认证 | ACL |
|------|-----|---------|-----|
| 公网/混合云 | 强制 | 强制 | 强制 |
| 内网物理隔离 | 可选 | 强制 | 强制 |
| 本地开发 | 关闭 | 关闭 | 关闭 |
