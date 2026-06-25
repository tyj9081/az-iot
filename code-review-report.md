# 🔍 AZ-IOT 代码审查报告

> **审查人**：Senior Developer（高级开发工程师）  
> **日期**：2026-06-26  
> **项目**：AZ-IOT 能源管理物联网平台  

---

## 📊 整体评估

| 维度 | 评分 | 说明 |
|------|------|------|
| 架构设计 | ⭐⭐⭐⭐ | 三端分离清晰，Maven/Cargo Workspace 多模块结构合理 |
| 代码规范 | ⭐⭐⭐ | 命名风格基本统一，但类型安全严重不足 |
| 安全性 | ⭐⭐ | 多个关键安全隐患，需立即修复 |
| 可维护性 | ⭐⭐⭐ | 分层结构好，但部分模块空实现 |
| 前端质量 | ⭐⭐⭐ | Element Plus 使用规范，但 `any` 滥用严重 |
| 测试覆盖 | ⭐⭐ | 有部分测试，覆盖率不足 |

---

## 🔴 P0 - 严重问题（阻塞上线）

### 1. 退出登录功能完全未实现

**文件**：`neuron-web/src/views/layout/index.vue:26`
```typescript
const logout = () => { /* Task 10 实现 */ }
```

**风险**：用户无法安全退出，Session 无法清除。

**修复**：
```typescript
import { useAuthStore } from '@/stores/auth'
import { useRouter } from 'vue-router'
const authStore = useAuthStore()
const router = useRouter()

const logout = async () => {
  await authApi.logout().catch(() => {})
  authStore.clearAuth()
  localStorage.removeItem('refresh_token')
  router.push('/login')
}
```

---

### 2. 默认用户名密码硬编码在源码中

**文件**：`neuron-web/src/views/login/index.vue:33`
```typescript
const form = reactive({ username: 'admin', password: 'admin123' })
```

**风险**：源码泄露即账号泄露，且容易被自动化扫描。

**修复**：移除默认值，将 `username` 和 `password` 初始化为空字符串：
```typescript
const form = reactive({ username: '', password: '' })
```

---

### 3. JWT 密钥硬编码且长度不足

**文件**：`neuron-platform/neuron-server/src/main/resources/application.yml:19`
```yaml
jwt:
  secret: ${JWT_SECRET:az-iot-jwt-secret-key-must-be-at-least-256-bits-long}
```

**风险**：
- 默认密钥仅有 45 字符（HMAC-SHA256 至少需要 32 字节 = 256 bits）
- 提交到 Git 成为公开信息
- 生产环境若无环境变量覆盖，直接使用弱密钥

**修复**：
```yaml
# 移除默认值，强制通过环境变量注入
jwt:
  secret: ${JWT_SECRET}  # 部署时必须设置，至少 256 bits 随机字符串
```

同时使用 `Keys.secretKeyFor(SignatureAlgorithm.HS256)` 自动生成合规密钥用于本地开发。

---

### 4. Token 存储在 localStorage（XSS 漏洞）

**文件**：`neuron-web/src/stores/auth.ts`、`neuron-web/src/utils/request.ts`

**风险**：任何注入到页面的恶意脚本都能读取 `localStorage.getItem('access_token')`，直接窃取凭证。

**修复方案**：
1. 后端设置 `httpOnly` + `Secure` + `SameSite=Strict` Cookie
2. 前端不再手动管理 Token，由浏览器自动携带
3. Spring Security 配置：
```java
http.csrf(csrf -> csrf
    .csrfTokenRepository(CookieCsrfTokenRepository.withHttpOnlyFalse())
```

---

### 5. AuthController 跳过 Service 层直接调用 Mapper

**文件**：`neuron-platform/neuron-controller/.../AuthController.java`

```java
private final SysUserMapper userMapper;  //  Controller 直接依赖 Mapper
```

**风险**：违反分层架构原则，业务逻辑无法复用，事务管理缺失。

**修复**：创建 `AuthService`，将登录逻辑、Token 刷新逻辑封装在 Service 层。

---

### 6. 使用 RuntimeException 抛出业务异常

**文件**：`AuthController.java:30-33`
```java
throw new RuntimeException("账号不存在或已禁用");
throw new RuntimeException("密码错误");
```

**风险**：统一异常处理器将 RuntimeException 一律返回 HTTP 500，前端无法区分业务错误类型。

**修复**：使用已有的 `BusinessException`：
```java
throw new BusinessException(401, "账号或密码错误");
// 不要分别提示"账号不存在"和"密码错误"——这会暴露用户枚举漏洞
```

