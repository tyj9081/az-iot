# 数据库开发规范 — XML Mapper 强制规范

> 版本: v1.0 | 生效日期: 2026-06-27 | 适用范围: 全项目强制

---

## 一、核心规则

### 1.1 所有 SQL 操作必须写入 XML Mapper

| ✅ 正确做法 | ❌ 禁止做法 |
|------------|------------|
| Mapper 接口定义方法 + XML 文件编写 SQL | Service 层直接 `new LambdaQueryWrapper<>()` |
| `@Mapper` + `XXXMapper.xml` | Controller 层直接操作 Mapper |
| 复杂查询用 `<select>` `<where>` `<if>` 标签 | Java 代码拼接 SQL 字符串 |
| 批量操作用 `<foreach>` | for 循环逐条调用 Mapper |

### 1.2 XML 文件对应规则

- **每个 Mapper 接口必须有同名 XML 文件**
- 路径: `src/main/resources/mapper/` + 与 Java 包结构一致
- 命名: `{EntityName}Mapper.xml`，与 Mapper 接口一一对应

```
neuron-dao/
├── src/main/java/com/aziot/dao/mapper/
│   ├── system/SysUserMapper.java
│   ├── system/SysRoleMapper.java
│   ├── device/DevDeviceMapper.java
│   └── ...
└── src/main/resources/mapper/
    ├── system/SysUserMapper.xml        ← 必须存在
    ├── system/SysRoleMapper.xml
    ├── device/DevDeviceMapper.xml
    └── ...
```

### 1.3 配置要求

`application.yml` 中必须声明 XML 路径:

```yaml
mybatis-plus:
  mapper-locations: classpath*:/mapper/**/*.xml
  configuration:
    map-underscore-to-camel-case: true
    log-impl: org.apache.ibatis.logging.stdout.StdOutImpl
```

---

## 二、Service 层调用规范

### 2.1 正确模式: Service → Mapper → XML

```java
// Mapper 接口
@Mapper
public interface SysUserMapper extends BaseMapper<SysUser> {
    // 自定义查询方法声明
    List<SysUser> selectByRoleCode(@Param("roleCode") String roleCode);
    SysUser selectByUsername(@Param("username") String username);
}
```

```xml
<!-- SysUserMapper.xml -->
<mapper namespace="com.aziot.dao.mapper.system.SysUserMapper">

    <select id="selectByRoleCode" resultType="com.aziot.dao.entity.system.SysUser">
        SELECT u.* FROM sys_user u
        INNER JOIN sys_user_role ur ON u.id = ur.user_id
        INNER JOIN sys_role r ON ur.role_id = r.id
        WHERE r.role_code = #{roleCode}
          AND u.is_deleted = 0
    </select>

    <select id="selectByUsername" resultType="com.aziot.dao.entity.system.SysUser">
        SELECT * FROM sys_user
        WHERE username = #{username}
          AND is_deleted = 0
    </select>

</mapper>
```

```java
// Service 层 — 只调用 Mapper 方法，不写 SQL
@Service
public class SysUserService extends ServiceImpl<SysUserMapper, SysUser> {
    public SysUser getByUsername(String username) {
        return baseMapper.selectByUsername(username);
    }
}
```

### 2.2 分页查询模式

```xml
<!-- 分页查询: MyBatis-Plus 自动处理分页拦截 -->
<select id="selectPageByCondition" resultType="SysUser">
    SELECT * FROM sys_user
    <where>
        is_deleted = 0
        <if test="username != null and username != ''">
            AND username LIKE CONCAT('%', #{username}, '%')
        </if>
        <if test="status != null">
            AND status = #{status}
        </if>
    </where>
    ORDER BY created_at DESC
</select>
```

---

## 三、XML 编写规范

### 3.1 标签使用

