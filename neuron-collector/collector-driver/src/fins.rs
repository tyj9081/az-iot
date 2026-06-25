//! OMRON FINS/TCP 驱动 — 欧姆龙 PLC 通信协议。
//! 通过 FINS 命令读写 DM / CIO / WR / HR 区。

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use super::ProtocolDriver;

pub struct FinsTcpDriver;

impl FinsTcpDriver { pub fn new() -> Self { Self } }

impl ProtocolDriver for FinsTcpDriver {
    fn protocol_name(&self) -> &str { "FINS TCP" }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let addr = match &device.bus {
            BusType::Tcp { host, port } => format!("{host}:{port}"),
            _ => anyhow::bail!("FINS requires TCP bus"),
        };

        let mut stream = TcpStream::connect_timeout(
            &addr.parse().unwrap(), Duration::from_secs(5),
        ).context("FINS TCP connect")?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;

        // FINS/TCP header + FINS command frame
        let mut readings = HashMap::new();
        let src_net = 0u8;
        let src_node = 1u8;
        let src_unit = 0u8;
        let dest_net = 0u8;
        let dest_node = device.slave_addr;
        let dest_unit = 0u8;

        for pt in &device.data_points {
            let (area_code, addr) = parse_fins_area(&pt.func_code, pt.register_address);
            let cmd = build_fins_memory_read(
                src_net, src_node, src_unit,
                dest_net, dest_node, dest_unit,
                area_code, addr, 1,
            );

            // FINS/TCP header: 'FINS' + length + command + error code
            let mut frame = vec![b'F', b'I', b'N', b'S'];
            let len = cmd.len() as u32;
            frame.extend_from_slice(&len.to_be_bytes());
            frame.extend_from_slice(&[0x00, 0x00, 0x00, 0x02]); // Command + error
            frame.extend_from_slice(&cmd);

            stream.write_all(&frame).context("FINS write")?;

            let mut buf = [0u8; 512];
            let n = stream.read(&mut buf).context("FINS read")?;

            if let Some(val) = parse_fins_response(&buf[..n], &pt.data_type) {
                let transformed = super::apply_transform(val, pt.coefficient, pt.offset);
                readings.insert(pt.sensor_code.clone(), transformed);
            }
        }
        Ok(readings)
    }
}

/// Parse area code from func_code string (e.g. "DM", "CIO", "WR", "HR")
fn parse_fins_area(func_code: &str, register_addr: u16) -> (u8, u16) {
    match func_code.to_uppercase().as_str() {
        "DM" => (0x82, register_addr),
        "CIO" | "IO" => (0xB0, register_addr),
        "WR" => (0xB1, register_addr),
        "HR" => (0xB2, register_addr),
        "AR" => (0xB3, register_addr),
        _ => (0x82, register_addr), // default DM
    }
}

fn build_fins_memory_read(
    sna: u8, sa1: u8, sa2: u8,
    dna: u8, da1: u8, da2: u8,
    area: u8, addr: u16, count: u16,
) -> Vec<u8> {
    let mut cmd = vec![
        0x80, 0x00, 0x02, // ICF + RSV + GCT
        dna, da1, da2,
        sna, sa1, sa2,
        0x00, // SID
        0x01, 0x01, // MRC=01, SRC=01 (Memory Area Read)
    ];
    cmd.push(area);
    cmd.extend_from_slice(&addr.to_be_bytes());
    cmd.push(0x00); // Sub-address
    cmd.extend_from_slice(&count.to_be_bytes());
    cmd
}

fn parse_fins_response(buf: &[u8], dtype: &str) -> Option<f64> {
    if buf.len() < 16 { return None; }
    if &buf[..4] != b"FINS" { return None; }

    let cmd_start = 12;
    let cmd = &buf[cmd_start..];
    if cmd.len() < 14 { return None; }

    let end_code = u16::from_be_bytes([cmd[12], cmd[13]]);
    if end_code != 0 { return None; } // Error

    let data = &cmd[14..];
    if data.len() < 2 { return None; }

    match dtype {
        "int16" if data.len() >= 2 => Some(i16::from_be_bytes([data[0], data[1]]) as f64),
        "uint16" if data.len() >= 2 => Some(u16::from_be_bytes([data[0], data[1]]) as f64),
        "int32" if data.len() >= 4 => {
            Some(i32::from_be_bytes([data[0], data[1], data[2], data[3]]) as f64)
        }
        "uint32" if data.len() >= 4 => {
            Some(u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as f64)
        }
        "float32" if data.len() >= 4 => {
            Some(f32::from_be_bytes([data[0], data[1], data[2], data[3]]) as f64)
        }
        "bool" => Some(if data[0] != 0 { 1.0 } else { 0.0 }),
        _ if !data.is_empty() => Some(u16::from_be_bytes([data[0], data[1].min(data.len() as u8 - 1)]) as f64),
        _ => None,
    }
}
