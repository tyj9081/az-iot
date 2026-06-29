//! Siemens S7-300/400/1200/1500 PLC 驱动。
//! 实现 S7 通信协议 (ISO-on-TCP)，读取 DB/M/I/Q 区的数据。

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use super::ProtocolDriver;

pub struct S7CommDriver;

impl S7CommDriver { pub fn new() -> Self { Self } }

impl ProtocolDriver for S7CommDriver {
    fn protocol_name(&self) -> &str { "S7 Communication" }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let addr = match &device.bus {
            BusType::Tcp { host, port } => format!("{host}:{port}"),
            _ => anyhow::bail!("S7 requires TCP bus"),
        };

        let mut stream = TcpStream::connect_timeout(
            &addr.parse().unwrap(), Duration::from_secs(5),
        ).context("S7 TCP connect")?;
        stream.set_read_timeout(Some(Duration::from_secs(10)))?;

        // ISO-on-TCP connection request (COTP CR)
        let cotp_cr = s7_build_cotp_cr();
        stream.write_all(&cotp_cr).context("S7 COTP CR")?;
        let mut buf = [0u8; 256];
        let n = stream.read(&mut buf).context("S7 COTP CC")?;
        if n < 4 || buf[1] != 0xD0 { anyhow::bail!("S7 connection rejected"); }

        // S7 SETUP COMMUNICATION
        let setup = s7_build_setup_comm();
        stream.write_all(&setup).context("S7 setup")?;
        let _n = stream.read(&mut buf).context("S7 setup ack")?;

        // Read data for each data point
        let mut readings = HashMap::new();
        for pt in &device.data_points {
            let area = match pt.func_code.as_str() {
                "DB" | "db" => 0x84,
                "M" | "m" => 0x83,
                "I" | "i" => 0x81,
                "Q" | "q" => 0x82,
                _ => 0x84, // default to DB
            };
            let db_num = if area == 0x84 { (pt.register_address as u32 >> 16) as u16 } else { 0u16 };
            let byte_offset = (pt.register_address & 0xFFFF) as u16;

            let read_cmd = s7_build_read_var(area, db_num, byte_offset, 4);
            stream.write_all(&read_cmd).context("S7 read")?;
            let n = stream.read(&mut buf).context("S7 read response")?;

            if let Some(val) = parse_s7_read_response(&buf[..n], &pt.data_type) {
                let transformed = super::apply_transform(val, pt.coefficient, pt.offset);
                readings.insert(pt.sensor_code.clone(), transformed);
            }
        }
        Ok(readings)
    }
}

fn s7_header(tpkt_len: u16, payload: &[u8]) -> Vec<u8> {
    let mut hdr = vec![
        0x03, 0x00, // TPKT version + reserved
        (tpkt_len >> 8) as u8, (tpkt_len & 0xFF) as u8,
        0x02, 0xF0, 0x80, // ISO DT
    ];
    hdr.extend_from_slice(payload);
    hdr
}

fn s7_build_cotp_cr() -> Vec<u8> {
    let cr = vec![
        0x32, // Header length
        0x01, 0x00, 0x00, // PDU size ref
        0x00, 0x08, // Dest TSAP
        0x00, 0x08, // Src TSAP
        0x00, // TPDU size
    ];
    let tpkt_len = (4 + cr.len()) as u16;
    let mut frame = s7_header(tpkt_len, &[0xE0]); // CR
    frame.extend_from_slice(&[0x00, 0x00]); // Dest ref
    frame.extend_from_slice(&[0x00, 0x08]); // Src ref
    frame.extend_from_slice(&[0x00, 0x00]); // Options
    frame.extend_from_slice(&cr);
    frame
}

fn s7_build_setup_comm() -> Vec<u8> {
    let payload = vec![
        0x32, // Header
        0x01, // BLOB
        0x00, 0x00, // Reserved
        0x00, 0x08, // Max AmQ calling
        0x00, 0x08, // Max AmQ called
        0x00, 0x00, // PDU length
    ];
    let tpkt_len = (4 + 1 + 2 + 2 + payload.len()) as u16;
    let mut frame = s7_header(tpkt_len, &[0xF0, 0x00, 0x00]);
    let setup = [0x00, 0x01, 0x00, 0x1C, 0x00, 0x00]; // S7 header for setup
    frame.extend_from_slice(&setup);
    frame.extend_from_slice(&payload);
    frame
}

fn s7_build_read_var(area: u8, db_num: u16, byte_offset: u16, byte_len: u16) -> Vec<u8> {
    let mut item = vec![
        0x12, // Spec: 1 item
        0x0A, // Item length
        0x10, // Syntax: S7 Any
        0x02, // Transport size: BYTE
        (byte_len >> 8) as u8, (byte_len & 0xFF) as u8,
        (db_num >> 8) as u8, (db_num & 0xFF) as u8,
        area,
    ];
    // Byte address in bit format (byte * 8)
    let bit_addr = (byte_offset as u32) * 8;
    item.extend_from_slice(&[
        ((bit_addr >> 16) & 0xFF) as u8,
        ((bit_addr >> 8) & 0xFF) as u8,
        (bit_addr & 0xFF) as u8,
    ]);

    let s7_payload_len = 2 + item.len(); // Header + params
    let tpkt_len = (4 + 3 + s7_payload_len) as u16;
    let mut frame = s7_header(tpkt_len, &[0xF0, 0x00, 0x00]);

    // S7 request header
    frame.extend_from_slice(&[
        0x00, 0x01, 0x00, 0x00,
        0x04, // Read Var function
        0x01, // Item count
    ]);
    frame.extend_from_slice(&item);
    frame
}

fn parse_s7_read_response(buf: &[u8], dtype: &str) -> Option<f64> {
    if buf.len() < 25 { return None; }
    // TPKT + ISO + S7 header: 4 + 3 + 19 = 26 bytes before data
    let data_start = 25;
    if buf.len() < data_start + 2 { return None; }

    let ret_code = buf[17];
    if ret_code != 0xFF { return None; } // Error

    let data_len = buf[data_start] as usize;
    let data = &buf[data_start + 1..];
    if data.len() < data_len.min(4) { return None; }

    Some(match dtype {
        "float32" if data.len() >= 4 => f32::from_be_bytes([data[0], data[1], data[2], data[3]]) as f64,
        "int16" if data.len() >= 2 => i16::from_be_bytes([data[0], data[1]]) as f64,
        "uint16" if data.len() >= 2 => u16::from_be_bytes([data[0], data[1]]) as f64,
        "int32" if data.len() >= 4 => i32::from_be_bytes([data[0], data[1], data[2], data[3]]) as f64,
        "uint32" if data.len() >= 4 => u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as f64,
        "bool" => if data[0] != 0 { 1.0 } else { 0.0 },
        "float64" if data.len() >= 8 => f64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]),
        _ => {
            let mut val: u64 = 0;
            for &b in data.iter().take(data_len.min(8)) {
                val = (val << 8) | b as u64;
            }
            val as f64
        }
    })
}
