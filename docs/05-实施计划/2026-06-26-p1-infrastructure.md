# P1: 基础设施 + 系统骨架 实施计划

> **For implementer:** Use TDD throughout. Write failing test first. Watch it fail. Then implement.

**Goal:** 搭建 AZ-IOT 三端工程骨架 + Flyway DDL + JWT 认证 + RBAC + 审计

**Architecture:** Java Maven 多模块 (neuron-platform/) + Rust Cargo Workspace (neuron-collector/) + Vue3 Vite (neuron-web/)

**Tech Stack:** Java 17 + Spring Boot 3.2 + MyBatis-Plus 3.5 + Flyway; Rust 1.80 + tokio + rumqttc; Vue 3.4 + Vite 5 + Element Plus + Pinia

---

## Task 1: Java Maven 多模块骨架

**Files:**
- Create: `neuron-platform/pom.xml` (parent)
- Create: `neuron-platform/neuron-common/pom.xml`
- Create: `neuron-platform/neuron-dao/pom.xml`
- Create: `neuron-platform/neuron-service/pom.xml`
- Create: `neuron-platform/neuron-controller/pom.xml`
- Create: `neuron-platform/neuron-security/pom.xml`
- Create: `neuron-platform/neuron-mqtt/pom.xml`
- Create: `neuron-platform/neuron-server/pom.xml`

**Step 1: 创建 parent pom.xml**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 https://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.aziot</groupId>
    <artifactId>neuron-platform</artifactId>
    <version>1.0.0-SNAPSHOT</version>
    <packaging>pom</packaging>
    <name>AZ-IOT Platform</name>

    <parent>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-parent</artifactId>
        <version>3.2.0</version>
        <relativePath/>
    </parent>

    <properties>
        <java.version>17</java.version>
        <mybatis-plus.version>3.5.5</mybatis-plus.version>
        <jwt.version>0.12.3</jwt.version>
    </properties>

    <modules>
        <module>neuron-common</module>
        <module>neuron-dao</module>
        <module>neuron-service</module>
        <module>neuron-controller</module>
        <module>neuron-security</module>
        <module>neuron-mqtt</module>
        <module>neuron-server</module>
    </modules>
</project>
```

**Step 2: 各子模块 pom.xml**
- neuron-common: 无特殊依赖
- neuron-dao: spring-boot-starter + mybatis-plus-boot-starter + mysql-connector + flyway-core + h2 (test)
- neuron-service: spring-boot-starter + neuron-dao
- neuron-controller: spring-boot-starter-web + spring-boot-starter-validation + neuron-service
- neuron-security: spring-boot-starter-security + jjwt-api/impl/jackson + neuron-dao
- neuron-mqtt: spring-boot-starter + mqtt-client (org.eclipse.paho.client.mqttv3)
- neuron-server: neuron-controller + neuron-security + neuron-mqtt (启动模块)

**Step 3: 创建 Spring Boot 主类**
```java
// neuron-server/src/main/java/com/aziot/server/NeuronServerApplication.java
package com.aziot.server;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.annotation.ComponentScan;

@SpringBootApplication
@ComponentScan("com.aziot")
public class NeuronServerApplication {
    public static void main(String[] args) {
        SpringApplication.run(NeuronServerApplication.class, args);
    }
}
```

**Step 4: application.yml**
```yaml
# neuron-server/src/main/resources/application.yml
spring:
  application:
    name: neuron-server
  datasource:
    url: jdbc:mysql://localhost:3306/az_iot?useUnicode=true&characterEncoding=utf8mb4&useSSL=false&serverTimezone=Asia/Shanghai&allowPublicKeyRetrieval=true
    username: root
    password: ${DB_PASSWORD:root}
    driver-class-name: com.mysql.cj.jdbc.Driver
  flyway:
    enabled: true
    locations: classpath:db/migration
    baseline-on-migrate: true

mybatis-plus:
  configuration:
    map-underscore-to-camel-case: true
    log-impl: org.apache.ibatis.logging.stdout.StdOutImpl

jwt:
  secret: ${JWT_SECRET:az-iot-jwt-secret-key-must-be-at-least-256-bits-long}
  access-token-expiration: 1800000
  refresh-token-expiration: 604800000

server:
  port: 8080
