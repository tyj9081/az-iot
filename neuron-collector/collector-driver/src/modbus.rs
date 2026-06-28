//! Modbus RTU / TCP drivers backed by tokio_modbus.
//! Real I/O over serial port (RTU) or TCP socket (TCP).

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio_modbus::prelude::*;
use tokio_serial::{DataBits, Parity, SerialPortBuilderExt, StopBits};

use super::{block_on_driver_runtime, ProtocolDriver};

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

        let read_start = std::time::Instant::now();
        tracing::debug!("Modbus read start: fc={} addr={} count={}", fc, addr, count);

        let raw: Vec<u16> = match fc {
            1 | 2 => {
                let result = ctx.read_coils(addr, count).await
                    .with_context(|| format!("Read coils at {addr}"))?;
                match result {
                    Ok(coils) => {
                        tracing::debug!(
                            "Modbus coils read OK: addr={} count={} elapsed={}ms",
                            addr, count, read_start.elapsed().as_millis()
                        );
                        coils.iter().map(|&b| if b { 1u16 } else { 0u16 }).collect()
                    }
                    Err(exception) => {
                        return Err(anyhow::anyhow!(
                            "Modbus exception fc=0x{:02X} addr={} count={}: {:?}",
                            fc, addr, count, exception
                        ));
                    }
                }
            }
            3 | 4 => {
                let result = ctx.read_holding_registers(addr, count).await
                    .with_context(|| format!("Read registers at {addr}"))?;
                match result {
                    Ok(data) => {
                        tracing::debug!(
                            "Modbus registers read OK: addr={} count={} elapsed={}ms",
                            addr, count, read_start.elapsed().as_millis()
                        );
                        data
                    }
                    Err(exception) => {
                        return Err(anyhow::anyhow!(
                            "Modbus exception fc=0x{:02X} addr={} count={}: {:?}",
                            fc, addr, count, exception
                        ));
                    }
                }
            }
            _ => {
                let result = ctx.read_holding_registers(addr, count).await
                    .with_context(|| format!("Read registers at {addr}"))?;
                match result {
                    Ok(data) => {
                        tracing::debug!(
                            "Modbus registers read OK (default fc): addr={} count={} elapsed={}ms",
                            addr, count, read_start.elapsed().as_millis()
                        );
                        data
                    }
                    Err(exception) => {
                        return Err(anyhow::anyhow!(
                            "Modbus exception fc=0x{:02X} addr={} count={}: {:?}",
                            fc, addr, count, exception
                        ));
                    }
                }
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
        let (port_name, bus_param) = match &device.bus {
            BusType::Serial { port_name, bus_param } => (port_name.clone(), bus_param.clone()),
            _ => anyhow::bail!("ModbusRTU requires serial bus, got TCP"),
        };

        // spawn+channel 替代 block_on，彻底避免全局 context 检查导致的 panic
        let slave_addr = device.slave_addr;
        let device_clone = device.clone();
        block_on_driver_runtime(async move {
            let port = tokio_serial::new(&port_name, bus_param.baud)
                .data_bits(match bus_param.data_bits {
                    5 => DataBits::Five,
                    6 => DataBits::Six,
                    7 => DataBits::Seven,
                    _ => DataBits::Eight,
                })
                .parity(match bus_param.parity.as_str() {
                    "odd" => Parity::Odd,
                    "even" => Parity::Even,
                    _ => Parity::None,
                })
                .stop_bits(match bus_param.stop_bits {
                    2 => StopBits::Two,
                    _ => StopBits::One,
                })
                .timeout(Duration::from_secs(3))
                .open_native_async()
                .with_context(|| format!("Open serial port {port_name}"))?;
            let mut ctx = tokio_modbus::client::rtu::attach_slave(port, Slave(slave_addr));
            mb_collect_async(&mut ctx, &device_clone).await
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

        // spawn+channel 替代 block_on，彻底避免全局 context 检查导致的 panic
        let slave_addr = device.slave_addr;
        let device_clone = device.clone();
        block_on_driver_runtime(async move {
            let mut ctx = tokio_modbus::client::tcp::connect(socket_addr).await
                .context("ModbusTCP connect")?;
            ctx.set_slave(Slave(slave_addr));
            mb_collect_async(&mut ctx, &device_clone).await
        })
    }
}
