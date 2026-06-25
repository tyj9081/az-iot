//! Mitsubishi MELSEC MC 协议驱动 — 三菱 PLC 通信。
//! 支持 3E 帧（二进制）读写 D / M / X / Y 软元件。

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use super::ProtocolDriver;

pub struct MitsubishiMCDriver;

impl MitsubishiMCDriver { pub fn new() -> Self { Self } }

impl ProtocolDriver for MitsubishiMCDriver {
    fn protocol_name(&self) -> &str { "Mitsubishi MC" }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let addr = match &device.bus {
            BusType::Tcp { host, port } => format!("{host}:{port}"),
            _ => anyhow::bail!("Mitsubishi MC requires TCP bus"),
        };

        let mut stream = TcpStream::connect_timeout(
            &addr.parse().unwrap(), Duration::from_secs(5),
        ).context("Mitsubishi MC TCP connect")?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;

        let mut readings = HashMap::new();
        for pt in &device.data_points {
            let (dev_code, offset) = parse_mc_device(&pt.func_code, pt.register_address);
            let cmd = build_mc_3e_read(dev_code, offset, 1);
            stream.write_all(&cmd).context("MC write")?;

            let mut buf = [0u8; 256];
            let n = stream.read(&mut buf).context("MC read")?;

            if let Some(val) = parse_mc_response(&buf[..n], &pt.data_type) {
                let transformed = super::apply_transform(val, pt.coefficient, pt.offset);
                readings.insert(pt.sensor_code.clone(), transformed);
            }
        }
        Ok(readings)
    }
}

/// Parse device code from func_code (e.g. "D100" → code=0xA8, offset=100)
fn parse_mc_device(func_code: &str, register_addr: u16) -> (u8, u32) {
    let (code, base) = match func_code.to_uppercase().as_str() {
        "D" => (0xA8, 0u32),
        "M" => (0x90, 0u32),
        "X" => (0x9C, 0u32),
        "Y" => (0x9D, 0u32),
        "W" => (0xB4, 0u32),
        "L" => (0x92, 0u32),
        _ => (0xA8, 0u32), // default D
    };
    (code, base + register_addr as u32)
}

fn build_mc_3e_read(dev_code: u8, offset: u32, count: u16) -> Vec<u8> {
    let mut frame = vec![
        0x50, 0x00,       // Subheader
        0x00, 0xFF,       // Network + PC No
        0xFF, 0x03,       // Request dest + unit
        0x00,             // Request dest I/O
        0x0C, 0x00,       // CPU monitoring timer
        0x01, 0x04,       // Command: Read (0401)
        0x00, 0x00,       // Subcommand
    ];

    // Device specification (3 bytes: code + 24-bit address)
    frame.push(dev_code);
    frame.push((offset & 0xFF) as u8);
    frame.push(((offset >> 8) & 0xFF) as u8);
    frame.push(((offset >> 16) & 0xFF) as u8);

    // Word count
    frame.extend_from_slice(&count.to_le_bytes());
    frame
}

fn parse_mc_response(buf: &[u8], dtype: &str) -> Option<f64> {
    if buf.len() < 11 { return None; }
    let subheader = u16::from_le_bytes([buf[0], buf[1]]);
    if subheader != 0xD000 { return None; }

    let end_code = u16::from_le_bytes([buf[9], buf[10]]);
    if end_code != 0 { return None; }

    let data = &buf[11..];
    if data.is_empty() { return None; }

    Some(match dtype {
        "int16" if data.len() >= 2 => i16::from_le_bytes([data[0], data[1]]) as f64,
        "uint16" if data.len() >= 2 => u16::from_le_bytes([data[0], data[1]]) as f64,
        "int32" if data.len() >= 4 => i32::from_le_bytes([data[0], data[1], data[2], data[3]]) as f64,
        "uint32" if data.len() >= 4 => u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as f64,
        "float32" if data.len() >= 4 => f32::from_le_bytes([data[0], data[1], data[2], data[3]]) as f64,
        "float64" if data.len() >= 8 => {
            f64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]])
        }
        "bool" => {
            // Bit device: each word has 16 bits
            let word = u16::from_le_bytes([data[0], data[1]]);
            if word & 0x0001 != 0 { 1.0 } else { 0.0 }
        }
        _ if !data.is_empty() => u16::from_le_bytes([data[0], data[1]]) as f64,
        _ => 0.0,
    })
}
