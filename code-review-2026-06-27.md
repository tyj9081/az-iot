# AZ-IOT 全栈代码审查报告

**日期**: 2026-06-27 | **审查范围**: Rust 采集端 / Java 服务端 / Vue3 前端  
**审查人**: Software Architect (Jeff) | **审查标准**: 开发规范 S01-S07 + 安全最佳实践 + 架构一致性

---

## 总览

| 层级 | 严重问题 | 中等问题 | 轻微问题 | 总评 |
|------|---------|---------|---------|------|
| 🦀 Rust 采集端 | 🔴 5 | 🟡 7 | 🟢 4 | 架构骨架良好，实现有硬伤 |
| ☕ Java 服务端 | 🔴 3 | 🟡 6 | 🟢 3 | 整体扎实，安全与边界需加强 |
| 🖥️ Vue3 前端 | 🟡 1 | 🟡 3 | 🟢 3 | 设计系统规范，类型安全不足 |

---

## 🦀 Rust 采集端 (neuron-collector)

### 🔴 严重问题

#### 1. Scheduler 主循环使用 `block_in_place` 会造成 Tokio 线程饥饿

**文件**: `collector-scheduler/src/lib.rs:96`
```rust
let result = tokio::task::block_in_place(|| driver.collect(&device));
```

**问题**: `block_in_place` 会占用当前 tokio worker 线程，而这个调用发生在 tokio 主循环的 `async fn run()` 中。当前 tokio runtime 默认只有 CPU 核心数的 worker 线程。如果有 8 个设备且 Tokio 只有 4 个 worker，后 4 个设备会等到前 4 个采集完成后才能开始。更重要的是，**所有 MQTT 事件循环、心跳等异步任务共享这些 worker 线程**——它们可能被饿死。

**修复建议**:
```rust
// 方案: 使用 spawn_blocking 将同步 I/O 卸到独立线程池
let device_clone = device.clone();
let result = tokio::task::spawn_blocking(move || {
    let driver = DriverFactory::create(&device_clone)?;
    driver.collect(&device_clone)
}).await?;
```

#### 2. Uploader MQTT 重连泄露 EventLoop

**文件**: `collector-uploader/src/lib.rs:156-159`
```rust
let (c, el) = AsyncClient::new(opts2, 100);
*mqtt_clone.write().await = Some(c);
*fail_count.write().await = 0;
let _ = el;  // ← eventloop 被丢弃！
```

**问题**: 每次 MQTT 断连重连时，新的 `AsyncClient` 创建的 eventloop 被丢弃（`let _ = el`），导致该连接的 incoming 消息无人处理。这意味着：
- 新连接的 ConnAck 永远不会被处理
- 新连接永远不会被标记为 active
- **重连机制形同虚设**

同时，之前 spawn 的 eventloop handler task 仍在运行旧的事件循环——那个已经断连的 client 会持续报错，触发新一轮重连，形成死循环。

**修复建议**: 使用 `tokio::select!` 或 `CancellationToken` 来管理 eventloop 生命周期，切换时取消旧任务。

#### 3. ConfigPushService delete 存在竞态条件

**文件**: `neuron-service/.../ConfigPushService.java` 配合 `DevDeviceService.java:234-240`
```java
// DevDeviceService.delete():
configPushService.pushDelta(id, "remove");  // 先推送
removeById(id);                              // 再删除
```

但 `pushDelta` 执行时需要查询 `DevDevice`——如果 DB 中还没删除，能找到。然而 `pushDelta` 内部查询 `serialPortMapper`、`collectorMapper` 等关联数据。如果这些关联在事务外被修改，可能导致采集器收到残缺的配置 JSON。

**修复建议**: 确保 `pushDelta` 在事务内执行，或使用"先构建 payload，再删除，再推送"的两阶段模式。

#### 4. 采集端 main.rs 使用 `unwrap()` 违反开发规范

**文件**: `collector-bin/src/main.rs:17`
```rust
let config = CollectorConfig::load().unwrap_or_default();
```

**问题**: 开发规范 S07 明确禁止 `unwrap()`。虽然这里用了 `unwrap_or_default()` 降级，但 `ConfigDelta` 测试代码中仍有 3 处 bare `unwrap()` (model/lib.rs:227, 232, 238)——这些测试代码也需要修正为 `expect()`。

#### 5. `application-dev.yml` 硬编码生产凭据

**文件**: `neuron-server/src/main/resources/application-dev.yml`
```
password: Gzb1qVL74XFmaueq    # MySQL 密码
password: emqx_neuron_2026     # MQTT 密码
```

