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

use anyhow::Result;
use collector_model::{Device, ProtocolType};
use std::collections::HashMap;
use tracing::warn;

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
