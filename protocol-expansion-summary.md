# 📡 协议扩充与对齐 — 完成报告

> **日期**: 2026-06-26  
> **原则**: 协议不能 Mock，全部真实实现

---

## ✅ 完成项

### 1. 协议数量：4 → 18 种

```
原 有:  ModbusRTU, ModbusTCP, DL645, ATCommand (仅 Mock，无真实驱动)
新 增:  14 种工业协议，全部带真实帧级驱动

串口协议 (6):  Modbus RTU, DL/T645-2007, DL/T645-1997, IEC 101, CAN Bus, PROFIBUS DP
TCP/IP (9):   Modbus TCP, IEC 104, DNP3, OPC UA, BACnet/IP, S7, FINS TCP, EtherNet/IP, Mitsubishi MC
上层协议 (3):  MQTT, SNMP v2c, HTTP JSON
```

### 2. 采集端 (Rust) — 每个协议都有真实驱动

| 协议 | 驱动文件 | 实现方式 |
|------|---------|---------|
| Modbus RTU/TCP | `modbus.rs` | tokio-modbus 库 + 真实串口/TCP 读写 |
| DL/T645 | `dlt645.rs` | 自实现 645 帧协议 (帧组帧/BCD解码/CS校验) |
| IEC 104 | `iec104.rs` | APCI + ASDU 帧，U/I/S 格式，C_IC_NA_1 总召唤 |
| IEC 101 | `iec101.rs` | FT1.2 帧格式，固定/可变帧长 |
| DNP3 | `dnp3.rs` | DNP3 CRC, Class 0 轮询, Analog/Binary 解析 |
| OPC UA | `opcua.rs` | OPC UA TCP Hello + 二进制消息解析 |
| MQTT | `mqtt_driver.rs` | rumqttc 真实连接/订阅/JSON解析 |
| BACnet/IP | `bacnet.rs` | BVLC + NPDU + Who-Is + ReadProperty |
| SNMP v2c | `snmp.rs` | BER 编码 GetRequest + OID 解析 |
| HTTP JSON | `http_poll.rs` | reqwest HTTP GET + JSON 解析 |
| CAN Bus | `canbus.rs` | Linux SocketCAN (libc), 其他平台明确报错 |
| S7 Comm | `s7comm.rs` | ISO-on-TCP + S7 协议 (Read Var / DB/M/I/Q) |
| PROFIBUS DP | `profibus.rs` | DP-V0 Data_Exchange 帧 |
| EtherNet/IP | `ethernet_ip.rs` | EIP Session + CIP Forward Open + Read Tag |
| FINS TCP | `fins.rs` | FINS/TCP header + Memory Area Read |
| Mitsubishi MC | `mitsubishi.rs` | 3E 帧二进制格式 |

### 3. 服务端 (Java) — 枚举 + 数据库完全对齐

```
新增:
  neuron-common/.../enums/ProtocolType.java     ← 18 种协议枚举，code 与 Rust 严格一致
  neuron-server/.../db/migration/V1__init_schema.sql  ← Flyway 建表 + 插 18 条记录
```

两端 code 值对照：

| Rust serde rename | Java enum code | 数据库 code |
|-------------------|---------------|------------|
| `MODBUS_RTU` | `MODBUS_RTU` | `MODBUS_RTU` |
| `DL_T645_2007` | `DL_T645_2007` | `DL_T645_2007` |
| `IEC_60870_5_104` | `IEC_60870_5_104` | `IEC_60870_5_104` |
| ... (全部 18 个严格一致) | | |

### 4. 关键架构决策

- `ProtocolType` 枚举在两个代码库中保持 **编译期对齐**
- 每个枚举值都有 `is_serial()` / `is_tcp()` / `is_app_layer()` 辅助方法
- CAN Bus 和 PROFIBUS DP 根据 `#[cfg(target_os)]` 做平台适配
- 串口协议统一使用 `tokio-serial`（非阻塞）
- 所有数字解码支持 float32/64, int16/32, uint16/32, bool

---

## 📁 新增文件清单

```
Rust 采集端 (16 个驱动文件):
  collector-driver/src/modbus.rs
  collector-driver/src/dlt645.rs
  collector-driver/src/iec104.rs
  collector-driver/src/iec101.rs
  collector-driver/src/dnp3.rs
  collector-driver/src/opcua.rs
  collector-driver/src/mqtt_driver.rs
  collector-driver/src/bacnet.rs
  collector-driver/src/snmp.rs
  collector-driver/src/http_poll.rs
  collector-driver/src/canbus.rs
  collector-driver/src/s7comm.rs
  collector-driver/src/profibus.rs
  collector-driver/src/ethernet_ip.rs
  collector-driver/src/fins.rs
  collector-driver/src/mitsubishi.rs

Java 服务端:
  neuron-common/.../enums/ProtocolType.java
  neuron-server/.../db/migration/V1__init_schema.sql
```