```

**Step 5: 验证 — `mvn compile`**
Command: `cd neuron-platform && mvn compile -q`
Expected: BUILD SUCCESS

---

## Task 2: Rust Cargo Workspace 骨架

**Files:**
- Create: `neuron-collector/Cargo.toml` (workspace)
- Create: `neuron-collector/collector-model/Cargo.toml`
- Create: `neuron-collector/collector-driver/Cargo.toml`
- Create: `neuron-collector/collector-scheduler/Cargo.toml`
- Create: `neuron-collector/collector-config-sync/Cargo.toml`
- Create: `neuron-collector/collector-reporter/Cargo.toml`
- Create: `neuron-collector/collector-storage/Cargo.toml`
- Create: `neuron-collector/collector-telemetry/Cargo.toml`
- Create: `neuron-collector/collector-bin/Cargo.toml`
- Create: `neuron-collector/collector-model/src/lib.rs`

**Step 1: Cargo.toml (workspace)**
```toml
[workspace]
resolver = "2"
members = [
    "collector-model",
    "collector-driver",
    "collector-scheduler",
    "collector-config-sync",
    "collector-reporter",
    "collector-storage",
    "collector-telemetry",
    "collector-bin",
]

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
rumqttc = "0.24"
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Step 2: collector-model 核心数据模型**
```toml
# collector-model/Cargo.toml
[package]
name = "collector-model"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
```

```rust
// collector-model/src/lib.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProtocolType {
    #[serde(rename = "MODBUS_RTU")]
    ModbusRTU,
    #[serde(rename = "MODBUS_TCP")]
    ModbusTCP,
    #[serde(rename = "DL_T645")]
    DL645,
    #[serde(rename = "AT_COMMAND")]
    ATCommand,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BusType {
    #[serde(rename = "serial")]
    Serial {
        port_name: String,
        bus_param: BusParam,
    },
    #[serde(rename = "tcp")]
    Tcp {
        host: String,
        port: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusParam {
    pub baud: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub sensor_code: String,
    pub sensor_name: String,
    pub register_address: u16,
    pub register_count: u16,
    pub data_type: String,
    pub byte_order: String,
    pub func_code: String,
    pub coefficient: f64,
    pub offset: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub protocol: ProtocolType,
    pub slave_addr: u8,
    pub bus: BusType,
    pub collect_interval_sec: Option<u64>,
    pub data_points: Vec<DataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDelta {
    pub version: u64,
    pub action: String,  // "add" | "update" | "remove"
    pub device: Option<Device>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_config_delta() {
        let json = r#"{
            "version": 1,
            "action": "add",
            "device": {
                "id": 123,
                "code": "DEV-001",
                "name": "test",
                "protocol": "MODBUS_RTU",
                "slave_addr": 1,
                "bus": {
                    "serial": {
                        "port_name": "COM5",
                        "bus_param": {"baud": 9600, "data_bits": 8, "stop_bits": 1, "parity": "none"}
                    }
                },
                "collect_interval_sec": null,
                "data_points": []
            }
        }"#;
        let delta: ConfigDelta = serde_json::from_str(json).unwrap();
        assert_eq!(delta.version, 1);
        assert_eq!(delta.action, "add");
    }
}
```

**Step 3: 其他 crate 最小骨架**
每个 crate 的 Cargo.toml 只包含 `[workspace]` 依赖，lib.rs 只有一个空 pub fn 占位。

**Step 4: collector-bin main.rs**
```rust
// collector-bin/src/main.rs
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("AZ-IOT Collector starting...");
    Ok(())
}
```

**Step 5: 验证 — `cargo build`**
Command: `cd neuron-collector && cargo build 2>&1`
Expected: Compiling... Finished

---

## Task 3: Vue 3 + Vite 骨架

**Step 1: 使用 npm create vite**
Command: `cd /d/物联网/az-iot/neuron-web && npm create vite@latest . -- --template vue-ts`
**注意**: 需要先确保目录存在。

**Step 2: 安装依赖**
```bash
cd /d/物联网/az-iot/neuron-web
npm install element-plus pinia vue-router@4 axios @element-plus/icons-vue
npm install -D @types/node sass
```

**Step 3: vite.config.ts**
```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: { '@': resolve(__dirname, 'src') }
  },
  server: {
    port: 3000,
    proxy: {
      '/api': { target: 'http://localhost:8080', changeOrigin: true }
    }
  }
})
```

