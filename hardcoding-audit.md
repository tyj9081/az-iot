# 🚫 全项目硬编码专项审查报告

> **审查时间**：2026-06-26 01:32  
> **原则**：敏感信息、环境相关配置、业务常量一律外置，零容忍

---

## 📊 总计：发现 **14 项硬编码违规**，① 凭据类：4 项 | ② 地址类：5 项 | ③ 魔法数字类：5 项

---

## 🔴 ① 凭据 / 密钥硬编码（4 项）

| # | 文件 | 行 | 硬编码内容 | 风险等级 |
|---|------|----|-----------|----------|
| 1 | `neuron-web/src/views/login/index.vue` | 33 | `password: 'admin123'` | 🔴严重 |
| 2 | `neuron-platform/.../application.yml` | 7 | `password: ${DB_PASSWORD:root}` | 🔴严重 |
| 3 | `neuron-platform/.../application.yml` | 6 | `username: root` | 🔴严重 |
| 4 | `neuron-platform/.../application.yml` | 19 | `secret: ${JWT_SECRET:az-iot-jwt-secret-key...}` | 🔴严重 |

---

### 🔧 修复方案

#### #1 移除前端硬编码默认密码

**文件**：`neuron-web/src/views/login/index.vue:33`

```diff
- const form = reactive({ username: 'admin', password: 'admin123' })
+ const form = reactive({ username: '', password: '' })
```

---

#### #2~4 移除 application.yml 中所有默认值

**文件**：`neuron-platform/neuron-server/src/main/resources/application.yml`

```diff
  spring:
    datasource:
      url: jdbc:mysql://localhost:3306/az_iot?useUnicode=true&characterEncoding=utf8mb4&useSSL=false&serverTimezone=Asia/Shanghai&allowPublicKeyRetrieval=true
-     username: root
-     password: ${DB_PASSWORD:root}
+     username: ${DB_USERNAME}
+     password: ${DB_PASSWORD}

  jwt:
-   secret: ${JWT_SECRET:az-iot-jwt-secret-key-must-be-at-least-256-bits-long}
+   secret: ${JWT_SECRET}
```

**⚠️ 关键**：移除所有 `:fallback` 语法，强制启动时校验环境变量是否已设置。启动类加校验：

```java
@PostConstruct
void validateConfig() {
    if (jwtSecret == null || jwtSecret.length() < 32) {
        throw new IllegalStateException("JWT_SECRET 至少需要 256 bits (32 字节)");
    }
}
```

---

## 🟠 ② 地址 / URL 硬编码（5 项）

| # | 文件 | 行 | 硬编码内容 |
|---|------|----|-----------|
| 5 | `neuron-web/vite.config.ts` | 16 | `target: 'http://localhost:8080'` |
| 6 | `neuron-web/vite.config.ts` | 13 | `port: 3000` |
| 7 | `neuron-web/src/utils/request.ts` | 5 | `baseURL: '/api/v1'` |
| 8 | `neuron-platform/.../application.yml` | 5 | `jdbc:mysql://localhost:3306/...` |
| 9 | `neuron-platform/.../application.yml` | 24 | `server.port: 8080` |

---

### 🔧 修复方案

#### #5 Vite 代理目标

```diff
  // vite.config.ts
+ const API_TARGET = process.env.VITE_API_TARGET || 'http://localhost:8080'

  server: {
-   port: 3000,
+   port: Number(process.env.VITE_DEV_PORT) || 3000,
    proxy: {
      '/api': {
-       target: 'http://localhost:8080',
+       target: API_TARGET,
        changeOrigin: true
      }
    }
  }
```

#### #6 API baseURL 从环境变量读取

```diff
  // src/utils/request.ts
  const http = axios.create({
-   baseURL: '/api/v1',
+   baseURL: import.meta.env.VITE_API_BASE_URL || '/api/v1',
    timeout: 15000,
  })
```

#### #8~9 数据库 & 端口全外置

```diff
  spring:
    datasource:
-     url: jdbc:mysql://localhost:3306/az_iot?useUnicode=true&...
+     url: ${DB_URL}
-     username: ${DB_USERNAME}
+     username: ${DB_USERNAME}
-     password: ${DB_PASSWORD}
+     password: ${DB_PASSWORD}

  server:
-   port: 8080
+   port: ${SERVER_PORT:8080}    // 端口允许默认值，不属于敏感信息
```

**⚠️ 注意**：MySQL 连接参数 `useSSL=false` + `allowPublicKeyRetrieval=true` 在生产环境应改为 `useSSL=true` + 移除 `allowPublicKeyRetrieval`。

---

## 🟡 ③ 魔法数字硬编码（5 项）