---

### 7. Rust 采集端 Mock 驱动的随机数生成器有 Bug

**文件**：`neuron-collector/collector-driver/src/lib.rs:52-58`
```rust
fn rand_val() -> f64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut h = RandomState::new().build_hasher();
    h.write_u64(0);
    (h.finish() % 100) as f64 / 100.0
}
```

**风险**：
- `RandomState` 每次调用 `new()` 使用相同的种子（进程级），返回值是**确定的**而非随机
- 这不是正确的随机数生成方式

**修复**：引入 `rand` crate 或使用线程局部随机数：
```toml
[dependencies]
rand = "0.8"
```
```rust
fn rand_val() -> f64 {
    rand::random::<f64>()
}
```

---

### 8. 三个 Rust 模块完全空实现

| 模块 | 文件 | 状态 |
|------|------|------|
| `collector-config-sync` | `src/lib.rs` | `pub fn init() { // Phase 3 实现 }` |
| `collector-storage` | `src/lib.rs` | `pub fn init() { // Phase 6 实现 }` |
| `collector-telemetry` | `src/lib.rs` | `pub fn init() { // Phase 7 实现 }` |

**风险**：配置同步、本地存储、遥测上报全部缺失，采集器无法接收平台指令、无法持久化数据、无法上报健康状态。

---

## 🟠 P1 - 高优先级问题

### 9. TypeScript `any` 类型泛滥

**涉及文件**：几乎全部 Vue 组件和 API 文件

```typescript
const res: any = await deviceApi.list({...})     // device/index.vue:269
const userInfo = ref<any>(null)                    // stores/auth.ts:6
const overview = ref<any>(null)                    // dashboard/index.vue:57
```

**影响**：失去了 TypeScript 的类型检查能力，重构风险极高。

**修复**：充分利用已定义的 `types/api.d.ts` 中的类型：
```typescript
import type { ApiResponse, PageResult } from '@/types/api'
import type { DevDeviceVO } from '@/types/device'

const res = await deviceApi.list({...}) as ApiResponse<PageResult<DevDeviceVO>>
tableData.value = res.data.records
```

---

### 10. 登录接口无反爬/限流保护

**文件**：`AuthController.java:22-40`

**风险**：`POST /api/v1/auth/login` 可被暴力破解。

**修复**：
1. 引入 Spring 的 Bucket4j 或 Guava RateLimiter
2. 基于 IP + username 双重限流（5 次/分钟失败即锁定 15 分钟）
3. 登录失败返回统一错误信息"用户名或密码错误"

---

### 11. ConfigPushService 异常被静默吞掉

**文件**：`ConfigPushService.java:80-82`
```java
} catch (Exception e) {
    log.error("Config push failed for device " + deviceId, e);
}
```

**风险**：MQTT 推送失败不影响数据库操作，设备和采集器状态可能出现不一致。

**修复**：
- 引入重试机制（至少 3 次指数退避）
- 记录失败事件到 `sys_audit_log` 或专用表
- 提供管理端"重新下发"按钮

---

### 12. CORS 未配置

**文件**：`SecurityConfig.java`

**风险**：
- 开发阶段前端 `localhost:3000` 无法直接调用后端 `localhost:8080`（虽然 Vite Proxy 缓解了开发环境）
- 生产部署（不同域名）将完全无法工作

**修复**：
```java
http.cors(cors -> cors.configurationSource(corsConfigurationSource()));
```

---

### 13. 无数据库连接池配置

**文件**：`application.yml`

**风险**：默认 HikariCP 连接池仅 10 个连接，高并发下容易出现连接等待超时。

**修复**：
```yaml
spring:
  datasource:
    hikari:
      maximum-pool-size: 30
      minimum-idle: 5
      connection-timeout: 5000
      idle-timeout: 600000
      max-lifetime: 1800000
```

---

## 🟡 P2 - 中优先级问题

### 14. 下拉列表一次性加载全部数据（pageSize: 999）

**文件**：`neuron-web/src/views/device/index.vue:269`, `:278` 等多处
```typescript
const res: any = await collectorApi.list({ page: 1, pageSize: 999 })
```

**影响**：数据量大时性能下降、网络开销大。

**修复**：对于下拉选择器，后端提供专门的 `/api/v1/collectors/options` 轻量接口，仅返回 `id` + `name` + `code`。

---

### 15. Dashboard 数据加载失败无用户提示

**文件**：`neuron-web/src/views/dashboard/index.vue:80-83`
```typescript
} catch {
    overview.value = null
    recentReadings.value = []
}
```

