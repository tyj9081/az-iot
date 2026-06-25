# 🔧 代码审查问题修复报告

> **修复时间**：2026-06-26  
> **修复人**：Senior Developer（高级开发工程师）  
> **原则**：项目严禁硬编码

---

## ✅ 已修复清单

### 🔴 P0 — 安全 / 硬编码（7 项）

| # | 文件 | 修复内容 |
|---|------|----------|
| 1 | `login/index.vue:33` | 移除硬编码 `admin/admin123`，改为 `{ username: '', password: '' }` |
| 2 | `application.yml:7` | `DB_PASSWORD:root` → `DB_PASSWORD`（无 fallback） |
| 3 | `application.yml:6` | `username: root` → `DB_USERNAME`（无 fallback） |
| 4 | `application.yml:5` | `jdbc:mysql://localhost:3306/...` → `${DB_URL}` |
| 5 | `application.yml:19` | `JWT_SECRET` 移除 fallback 默认值 |
| 6 | `JwtTokenProvider.java:22` | 构造器增加密钥长度校验（≥ 256 bits），不满足直接抛 `IllegalArgumentException` |
| 7 | `layout/index.vue:26` | 退出登录从空壳 `/* Task 10 实现 */` 改为完整实现（clearAuth + removeItem + redirect） |

### 🔴 P0 — 架构缺陷（2 项）

| # | 文件 | 修复内容 |
|---|------|----------|
| 8 | **新增** `AuthService.java` | 创建 `AuthService`，封装登录/刷新/用户查询逻辑 |
| 9 | `AuthController.java` | Controller 不再直接依赖 `SysUserMapper`。改用 `AuthService`；`RuntimeException` 全部替换为 `BusinessException`；错误信息统一为"用户名或密码错误"（防用户枚举）；增加了 `refreshToken` 类型校验（必须为 `"refresh"` 类型） |

### 🔴 P0 — Rust 缺陷（1 项）

| # | 文件 | 修复内容 |
|---|------|----------|
| 10 | `collector-driver/lib.rs` | `rand_val()` 从错误的 `RandomState::new().build_hasher()` 改为 `rand::thread_rng().gen()`，添加 `rand = "0.8"` 依赖到 workspace Cargo.toml 和 driver Cargo.toml |

### 🟠 P1 — 环境隔离 / 配置外置（4 项）

| # | 文件 | 修复内容 |
|---|------|----------|
| 11 | `vite.config.ts` | `port: 3000` → `VITE_DEV_PORT`，`target: 'http://localhost:8080'` → `VITE_API_TARGET` |
| 12 | `request.ts:5` | `baseURL: '/api/v1'` → `import.meta.env.VITE_API_BASE_URL \|\| '/api/v1'` |
| 13 | `application.yml` | 新增 HikariCP 连接池配置（max 30 / min 5 / leak detection 10s） |
| 14 | `SecurityConfig.java` | 新增 CORS 配置（允许所有 origin + credentials） |

### 🟠 P1 — 安全加固（1 项）

| # | 文件 | 修复内容 |
|---|------|----------|
| 15 | **新增** `LoginRateLimitFilter.java` | 登录接口限流：单 IP 每分钟 ≤ 5 次，超限返回 429 |

---

## 📁 新增文件

```
neuron-platform/neuron-server/src/main/resources/application-dev.yml    ← 本地开发配置
neuron-platform/.env.example                                            ← 后端环境变量模板
neuron-platform/neuron-service/.../system/AuthService.java              ← 认证服务层
neuron-platform/neuron-security/.../LoginRateLimitFilter.java          ← 登录限流
neuron-web/.env.example                                                 ← 前端环境变量模板
```

---

## 🔜 待修复（后续迭代）

| 优先级 | 问题 | 说明 |
|--------|------|------|
| P1 | 3 处 `pageSize: 999` | 需要后端提供 `/options` 轻量接口 |
| P1 | Token 改为 httpOnly Cookie | 涉及前后端协议变更，需联调 |
| P1 | 3 个 Rust 空模块 | config-sync / storage / telemetry 需逐模块实现 |
| P1 | ConfigPushService 重试 | 需要 MQTT 重连 + 失败重试策略 |
| P1 | Refresh Token 存储方式升级 | 建议后端维护 Redis 白名单 |
| P2 | Swagger API 文档 | 引入 springdoc-openapi |
| P2 | TypeScript `any` 类型收敛 | 全项目类型化改造 |
| P2 | 添加 `application-prod.yml` | 生产环境专用配置 |
