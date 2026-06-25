//! DNP3 (Distributed Network Protocol) TCP 驱动。
//! 实现 Class 0 轮询 — 请求所有静态数据点。

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use super::ProtocolDriver;

pub struct DNP3Driver;

impl DNP3Driver { pub fn new() -> Self { Self } }

impl ProtocolDriver for DNP3Driver {
    fn protocol_name(&self) -> &str { "DNP3" }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let addr = match &device.bus {
            BusType::Tcp { host, port } => format!("{host}:{port}"),
            _ => anyhow::bail!("DNP3 requires TCP bus"),
        };

        let mut stream = TcpStream::connect_timeout(
            &addr.parse().unwrap(), Duration::from_secs(5),
        ).context("DNP3 TCP connect")?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;

        // DNP3 request link status
        let req = build_dnp3_request(device.slave_addr as u16);
        stream.write_all(&req).context("DNP3 write")?;

        let mut buf = [0u8; 512];
        let n = stream.read(&mut buf).context("DNP3 read")?;

        let readings = parse_dnp3_response(&buf[..n], device);
        Ok(readings)
    }
}

/// Build a minimal DNP3 READ request (Function Code 0x01 = Read, Class 0)
fn build_dnp3_request(dest: u16) -> Vec<u8> {
    let mut frame = vec![
        0x05, 0x64,           // Start bytes
        0x0C,                 // Length (Data Link Header = 10 + CRC)
        0xC4,                 // Control: DIR=1 PRM=1 FCB=0
        (dest & 0xFF) as u8,  // Destination LSB
        ((dest >> 8) & 0xFF) as u8, // Destination MSB
        0x01, 0x00,           // Source = 1
    ];
    let crc1 = crc_dnp(&frame[2..8]);
    frame.extend_from_slice(&crc1.to_le_bytes());

    // Transport header + App header + Object header
    frame.extend_from_slice(&[
        0xC0, // Transport: FIR + FIN, seq=0
        0x02, // App seq
        0x01, // Function Code: Read
        0x3C, // IIN1
        0x02, // IIN2
        0x01, // Object Group 60 (Class objects)
        0x02, // Variation 2 (Class 0)
        0x06, // Qualifier: all points
        0x00, // Class 0
    ]);
    frame
}

/// CRC-16/DNP polynomial
fn crc_dnp(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xA6BC;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

/// Parse DNP3 response and extract analog/binary values matching device data points
fn parse_dnp3_response(buf: &[u8], device: &Device) -> HashMap<String, f64> {
    let mut readings = HashMap::new();
    if buf.len() < 12 { return readings; }

    // Scan for analog input objects (Group 30) or binary input objects (Group 1)
    let mut pos = 10; // After Data Link + Transport + App header
    while pos + 3 < buf.len() {
        let group = buf[pos];
        let var = buf[pos + 1];
        let qual = buf[pos + 2];
        pos += 3;

        match group {
            30 => {
                // Analog Input — parse range
                let count = if qual & 0x80 != 0 { (qual & 0x7F) as usize } else { 1 };
                for i in 0..count.min(device.data_points.len()) {
                    if pos + 4 <= buf.len() {
                        let value = f32::from_le_bytes([buf[pos], buf[pos+1], buf[pos+2], buf[pos+3]]) as f64;
                        if let Some(pt) = device.data_points.get(i) {
                            let val = super::apply_transform(value, pt.coefficient, pt.offset);
                            readings.insert(pt.sensor_code.clone(), val);
                        }
                        pos += 4;
                    }
                }
            }
            1 => {
                let count = (qual & 0x7F) as usize;
                for _ in 0..count {
                    if pos < buf.len() {
                        let val = if buf[pos] & 0x80 != 0 { 1.0 } else { 0.0 };
                        if let Some(pt) = device.data_points.first() {
                            readings.insert(pt.sensor_code.clone(), val);
                        }
                        pos += 1;
                    }
                }
            }
            _ => break,
        }
    }
    readings
}