**修复**：
```typescript
} catch (e: any) {
    overview.value = null
    recentReadings.value = []
    ElMessage.error('加载仪表盘数据失败：' + (e?.response?.data?.message || '网络错误'))
}
```

---

### 16. 采集器创建时串口参数硬编码

**文件**：`DevCollectorService.java:51`
```java
String busParam = "{\"baud\":9600,\"data_bits\":8,\"stop_bits\":1,\"parity\":\"none\"}";
```

**风险**：所有串口使用相同参数，不支持不同波特率、校验位配置。

**修复**：在创建采集器时提供串口参数配置表单，或在型号/设备级别覆盖。

---

### 17. 状态码字段类型不一致

- Java `DevDevice.status`: `String` ("online"/"offline")
- TypeScript `handleDisable`: 传入 `1` / `0`（数字）

**修复**：统一使用枚举或字符串常量，前后端保持一致。

---

### 18. Refresh Token 类型检查缺失

**文件**：`AuthController.java:43-54`

`/refresh` 接口未检查 Token 的 `type` claim 是否为 `"refresh"`，意味着 Access Token 也能被用来刷新——失去了双 Token 的安全价值。

**修复**：
```java
String tokenType = parseClaims(refreshToken).get("type", String.class);
if (!"refresh".equals(tokenType)) {
    throw new BusinessException(401, "无效的刷新令牌");
}
```

---

## 🟢 P3 - 优化建议

### 前端优化

1. **Element Plus 全量引入** → 改为按需引入或使用 `unplugin-element-plus` 自动导入
2. **无自动刷新 Token** → 在 `request.ts` 拦截器中实现无感刷新
3. **缺乏移动端适配** → 采集器管理、设备详情等页面未做响应式处理
4. **favicon 缺失** → `index.html` 引用 `/vite.svg` 不存在
5. **没有 Loading/Skeleton 统一处理** → 建议封装 `useRequest` composable

### 后端优化

1. **缺少 Swagger/Knife4j API 文档** → 加上 `springdoc-openapi-starter-webmvc-ui`
2. **缺少健康检查端点** → 引入 `spring-boot-starter-actuator`
3. **无环境分离** → 增加 `application-dev.yml` / `application-prod.yml`
4. **密码强度无校验** → 登录无锁定、密码无复杂度要求
5. **MQTT 客户端版本过旧** → `org.eclipse.paho.client.mqttv3:1.2.5` 发布于 2019 年

### Rust 采集端优化

1. **添加 `Cargo.toml` 依赖 `rand`** 替代有 Bug 的随机数生成
2. **实现 `collector-config-sync`**：订阅 MQTT `neuron/{client_id}/config/delta` 主题
3. **实现 `collector-storage`**：SQLite 本地持久化采集数据
4. **实现 `collector-telemetry`**：定期上报 CPU/内存/磁盘使用率
5. **添加 `clap`** 命令行参数解析（`--mqtt-host`, `--client-id` 等）

### 工程化建议

1. **添加 `.editorconfig`** 统一代码风格
2. **前端添加 ESLint + Prettier** 配置
3. **后端添加 Checkstyle / SpotBugs**
4. **编写 Dockerfile** 用于容器化部署
5. **添加 GitHub Actions / Jenkins CI** 流水线
6. **添加 `docker-compose.yml`** 一键启动 EMQX + MySQL + Redis + 应用

---

## 📋 修复优先级 checklist

### 🚨 本周必修
- [ ] 实现退出登录功能
- [ ] 移除前端硬编码默认密码
- [ ] JWT 密钥外置，移除默认值
- [ ] Token 改为 httpOnly Cookie 或至少解决 XSS
- [ ] AuthController 改为调用 Service 层

### ⚡ 迭代二必修
- [ ] TypeScript `any` 类型全部替换为具体类型
- [ ] 登录接口加限流
- [ ] ConfigPushService 加重试机制
- [ ] 配置 CORS
- [ ] 实现三个空 Rust 模块的基础功能

### 📌 迭代三规划
- [ ] 数据库连接池调优
- [ ] 实现下拉选项专用 API
- [ ] 添加 Swagger 文档
- [ ] 修复 Refresh Token 类型校验
- [ ] 补全测试用例

---

> **总结**：项目架构设计良好，三端分离、协议分层清晰。主要问题集中在**安全实现**（JWT/Token/XSS）和**代码健壮性**（类型安全/异常处理）。建议按 P0 → P1 → P2 优先级逐一修复，不要积累技术债务。