**Step 4: main.ts**
```typescript
import { createApp } from 'vue'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import zhCn from 'element-plus/dist/locale/zh-cn.mjs'
import App from './App.vue'
import router from './router'
import { createPinia } from 'pinia'

const app = createApp(App)
app.use(ElementPlus, { locale: zhCn })
app.use(router)
app.use(createPinia())
app.mount('#app')
```

**Step 5: 基础路由**
```typescript
// src/router/index.ts
import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/login/index.vue'),
    meta: { title: '登录' }
  },
  {
    path: '/',
    component: () => import('@/views/layout/index.vue'),
    redirect: '/dashboard',
    children: [
      {
        path: 'dashboard',
        name: 'Dashboard',
        component: () => import('@/views/dashboard/index.vue'),
        meta: { title: '工作台' }
      }
    ]
  }
]

const router = createRouter({ history: createWebHistory(), routes })
export default router
```

**Step 6: 创建目录结构**
```
neuron-web/src/
├── api/           # auth.ts
├── views/
│   ├── login/index.vue
│   ├── layout/index.vue
│   └── dashboard/index.vue
├── router/index.ts
├── stores/auth.ts
├── types/api.d.ts
└── utils/request.ts
```

**验证:** `pnpm build` 无报错

---

## Task 4: Flyway DDL — 全部核心表

**Files:**
- Create: `neuron-platform/neuron-dao/src/main/resources/db/migration/V1__init_sys_tables.sql`
- Create: `neuron-platform/neuron-dao/src/main/resources/db/migration/V2__init_dev_protocol.sql`
- Create: `neuron-platform/neuron-dao/src/main/resources/db/migration/V3__init_dev_manufacturer.sql`
- Create: `neuron-platform/neuron-dao/src/main/resources/db/migration/V4__init_dev_device_model.sql`
- Create: `neuron-platform/neuron-dao/src/main/resources/db/migration/V5__init_dev_register_map.sql`
- Create: `neuron-platform/neuron-dao/src/main/resources/db/migration/V6__init_dev_collector.sql`
- Create: `neuron-platform/neuron-dao/src/main/resources/db/migration/V7__init_dev_serial_port.sql`
- Create: `neuron-platform/neuron-dao/src/main/resources/db/migration/V8__init_dev_device.sql`
- Create: `neuron-platform/neuron-dao/src/main/resources/db/migration/V9__init_dev_device_reading.sql`
- Create: `neuron-platform/neuron-dao/src/main/resources/db/migration/V10__init_sys_config.sql`

详见 `docs/开发总纲.md` 第三章数据模型定义。

**注意**: `dev_device_reading` 使用 `PARTITION BY RANGE (TO_DAYS(read_at))`

---

## Task 5: Java Entity 层

创建所有 MyBatis-Plus Entity 类，包路径 `com.aziot.dao.entity.{domain}`

按域分包:
- `entity.device/`: DevProtocol, DevManufacturer, DevDeviceModel, DevRegisterMap
- `entity.collector/`: DevCollector, DevSerialPort
- `entity.device/`: DevDevice, DevDeviceReading
- `entity.system/`: SysUser, SysRole, SysPermission, SysAuditLog, SysConfig

**每个 Entity 必须包含**: `@TableName`, `@TableId(type=IdType.AUTO)`, `createdAt`/`updatedAt` (标注 `@TableField(fill = ...)`)

---

## Task 6: 统一 ApiResponse + GlobalExceptionHandler

**Files:**
- Create: `neuron-common/src/main/java/com/aziot/common/dto/ApiResponse.java`
- Create: `neuron-controller/src/main/java/com/aziot/controller/handler/GlobalExceptionHandler.java`
- Create: `neuron-common/src/main/java/com/aziot/common/exception/BusinessException.java`

```java
// ApiResponse.java
@Data
@NoArgsConstructor
@AllArgsConstructor
public class ApiResponse<T> {
    private int code;
    private String message;
    private T data;
    private long timestamp;

    public static <T> ApiResponse<T> ok(T data) {
        return new ApiResponse<>(0, "success", data, System.currentTimeMillis());
    }

    public static <T> ApiResponse<T> fail(int code, String message) {
        return new ApiResponse<>(code, message, null, System.currentTimeMillis());
    }
}
```

---