**问题**: 数据库和 MQTT 密码明文写在 Git 跟踪的文件中。即使这是 dev 配置，它指向的是真实远程服务器 `8.163.61.99`。

**修复建议**: 立即（1）将凭据移到 `.env` 或环境变量；（2）如果已提交 Git，轮换密码；（3）将 `application-dev.yml` 加入 `.gitignore`。

---

### 🟡 中等问题

#### 6. `publish_ws` 实际用的是 HTTP POST 而非 WebSocket

**文件**: `collector-uploader/src/lib.rs:274-314`

`publish_ws()` 方法名暗示通过 WebSocket 连接发送数据，但实际代码使用 `reqwest::Client::post()` 发 HTTP 请求。`try_ws_connect()` 建立的 WebSocket 连接只用于 keep-alive，不用于实际数据传输。"WS fallback" 的语义不实。

**建议**: 重命名为 `publish_http_fallback()` 或真正实现 WebSocket 消息发送。

#### 7. Aggregator 输出数据不完整

**文件**: `collector-reporter/src/lib.rs:70-78`
```rust
AggregatedReading {
    device_id: 0,           // 硬编码
    window_start: "".into(), // 空字符串
    window_end: "".into(),   // 空字符串
}
```

**问题**: 聚合读数缺少关键标识信息。下游无法区分哪个设备的聚合窗口。

#### 8. Telemetry 的 publisher 回调未接入实际 MQTT

**文件**: `collector-telemetry/src/lib.rs:39`
```rust
publisher: Arc<dyn Fn(String, String) + Send + Sync>,
```

**问题**: `collector-bin/main.rs` 中没有实例化 `Telemetry` 结构体，也没提供回调闭包。遥测模块是**死代码**，不会发布心跳。

#### 9. Scheduler 每轮循环全量克隆设备列表

**文件**: `collector-scheduler/src/lib.rs:81-84`
```rust
let devices: Vec<Device> = {
    let registry = self.registry.read().await;
    registry.devices.values().cloned().collect()
};
```

**问题**: 每秒克隆全部 Device（含 data_points、alarm_config），分配压力大。100 个设备、每个 50 个点位 = 每秒分配 5000 个 DataPoint。

**建议**: 只 clone 采集所需字段，或用 Arc 共享 Device 数据。

#### 10. Storage 在 async 上下文中使用同步 Mutex

**文件**: `collector-storage/src/lib.rs:47-48`
```rust
writer: Mutex<File>,      // std::sync::Mutex
next_id: Mutex<u64>,
```

`save_batch()` 中的 `self.writer.lock().unwrap()` 如果在 async 上下文中被调用且锁被持有，会阻塞整个 OS 线程。理论上目前不会发生（因为 storage 尚未被实际调用），但一旦集成到 scheduler 中就会成为问题。

#### 11. 重复的 `parse_broker_url` 函数

**文件**: `collector-uploader/src/lib.rs:330-340` 和 `collector-config-sync/src/lib.rs:70-78`

两份完全相同的 URL 解析代码。应提取到 `collector-model` 或新建 `collector-common` crate。

#### 12. MQTT 配置下发中的 TCP 端口硬编码

**文件**: `ConfigPushService.java:62`
```java
tcp.put("port", 502);  // 硬编码 Modbus TCP 默认端口
```

不同 TCP 协议的默认端口不同（IEC104=2404, S7=102, OPC UA=4840）。采集端需要的是实际配置的端口。

---

### 🟢 轻微问题

13. **Modbus 功能码解析失败时静默回退**: `driver/modbus.rs:22` → `pt.func_code.parse().unwrap_or(3)`，函数码解析失败时会静默使用 03（读保持寄存器），可能读到错误数据而非报错。

14. **collector-model 测试使用 `unwrap()`**: 3 处测试代码违反 S07 规范。

15. **Uploader 的 `connect_mqtt` 在 `new()` 中同步调用**: 如果 MQTT broker 不可达，Uploader 创建会卡住整个启动流程。应改为 lazy connect。

16. **`debug.log` 文件被 Git 跟踪**: 项目根目录的 `debug.log` 应加入 `.gitignore`。

---

## ☕ Java 服务端 (neuron-platform)

### 🔴 严重问题

#### 1. CORS 配置过于宽松

**文件**: `SecurityConfig.java:42-46`
```java
config.setAllowedOriginPatterns(List.of("*"));  // 允许任意来源
config.setAllowCredentials(true);               // 允许携带凭证
```