| # | 文件 | 行 | 值 | 含义 |
|---|------|----|-----|------|
| 10 | `device/index.vue` | 269,278 | `pageSize: 999` | 全量下拉 |
| 11 | `device/index.vue` | 341,347 | `pageSize: 999` | 编辑回显全量下拉 |
| 12 | `device-model/index.vue` | 188 | `pageSize: 9999` | 全量下拉 |
| 13 | `application.yml` | 20 | `access-token-expiration: 1800000` | Token 过期 |
| 14 | `application.yml` | 21 | `refresh-token-expiration: 604800000` | 刷新过期 |

---

### 🔧 修复方案

#### #10~12 移除 "pageSize: 999" 全量加载反模式

**问题**：用 `pageSize: 999` 加载全量数据放入下拉框，数据量大时不可接受。

**方案一（推荐）**：后端提供专用下拉接口

```java
// 新增轻量接口
@GetMapping("/options")
public ApiResponse<List<IdNameVO>> options() {
    return ApiResponse.ok(collectorService.listOptions());
}
```
```sql
SELECT id, code, name FROM dev_collector WHERE status = 'online' ORDER BY name
```

```typescript
// 前端使用
const res = await collectorApi.options()  // 不再传 pageSize
collectorOptions.value = res.data
```

**方案二（临时）**：使用 Element Plus 远程搜索

```vue
<el-select
  v-model="form.collectorId"
  filterable
  remote
  :remote-method="searchCollector"
  :loading="collectorSearchLoading"
>
```

#### #13~14 Token 过期时间外置

```diff
  jwt:
    secret: ${JWT_SECRET}
-   access-token-expiration: 1800000
+   access-token-expiration: ${JWT_ACCESS_EXPIRATION:1800000}
-   refresh-token-expiration: 604800000
+   refresh-token-expiration: ${JWT_REFRESH_EXPIRATION:604800000}
```

> 时间值允许默认值，因为不属于敏感信息。但建议分类到 `application-prod.yml` / `application-dev.yml`。

---

## 📋 修复 Checklist

```
┌──────────────────────────────────────────────────────────┐
│  立即修复（P0 - 凭据泄露风险）                              │
├──────────────────────────────────────────────────────────┤
│  [ ] login/index.vue     移除 admin/admin123 默认值       │
│  [ ] application.yml     DB_PASSWORD:root → ${DB_PASSWORD}│
│  [ ] application.yml     DB_USERNAME:root → ${DB_USERNAME}│
│  [ ] application.yml     JWT_SECRET 移除默认 fallback     │
│  [ ] JwtTokenProvider    启动时校验密钥长度 ≥ 256 bits     │
└──────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────┐
│  本周必修（P1 - 环境绑定）                                   │
├──────────────────────────────────────────────────────────┤
│  [ ] vite.config.ts      proxy target → 环境变量            │
│  [ ] request.ts          baseURL → 环境变量                │
│  [ ] application.yml     DB_URL → ${DB_URL}               │
│  [ ] application.yml     useSSL=true (生产环境)            │
│  [ ] 添加 .env.example   模板文件供团队参考                  │
└──────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────┐
│  迭代修复（P2 - 代码质量）                                   │
├──────────────────────────────────────────────────────────┤
│  [ ] 3 处 pageSize: 999  → 替换为专用下拉接口               │
│  [ ] 创建 application-dev.yml / application-prod.yml      │
│  [ ] 添加 .gitignore 确保 .env 不被提交                    │
└──────────────────────────────────────────────────────────┘
```

---

## 📎 补充：环境变量模板

在项目根目录创建 `.env.example`（可提交到 Git）：

```bash
# ===== neuron-platform (Java) =====
DB_URL=jdbc:mysql://your-host:3306/az_iot?useSSL=true&serverTimezone=Asia/Shanghai
DB_USERNAME=your_db_user
DB_PASSWORD=your_db_password
JWT_SECRET=<至少 256 bits 随机字符串，推荐 openssl rand -base64 64>
JWT_ACCESS_EXPIRATION=1800000
JWT_REFRESH_EXPIRATION=604800000
SERVER_PORT=8080

# ===== neuron-web (Vue) =====
VITE_API_BASE_URL=/api/v1
VITE_DEV_PORT=3000
VITE_API_TARGET=http://localhost:8080

# ===== neuron-collector (Rust) =====
MQTT_BROKER=tcp://your-broker:1883
MQTT_CLIENT_ID=collector-001
MQTT_USERNAME=your_mqtt_user
MQTT_PASSWORD=your_mqtt_password
```

---

> **红线**：即日起，所有包含 `password` / `secret` / `key` / `token` / `username` 的默认值，**一律不得通过 fallback 语法（`${VAR:default}`）提供**。启动时若缺少环境变量必须**立即报错停止**，不能静默使用弱默认值上线。