## Task 7: JWT 认证模块

**Files:**
- Create: `neuron-security/src/main/java/com/aziot/security/JwtTokenProvider.java`
- Create: `neuron-security/src/main/java/com/aziot/security/JwtAuthenticationFilter.java`
- Create: `neuron-security/src/main/java/com/aziot/security/SecurityConfig.java`
- Create: `neuron-security/src/main/java/com/aziot/security/UserDetailsServiceImpl.java`
- Create: `neuron-controller/src/main/java/com/aziot/controller/AuthController.java`

**Step 1: JwtTokenProvider** — 生成/验证 access token (30min) + refresh token (7d)
**Step 2: JwtAuthenticationFilter** — 从 Authorization header 提取 Bearer token，验证并设置 SecurityContext
**Step 3: SecurityConfig** — 配置 SecurityFilterChain，放行 `/api/v1/auth/**`，其余全部认证
**Step 4: AuthController**
```java
POST /api/v1/auth/login     → {username, password} → {accessToken, refreshToken}
POST /api/v1/auth/refresh   → {refreshToken} → {accessToken, refreshToken}
POST /api/v1/auth/logout    → 清除 refresh token
GET  /api/v1/auth/me        → 当前用户信息 + 权限
```

**Step 5: 密码使用 BCrypt 加密**，存储在 sys_user.password_hash

**初始用户**: admin / admin123，通过 Flyway V1 migration 脚本预插入

---

## Task 8: RBAC 模块

**Files:**
- Create: `neuron-controller/src/main/java/com/aziot/controller/system/UserController.java`
- Create: `neuron-controller/src/main/java/com/aziot/controller/system/RoleController.java`
- Create: `neuron-controller/src/main/java/com/aziot/controller/system/PermissionController.java`
- Create: 对应 Service + Mapper

**API:**
```
Users:
  GET    /api/v1/users                    # 分页
  GET    /api/v1/users/{id}
  POST   /api/v1/users
  PUT    /api/v1/users/{id}
  PUT    /api/v1/users/{id}/password
  PUT    /api/v1/users/{id}/status

Roles:
  GET    /api/v1/roles
  GET    /api/v1/roles/{id}               # 含权限列表
  POST   /api/v1/roles
  PUT    /api/v1/roles/{id}
  DELETE /api/v1/roles/{id}
  PUT    /api/v1/roles/{id}/permissions   # {permissionIds:[1,2,3]}

Permissions:
  GET    /api/v1/permissions              # 权限树
  GET    /api/v1/permissions/menu         # 当前用户菜单树
```

---

## Task 9: 审计日志切面

**Files:**
- Create: `neuron-security/src/main/java/com/aziot/security/audit/AuditLog.java` (注解)
- Create: `neuron-security/src/main/java/com/aziot/security/audit/AuditLogAspect.java` (切面)

使用 `@AuditLog(module = "设备管理", action = "创建设备")` 注解在 Controller 方法上，AOP 切面自动记录操作人、请求IP、耗时、参数和结果。

---

## Task 10: 前端登录页 + Layout + 请求封装

**Files:**
- Create: `neuron-web/src/views/login/index.vue`
- Create: `neuron-web/src/views/layout/index.vue`
- Create: `neuron-web/src/utils/request.ts`
- Create: `neuron-web/src/stores/auth.ts`
- Create: `neuron-web/src/api/auth.ts`

**登录页**: 账号密码表单 + 登录按钮 + 错误提示
**Layout**: 左侧菜单 + 顶部栏(用户名+退出) + 右侧内容区 (RouterView)
**request.ts**: Axios 拦截器 — 请求注入 Bearer token，响应拦截 401 跳转登录页
**auth store**: Pinia store 管理 token 和用户信息

---

## 执行顺序

1. Task 1 (Java 骨架) 
2. Task 4 (Flyway DDL) — 依赖 Task 1
3. Task 5 (Entity) — 依赖 Task 4
4. Task 6 (ApiResponse) — 依赖 Task 1
5. Task 7 (JWT) — 依赖 Task 5+6
6. Task 8 (RBAC) — 依赖 Task 5+7
7. Task 2 (Rust) — 独立
8. Task 3 (Vue) — 独立
9. Task 10 (前端页面) — 依赖 Task 7+8
10. Task 9 (审计) — 依赖 Task 5+7