**问题**: `allowCredentials(true)` + `allowedOriginPatterns("*")` 的组合是 OWASP Top 10 级别的安全漏洞。任何恶意网站都可以发起携带 cookie/token 的跨域请求。

**修复**: 改为白名单模式，至少指定前端域名。

#### 2. `AuthController.me` 缺少空指针保护

**文件**: `AuthController.java:42-44`
```java
public ApiResponse<Map<String, Object>> me(@RequestHeader("Authorization") String auth) {
    String token = auth.substring(7);  // ← 如果 auth 为 null？
```

如果请求不带 Authorization header，Spring 会在框架层抛出 `MissingRequestHeaderException`（被 GlobalExceptionHandler 兜底），但 `auth.substring(7)` 没有校验 Bearer 前缀。

**修复**: 此逻辑应移到 `JwtAuthenticationFilter` 统一处理，Controller 从 SecurityContext 获取用户。

#### 3. DeviceController.create 暴露 Entity 到 API

**文件**: `DeviceController.java:41-53`
```java
@PostMapping
public ApiResponse<DevDevice> create(@RequestBody DeviceCreateDTO dto) {
    // ...
    return ApiResponse.ok(device);  // ← 返回 Entity！
}
```

**问题**: 违反开发规范 S03"禁止 Controller 暴露 Entity"。`DevDevice` 包含数据库内部字段。其他方法正确返回 `DevDeviceVO`。

---

### 🟡 中等问题

#### 4. `ConfigPushService.pushDelta` 吞异常

**文件**: `ConfigPushService.java:115-117`
```java
} catch (Exception e) {
    log.error("Config push failed for device " + deviceId, e);
}
```

**问题**: 配置下发失败被静默吞掉。调用方（`DevDeviceService.create/update/delete`）的无感知——设备创建成功但采集器永远不会收到配置。

**建议**: 至少抛出非阻断异常通知调用方，或写入待重试队列。

#### 5. MqttPublisherService 在 Spring 启动时同步阻塞

**文件**: `MqttPublisherService.java:102`
```java
client.connect(options).waitForCompletion(connTimeout * 1000L);
```

**问题**: 如果 EMQX 不可达，Spring Boot 启动会被阻塞 30 秒。结合 `@PostConstruct`，这会影响整个应用启动时间。

**建议**: 使用 `connect(options, null, new IMqttActionListener {...})` 异步连接。

#### 6. 缺少请求频率限制

`LoginRateLimitFilter` 类存在但可能未被正确注册到过滤器链。若未生效，登录接口可被暴力破解。结合已有的"不区分用户名不存在和密码错误"的防护策略，速率限制是必要的补充。

#### 7. 审计日志模块未记录请求体

`AuditLogAspect` 记录操作日志，但从代码看可能只捕获方法调用，未记录请求参数详情。对于配置变更操作（设备增删改），这是合规风险。

#### 8. DeviceService 缺少分页上限保护

`DevDeviceService.page()` 没有 pageSize 上限，攻击者可传 `pageSize=9999999` 触发大量查询。

#### 9. Flyway 版本号不连续

V1~V16 有 22 个迁移文件，但版本号相同（V1.1~V1.5）的文件分散在不同模块。建议统一版本号管理以避免执行顺序问题。

---

### 🟢 轻微问题

10. **JWT 密钥长度校验存在时序攻击可能**: `JwtTokenProvider` 构造函数中的 `secret.length()` 检查看起来没问题，但后续 `getBytes(StandardCharsets.UTF_8)` 使用固定编码是好的。

11. **`authService.getCurrentUser` 返回完整 Entity**: 用户密码哈希也包含在返回对象中（虽然 JSON 序列化时可能被 `@JsonIgnore` 过滤，但需确认）。

12. **部分 Service 没有 `@Transactional(readOnly = true)`**: 查询方法（如 `pageWithDetails`）应标记为只读事务以优化数据库连接。

---

## 🖥️ Vue3 前端 (neuron-web)

### 🟡 中等问题

#### 1. Token 存储在 localStorage（XSS 风险）

**文件**: `stores/auth.ts:5`, `utils/request.ts:11`
```typescript
const token = ref(localStorage.getItem('access_token') || '')
localStorage.setItem('refresh_token', res.data.refreshToken)
```

**问题**: localStorage 可被 XSS 攻击读取。对于 IIoT 平台（涉及物理设备控制），令牌泄露后果严重。