| 标签 | 用途 | 示例 |
|------|------|------|
| `<select>` | 查询 | `SELECT * FROM table WHERE id = #{id}` |
| `<insert>` | 插入 | `INSERT INTO table (...) VALUES (...)` |
| `<update>` | 更新 | `UPDATE table SET ... WHERE id = #{id}` |
| `<delete>` | 删除 | `DELETE FROM table WHERE id = #{id}` |
| `<sql>` | SQL片段复用 | `<sql id="Base_Column"> id, name, ... </sql>` |
| `<include>` | 引用片段 | `<include refid="Base_Column"/>` |
| `<if>` | 条件判断 | `<if test="name != null">AND name = #{name}</if>` |
| `<where>` | 动态WHERE | 自动去除首个AND/OR |
| `<foreach>` | 批量操作 | `collection="list" item="item" separator=","` |
| `<set>` | 动态SET | 自动去除尾部逗号 |
| `<resultMap>` | 复杂映射 | JOIN结果映射到嵌套对象 |

### 3.2 命名规范

| 操作类型 | 方法命名前缀 | XML id | 示例 |
|---------|-------------|--------|------|
| 单条查询 | `select`, `get` | 同方法名 | `selectById`, `selectByUsername` |
| 列表查询 | `selectList`, `list` | 同方法名 | `selectListByStatus` |
| 分页查询 | `selectPage` | 同方法名 | `selectPageByCondition` |
| 插入 | `insert` | 同方法名 | `insertBatch` |
| 更新 | `update` | 同方法名 | `updateStatus` |
| 删除 | `delete` | 同方法名 | `deleteByDeviceId` |
| 统计 | `count` | 同方法名 | `countByDeviceId` |

### 3.3 参数传递

```java
// 单个参数: 直接传
SysUser selectById(Long id);

// 多个参数: 用 @Param 明确命名
List<SysUser> selectByCondition(
    @Param("username") String username,
    @Param("status") Integer status
);

// 对象参数: #{字段名} 访问
int insert(SysUser user);  // XML: #{username}, #{nickname}
```

---

## 四、禁止清单

### 4.1 绝对禁止

| # | 禁止行为 | 当前违规数 |
|---|---------|-----------|
| ❌ 1 | Service 层使用 `LambdaQueryWrapper` | **14 个文件** |
| ❌ 2 | Service 层使用 `QueryWrapper` | **0 个文件** |
| ❌ 3 | Controller 层直接操作 Mapper | **1 个文件** (MqttAuthController) |
| ❌ 4 | Java 代码中拼接 SQL 字符串 | 待排查 |
| ❌ 5 | `@Select` / `@Insert` 等注解写 SQL | 当前 0 |
| ❌ 6 | `String.format` / `+` 拼接 SQL | 待排查 |

### 4.2 例外情况

以下 MyBatis-Plus `BaseMapper` 自带方法无需在 XML 中声明:

```java
// 这些可以继续使用，不需要 XML
baseMapper.selectById(id);
baseMapper.insert(entity);
baseMapper.updateById(entity);
baseMapper.deleteById(id);
baseMapper.selectList(null);       // 无条件查全表（谨慎使用）
baseMapper.selectPage(page, null); // 无条件分页（谨慎使用）
```

> ⚠️ BaseMapper 自带方法仅适用于简单 CRUD。**任何带 WHERE 条件、JOIN、聚合、排序的查询都必须走 XML**。

---

## 五、迁移指南

### 5.1 LambdaQueryWrapper → XML 对照

```java
// ❌ 改造前 (Service 层)
LambdaQueryWrapper<DevDevice> qw = new LambdaQueryWrapper<>();
qw.eq(DevDevice::getSerialPortId, portId)
  .eq(DevDevice::getIsDeleted, 0)
  .orderByDesc(DevDevice::getCreatedAt);
List<DevDevice> list = this.list(qw);
```

```xml
<!-- ✅ 改造后 (XML) -->
<select id="selectByPortId" resultType="DevDevice">
    SELECT * FROM dev_device
    WHERE serial_port_id = #{portId}
      AND is_deleted = 0
    ORDER BY created_at DESC
</select>
```

```java
// ✅ 改造后 (Service 层)
List<DevDevice> list = baseMapper.selectByPortId(portId);
```

### 5.2 复杂 JOIN 改造示例

```java
// ❌ 改造前 (Service 层多步查询)
LambdaQueryWrapper<DevDevice> qw1 = new LambdaQueryWrapper<>();
qw1.eq(DevDevice::getSerialPortId, portId);
List<DevDevice> devices = devDeviceMapper.selectList(qw1);
// 然后 for 循环逐个查 model... (N+1 问题!)
```

