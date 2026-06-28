//! Protocol driver system for AZ-IOT collector.
//!
//! Each protocol variant in [`ProtocolType`] maps to a real driver implementation
//! that communicates with physical hardware over serial, TCP, or application-layer
//! protocols (MQTT, SNMP, HTTP).
//!
//! # Architecture
//!
//! ```text
//! ProtocolType  →  DriverFactory  →  Box<dyn ProtocolDriver>
//!                                            ↓
//!                                   fn collect(&Device) → HashMap<sensor_code, value>
//! ```

pub mod modbus;
pub mod dlt645;
pub mod iec104;
pub mod iec101;
pub mod dnp3;
pub mod opcua;
pub mod mqtt_driver;
pub mod bacnet;
pub mod snmp;
pub mod http_poll;
pub mod canbus;
pub mod s7comm;
pub mod profibus;
pub mod ethernet_ip;
pub mod fins;
pub mod mitsubishi;

use anyhow::{Context, Result};
use collector_model::{BusParam, BusType, Device, ProtocolType};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;
use tokio_serial::SerialPortBuilderExt;
use tracing::warn;

/// 驱动层专用 I/O runtime，完全独立于主调度器的 tokio runtime。
///
/// 为什么需要独立 runtime：
///   - 驱动 `collect()` 被 spawn_blocking 调用，运行在阻塞线程上
///   - `Handle::current().block_on()` 在部分 tokio 版本/平台会 panic：
///     "Cannot start a runtime from within a runtime"
///   - 使用独立 runtime 彻底避免嵌套 runtime 问题
///
/// 所有需要在同步 context 中执行异步 I/O 的驱动都通过这个 runtime 做桥接。
pub(crate) fn driver_runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("driver-io")
            .enable_all()
            .build()
            .expect("Failed to create driver I/O runtime")
    })
}

/// 串口可用性预检 — 在采集循环开始前快速验证端口是否存在。
///
/// 流程：打开串口 → 立即关闭。不进行任何读写操作。
/// 超时 500ms，避免阻塞调度器主循环过久。
/// 失败原因会被 anyhow context 包装后返回。
pub fn probe_serial(port_name: &str, bus_param: &BusParam) -> Result<()> {
    let port_name = port_name.to_string();
    let bus_param = bus_param.clone();
    let rt = driver_runtime();
    rt.block_on(async move {
        let builder = tokio_serial::new(&port_name, bus_param.baud)
            .data_bits(match bus_param.data_bits {
                5 => tokio_serial::DataBits::Five,
                6 => tokio_serial::DataBits::Six,
                7 => tokio_serial::DataBits::Seven,
                _ => tokio_serial::DataBits::Eight,
            })
            .parity(match bus_param.parity.as_str() {
                "odd" => tokio_serial::Parity::Odd,
                "even" => tokio_serial::Parity::Even,
                _ => tokio_serial::Parity::None,
            })
            .stop_bits(match bus_param.stop_bits {
                2 => tokio_serial::StopBits::Two,
                _ => tokio_serial::StopBits::One,
            })
            .timeout(Duration::from_millis(500));
        let _port = builder
            .open_native_async()
            .with_context(|| format!("Probe serial port {port_name}"))?;
        // 端口打开成功，立即 drop 关闭
        Ok::<_, anyhow::Error>(())
    })
}

// ─── Trait ────────────────────────────────────────────

/// Common interface for all protocol drivers.
///
/// Every driver MUST implement real I/O — no mock values permitted in production.
pub trait ProtocolDriver: Send + Sync {
    /// Human-readable protocol name for logging.
    fn protocol_name(&self) -> &str;

    /// Execute one collection cycle against the given device.
    ///
    /// Returns a map of `sensor_code → reading_value`.
    /// Must apply `coefficient` and `offset` from each DataPoint before returning.
    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>>;
}

// ─── Factory ──────────────────────────────────────────

pub struct DriverFactory;

impl DriverFactory {
    /// Create the appropriate driver for a device's protocol.
    ///
    /// Returns `None` only for protocols that are known but not yet implemented
    /// on this platform (e.g. CAN bus on non-Linux systems).
    pub fn create(device: &Device) -> Option<Box<dyn ProtocolDriver>> {
        match &device.protocol {
            ProtocolType::ModbusRTU => Some(Box::new(modbus::ModbusRTUDriver::new())),
            ProtocolType::ModbusTCP => Some(Box::new(modbus::ModbusTCPDriver::new())),
            ProtocolType::DL645_2007 => Some(Box::new(dlt645::DLT645Driver::new(false))),
            ProtocolType::DL645_1997 => Some(Box::new(dlt645::DLT645Driver::new(true))),
            ProtocolType::IEC104 => Some(Box::new(iec104::IEC104Driver::new())),
            ProtocolType::IEC101 => Some(Box::new(iec101::IEC101Driver::new())),
            ProtocolType::DNP3 => Some(Box::new(dnp3::DNP3Driver::new())),
            ProtocolType::OpcUa => Some(Box::new(opcua::OpcUaDriver::new())),
            ProtocolType::Mqtt => Some(Box::new(mqtt_driver::MqttDriver::new())),
            ProtocolType::BacnetIP => Some(Box::new(bacnet::BacnetDriver::new())),
            ProtocolType::SnmpV2c => Some(Box::new(snmp::SnmpV2cDriver::new())),
            ProtocolType::HttpJson => Some(Box::new(http_poll::HttpJsonDriver::new())),
            ProtocolType::CanBus => {
                #[cfg(target_os = "linux")]
                { Some(Box::new(canbus::CanBusDriver::new())) }
                #[cfg(not(target_os = "linux"))]
                {
                    warn!("CAN bus driver requires Linux with SocketCAN — not available on this platform");
                    None
                }
            }
            ProtocolType::S7Comm => Some(Box::new(s7comm::S7CommDriver::new())),
            ProtocolType::ProfibusDP => {
                #[cfg(target_os = "linux")]
                { Some(Box::new(profibus::ProfibusDPDriver::new())) }
                #[cfg(not(target_os = "linux"))]
                {
                    warn!("PROFIBUS DP driver requires Linux with serial port — not available on this platform");
                    None
                }
            }
            ProtocolType::EthernetIP => Some(Box::new(ethernet_ip::EthernetIPDriver::new())),
            ProtocolType::FinsTcp => Some(Box::new(fins::FinsTcpDriver::new())),
            ProtocolType::MitsubishiMC => Some(Box::new(mitsubishi::MitsubishiMCDriver::new())),
        }
    }
}

// ─── Shared utilities ─────────────────────────────────

/// Apply `coefficient` and `offset` transformation to a raw reading.
#[inline]
pub fn apply_transform(raw: f64, coefficient: f64, offset: f64) -> f64 {
    raw * coefficient + offset
}