**建议**: 短期使用 httpOnly cookie + CSRF token；长期使用 BFF 模式。

#### 2. 401 拦截器使用硬页面跳转

**文件**: `utils/request.ts:19-23`
```typescript
if (error.response?.status === 401) {
    localStorage.removeItem('access_token')
    window.location.href = '/login'  // ← 整页刷新
}
```

**问题**: SPA 应用使用 `window.location.href` 会导致整个应用重新加载，丢失所有状态。应使用 `router.push('/login')`。

#### 3. 无 Token 自动刷新机制

用户在操作过程中 access_token 过期（30min），请求返回 401 后直接跳转登录页。refresh_token 已存储但从未使用。应实现请求拦截器中的静默刷新。

#### 4. Dashboard 错误处理静默失败

**文件**: `dashboard/index.vue:284-291`
```typescript
} catch {
    overview.value = null
    recentReadings.value = []
    hourlyBars.value = []
}
```

**问题**: 请求失败时只清空数据，不给用户任何提示。用户看到空页面以为系统正常但没有数据。

---

### 🟢 轻微问题

5. **TypeScript `any` 类型滥用**: `auth.ts` 中 `userInfo` 为 `any`，login 组件中 `res as any`，dashboard 中 `any[]`。

6. **未使用 `refresh_token` 过期检查**: login 组件存储了 refresh_token 但 router 守卫只检查 access_token。

7. **`isLoggedIn` 返回值不是响应式的**: 基于 `computed` 的 `token.value` 检查更可靠，当前 `isLoggedIn()` 作为方法调用没问题但在模板中用法不佳。

---

## 架构一致性检查

| 检查项 | 状态 | 说明 |
|--------|------|------|
| Rust/Java ProtocolType 枚举对齐 | ✅ | 两端都是 18 种变体，命名一致 |
| 双通道上报 (MQTT + WS fallback) | ⚠️ | WS fallback 实现有误导（HTTP POST 而非 WebSocket） |
| 配置下发链路 (Web→Java→MQTT→Rust) | ⚠️ | 下发失败不会通知前端 |
| 模块化单体分层 | ✅ | Java 7 模块分层合理 |
| Flyway 数据库迁移 | ⚠️ | 版本号分散在多个模块 |
| 禁止 Controller 暴露 Entity (S03) | ❌ | DeviceController.create 违反 |
| 禁止 `unwrap()` (S07) | ❌ | 采集端 4 处违反 |
| 测试使用 H2 非 mock | ✅ | DAO 层测试目录可见 |

---

## 优先级修复清单

### P0 — 本周必须修复

| # | 问题 | 层级 | 影响 |
|---|------|------|------|
| 1 | MQTT 重连 eventloop 泄露 | Rust | 断线后永远无法恢复 |
| 2 | CORS allowCredentials + * | Java | 安全漏洞 |
| 3 | application-dev.yml 硬编码凭据 | Java | 安全泄露 |
| 4 | Scheduler block_in_place 线程饥饿 | Rust | 采集性能问题 |

### P1 — 下个迭代修复

| # | 问题 | 层级 |
|---|------|------|
| 5 | Controller 暴露 Entity | Java |
| 6 | Token localStorage XSS 风险 | 前端 |
| 7 | WS fallback 实现不匹配命名 | Rust |
| 8 | ConfigPush 失败静默吞异常 | Java |
| 9 | Telemetry 未接入 | Rust |
| 10 | Aggregator 输出数据不完整 | Rust |

### P2 — 技术债清理

| # | 问题 |
|---|------|
| 11 | 重复 parse_broker_url |
| 12 | Dashboard 无错误提示 |
| 13 | TypeScript any 类型清理 |
| 14 | 分页无上限保护 |
| 15 | unlock mutex in async context (storage) |

---

## 总评

**整体评价**: 项目架构设计成熟——三层分离清晰、18 种协议枚举对齐、双通道上报、配置热更新都是正确的设计决策。代码可读性好，注释详尽。

**最大风险**: Rust 采集端的 MQTT 重连机制有 bug（eventloop 泄露），这是**生产阻塞项**——一旦 MQTT 断开，采集端永远无法自动恢复。

**最需加强**: 
1. Rust 采集端的 MQTT 连接生命周期管理
2. Java 端的安全配置（CORS、凭据管理）
3. 前端 Token 管理策略（localStorage → httpOnly cookie）

**代码质量评分: 7.2/10** — 良好的架构 + 工程化意识，但实现层面有数个需要立即修复的硬伤。
