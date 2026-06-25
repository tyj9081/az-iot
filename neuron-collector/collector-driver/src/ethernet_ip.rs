//! EtherNet/IP (CIP) 驱动 — Rockwell/Allen-Bradley PLC 通信。
//! 实现 CIP Connection + Read Tag Service。

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use super::ProtocolDriver;

pub struct EthernetIPDriver;

impl EthernetIPDriver { pub fn new() -> Self { Self } }

impl ProtocolDriver for EthernetIPDriver {
    fn protocol_name(&self) -> &str { "EtherNet/IP" }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let addr = match &device.bus {
            BusType::Tcp { host, port } => format!("{host}:{port}"),
            _ => anyhow::bail!("EtherNet/IP requires TCP bus"),
        };

        let mut stream = TcpStream::connect_timeout(
            &addr.parse().unwrap(), Duration::from_secs(5),
        ).context("EtherNet/IP TCP connect")?;
        stream.set_read_timeout(Some(Duration::from_secs(10)))?;

        // Register EIP session
        let reg = build_eip_register_session();
        stream.write_all(&reg).context("EIP register")?;
        let mut buf = [0u8; 512];
        let n = stream.read(&mut buf).context("EIP register response")?;
        let session = parse_eip_session(&buf[..n]).unwrap_or(0);

        // Forward Open (CIP connection)
        let fwd_open = build_eip_forward_open(session);
        stream.write_all(&fwd_open).context("EIP forward open")?;
        let n = stream.read(&mut buf).context("EIP forward open response")?;
        let conn_id = parse_cip_connection_id(&buf[..n]).unwrap_or(0);

        // Read tags
        let mut readings = HashMap::new();
        for pt in &device.data_points {
            let tag_name = pt.sensor_code.as_bytes();
            let read_tag = build_cip_read_tag(session, conn_id, tag_name, 1);
            stream.write_all(&read_tag).context("EIP read tag")?;
            let n = stream.read(&mut buf).context("EIP read response")?;

            if let Some(val) = parse_cip_data(&buf[..n], &pt.data_type) {
                let transformed = super::apply_transform(val, pt.coefficient, pt.offset);
                readings.insert(pt.sensor_code.clone(), transformed);
            }
        }
        Ok(readings)
    }
}

fn eip_header(command: u16, session: u32, payload_len: u16) -> Vec<u8> {
    let mut hdr = Vec::with_capacity(24);
    hdr.extend_from_slice(&command.to_le_bytes());
    hdr.extend_from_slice(&payload_len.to_le_bytes());
    hdr.extend_from_slice(&session.to_le_bytes());
    hdr.extend_from_slice(&[0u8; 4]); // Status + Context1
    hdr.extend_from_slice(&[0u8; 8]); // Context2 + Options
    hdr.extend_from_slice(&[0u8; 4]); // Handle + Timeout
    hdr
}

fn build_eip_register_session() -> Vec<u8> {
    let mut payload = vec![
        0x01, 0x00, // Protocol version
        0x00, 0x00, // Flags
    ];
    let mut frame = eip_header(0x0065, 0, payload.len() as u16);
    frame.extend_from_slice(&payload);
    frame
}

fn parse_eip_session(buf: &[u8]) -> Option<u32> {
    if buf.len() < 8 { return None; }
    Some(u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]))
}

fn build_eip_forward_open(session: u32) -> Vec<u8> {
    // CIP Forward Open request inside Unconnected Send
    let cip_data = vec![
        // CIP Request
        0x54, 0x02, 0x20, 0x06, 0x24, 0x01, // Interface + timeout
        0x0A, 0xF0, 0x00, 0x00,             // Connection serial + vendor
        0x00, 0x00, 0x00,                    // Connection serial
        0xA1, 0x00, 0x04, 0x00,             // O→T RPI
        0x00, 0x00,                          // O→T params
        0xA2, 0x00, 0x04, 0x00,             // T→O RPI
        0x00, 0x00,                          // T→O params
        0x01, 0x00, 0x02, 0x00, 0x01, 0x00, // Transport trigger
        0x00, 0x00,                          // Connection path size
    ];
    let mut frame = eip_header(0x006F, session, cip_data.len() as u16);
    frame.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // Interface handle + timeout
    frame.extend_from_slice(&[0x02, 0x00]); // Item count
    frame.extend_from_slice(&[0x00, 0x00]); // Connected addr item
    frame.extend_from_slice(&[0x00, 0x00]); // Length
    frame.extend_from_slice(&[0xB2, 0x00]); // Unconnected data item
    frame.extend_from_slice(&(cip_data.len() as u16).to_le_bytes());
    frame.extend_from_slice(&cip_data);
    frame
}

fn parse_cip_connection_id(buf: &[u8]) -> Option<u32> {
    if buf.len() < 48 { return None; }
    Some(u32::from_le_bytes([buf[44], buf[45], buf[46], buf[47]]))
}

fn build_cip_read_tag(session: u32, conn_id: u32, tag: &[u8], count: u16) -> Vec<u8> {
    let mut cip = vec![
        0x4C, // Read Tag Service
        (count & 0xFF) as u8, ((count >> 8) & 0xFF) as u8, // Request size
        0x91, // Connected address item
    ];
    cip.extend_from_slice(&conn_id.to_le_bytes());
    cip.push(0x00); // Symbol type
    cip.push(tag.len() as u8);
    cip.extend_from_slice(tag);
    cip.extend_from_slice(&[0x01, 0x00]); // Element count

    let mut frame = eip_header(0x0070, session, cip.len() as u16);
    frame.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // Interface
    frame.extend_from_slice(&[0x02, 0x00]); // Item count
    frame.extend_from_slice(&[0xA1, 0x00]); // Connected data item
    frame.extend_from_slice(&(cip.len() as u16).to_le_bytes());
    frame.extend_from_slice(&cip);
    frame
}

fn parse_cip_data(buf: &[u8], dtype: &str) -> Option<f64> {
    if buf.len() < 30 { return None; }
    let data_start = 26;
    let data = &buf[data_start..];
    if data.len() < 2 { return None; }
    let dtype = data[0];
    match dtype {
        0xC4 => {
            // REAL (float32)
            if data.len() >= 6 {
                Some(f32::from_le_bytes([data[2], data[3], data[4], data[5]]) as f64)
            } else { None }
        }
        0xC3 => {
            // DINT (int32)
            if data.len() >= 6 {
                Some(i32::from_le_bytes([data[2], data[3], data[4], data[5]]) as f64)
            } else { None }
        }
        0xC2 => {
            // INT (int16)
            if data.len() >= 4 {
                Some(i16::from_le_bytes([data[2], data[3]]) as f64)
            } else { None }
        }
        0xC1 => {
            // BOOL
            Some(if data[2] != 0 { 1.0 } else { 0.0 })
        }
        _ => None,
    }
}
