//! Modbus RTU / TCP drivers backed by tokio_modbus.
//! Real I/O over serial port (RTU) or TCP socket (TCP).

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio_modbus::prelude::*;

use super::ProtocolDriver;

/// Shared Modbus collection logic — runs inside tokio runtime context.
async fn mb_collect_async(
    ctx: &mut tokio_modbus::client::Context,
    device: &Device,
) -> Result<HashMap<String, f64>> {
    let mut values = HashMap::new();

    for pt in &device.data_points {
        let fc: u8 = pt.func_code.parse().unwrap_or(3);
        let addr = pt.register_address;
        let count = pt.register_count.max(1);

        let raw: Vec<u16> = match fc {
            1 | 2 => {
                let coils = ctx.read_coils(addr, count).await
                    .with_context(|| format!("Read coils at {addr}"))?
                    .with_context(|| format!("Modbus exception at {addr}")).unwrap_or_default();
                coils.iter().map(|&b| if b { 1u16 } else { 0u16 }).collect()
            }
            3 | 4 => {
                ctx.read_holding_registers(addr, count).await
                    .with_context(|| format!("Read registers at {addr}"))?
                    .with_context(|| format!("Modbus exception at {addr}")).unwrap_or_default()
            }
            _ => {
                ctx.read_holding_registers(addr, count).await
                    .with_context(|| format!("Read registers at {addr}"))?
                    .with_context(|| format!("Modbus exception at {addr}")).unwrap_or_default()
            }
        };

        let raw_f64 = decode_modbus_value(&raw, &pt.byte_order, &pt.data_type);
        let value = super::apply_transform(raw_f64, pt.coefficient, pt.offset);
        values.insert(pt.sensor_code.clone(), value);
    }
    Ok(values)
}

/// Decode raw Modbus register words into target data type.
fn decode_modbus_value(words: &[u16], byte_order: &str, dtype: &str) -> f64 {
    let mut bytes = Vec::with_capacity(words.len() * 2);
    for w in words {
        match byte_order {
            "big_endian" | "ABCD" => bytes.extend_from_slice(&w.to_be_bytes()),
            "little_endian" | "DCBA" => bytes.extend_from_slice(&w.to_le_bytes()),
            "mid_little" | "BADC" => {
                let [hi, lo] = w.to_be_bytes();
                bytes.extend_from_slice(&[lo, hi]);
            }
            "mid_big" | "CDAB" => {
                let [hi, lo] = w.to_le_bytes();
                bytes.extend_from_slice(&[lo, hi]);
            }
            _ => bytes.extend_from_slice(&w.to_be_bytes()),
        }
    }

    match dtype {
        "float32" => {
            if bytes.len() >= 4 {
                f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64
            } else { 0.0 }
        }
        "float64" => {
            if bytes.len() >= 8 {
                f64::from_be_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ])
            } else { 0.0 }
        }
        "int16" => {
            if bytes.len() >= 2 {
                i16::from_be_bytes([bytes[0], bytes[1]]) as f64
            } else { words.first().copied().unwrap_or(0) as i16 as f64 }
        }
        "uint16" => words.first().copied().unwrap_or(0) as f64,
        "int32" => {
            if bytes.len() >= 4 {
                i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64
            } else { 0.0 }
        }
        "uint32" => {
            if bytes.len() >= 4 {
                u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64
            } else { 0.0 }
        }
        "bool" => {
            if words.first().copied().unwrap_or(0) != 0 { 1.0 } else { 0.0 }
        }
        _ => words.first().copied().unwrap_or(0) as f64,
    }
}

// ─── RTU Driver ───────────────────────────────────────

pub struct ModbusRTUDriver;

impl ModbusRTUDriver {
    pub fn new() -> Self { Self }
}

impl ProtocolDriver for ModbusRTUDriver {
    fn protocol_name(&self) -> &str { "Modbus RTU" }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let (port_name, baud) = match &device.bus {
            BusType::Serial { port_name, bus_param } => (port_name.clone(), bus_param.baud),
            _ => anyhow::bail!("ModbusRTU requires serial bus, got TCP"),
        };

        let handle = tokio::runtime::Handle::current();
        handle.block_on(async {
            let builder = tokio_serial::new(&port_name, baud)
                .timeout(Duration::from_secs(3));
            let port = builder.open_native()
                .with_context(|| format!("Open serial port {port_name}"))?;
            let mut ctx = tokio_modbus::client::rtu::connect(port)
                .context("ModbusRTU connect")?;
            ctx.set_slave(Slave(device.slave_addr));
            mb_collect_async(&mut ctx, device).await
        })
    }
}

// ─── TCP Driver ───────────────────────────────────────

pub struct ModbusTCPDriver;

impl ModbusTCPDriver {
    pub fn new() -> Self { Self }
}

impl ProtocolDriver for ModbusTCPDriver {
    fn protocol_name(&self) -> &str { "Modbus TCP" }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let socket_addr: SocketAddr = match &device.bus {
            BusType::Tcp { host, port } => format!("{host}:{port}").parse()
                .context("Invalid ModbusTCP address")?,
            _ => anyhow::bail!("ModbusTCP requires TCP bus"),
        };

        let handle = tokio::runtime::Handle::current();
        handle.block_on(async {
            let mut ctx = tokio_modbus::client::tcp::connect(socket_addr).await
                .context("ModbusTCP connect")?;
            ctx.set_slave(Slave(device.slave_addr));
            mb_collect_async(&mut ctx, device).await
        })
    }
}