```xml
<!-- ✅ 改造后 (XML — 一条 SQL 解决) -->
<select id="selectWithModelByPortId" resultMap="DeviceWithModelMap">
    SELECT d.*, m.name as model_name, m.code as model_code
    FROM dev_device d
    LEFT JOIN dev_device_model m ON d.model_id = m.id
    WHERE d.serial_port_id = #{portId}
      AND d.is_deleted = 0
</select>
```

---

## 六、审查机制

### 6.1 Code Review 检查点

1. PR 中有新增 Service → 必须有对应 Mapper XML
2. 出现 `LambdaQueryWrapper` / `QueryWrapper` → 直接打回
3. 出现 SQL 字符串拼接 → 直接打回
4. Controller 中有 Mapper 注入 → 直接打回
5. Mapper XML 缺少 `parameterType` / `resultType` → 警告

### 6.2 自动化检查

```bash
# 检查 Service 层 LambdaQueryWrapper 违规
grep -rn "LambdaQueryWrapper" neuron-service/src/main/java/ | wc -l

# 检查 XML 文件是否与 Mapper 接口一一对应
# Mapper 数量 vs XML 数量应相等
```

---

## 附录: 当前违规清单 (2026-06-27)

### A. 缺失 XML 文件的 Mapper (15 个全缺)

| # | Mapper | 应有 XML 路径 |
|---|--------|-------------|
| 1 | SysUserMapper | `mapper/system/SysUserMapper.xml` |
| 2 | SysRoleMapper | `mapper/system/SysRoleMapper.xml` |
| 3 | SysPermissionMapper | `mapper/system/SysPermissionMapper.xml` |
| 4 | SysAuditLogMapper | `mapper/system/SysAuditLogMapper.xml` |
| 5 | SysConfigMapper | `mapper/system/SysConfigMapper.xml` |
| 6 | DevProtocolMapper | `mapper/device/DevProtocolMapper.xml` |
| 7 | DevManufacturerMapper | `mapper/device/DevManufacturerMapper.xml` |
| 8 | DevDeviceModelMapper | `mapper/device/DevDeviceModelMapper.xml` |
| 9 | DevRegisterMapMapper | `mapper/device/DevRegisterMapMapper.xml` |
| 10 | DevCollectorMapper | `mapper/collector/DevCollectorMapper.xml` |
| 11 | DevSerialPortMapper | `mapper/collector/DevSerialPortMapper.xml` |
| 12 | DevDeviceMapper | `mapper/device/DevDeviceMapper.xml` |
| 13 | DevDeviceReadingMapper | `mapper/device/DevDeviceReadingMapper.xml` |
| 14 | DevDeviceAlarmConfigMapper | `mapper/device/DevDeviceAlarmConfigMapper.xml` |
| 15 | DevDeviceInstructionMapper | `mapper/device/DevDeviceInstructionMapper.xml` |

### B. Service 层 QueryWrapper 违规 (14 个文件, 50+ 处)

| # | 文件 | LambdaQueryWrapper 次数 |
|---|------|------------------------|
| 1 | DevDeviceReadingService.java | 7 |
| 2 | DevDeviceService.java | 5 |
| 3 | DevDeviceModelService.java | 5 |
| 4 | DevRegisterMapService.java | 4 |
| 5 | DevCollectorService.java | 4 |
| 6 | DevDeviceAlarmConfigService.java | 3 |
| 7 | DevDeviceInstructionService.java | 3 |
| 8 | ConfigPushService.java | 2 |
| 9 | DevManufacturerService.java | 2 |
| 10 | DevProtocolService.java | 1 |
| 11 | SysUserService.java | 1 |
| 12 | SysRoleService.java | 1 |
| 13 | SysPermissionService.java | 1 |
| 14 | AuthService.java | 1 |
| 15 | DevSerialPortService.java | 1 |
| 16 | MqttSubscriberService.java | 1 |

### C. Controller 层越权 (1 处)

| # | 文件 | 行号 | 问题 |
|---|------|------|------|
| 1 | MqttAuthController.java | - | 直接注入 Mapper 并调用 LambdaQueryWrapper |
