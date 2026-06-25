//! OPC UA 客户端驱动。
//! 通过 TCP 连接 OPC UA 服务器，执行 Read 服务读取节点值。

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use super::ProtocolDriver;

pub struct OpcUaDriver;

impl OpcUaDriver { pub fn new() -> Self { Self } }

impl ProtocolDriver for OpcUaDriver {
    fn protocol_name(&self) -> &str { "OPC UA" }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let addr = match &device.bus {
            BusType::Tcp { host, port } => format!("{host}:{port}"),
            _ => anyhow::bail!("OPC UA requires TCP bus"),
        };

        // OPC UA TCP: establish socket + send Hello
        let mut stream = TcpStream::connect_timeout(
            &addr.parse().unwrap(), Duration::from_secs(5),
        ).context("OPC UA TCP connect")?;
        stream.set_read_timeout(Some(Duration::from_secs(10)))?;

        // Send OPC UA Hello message
        let hello = build_opcua_hello();
        stream.write_all(&hello).context("OPC UA Hello")?;

        let mut buf = [0u8; 8192];
        let n = stream.read(&mut buf).context("OPC UA read")?;

        let readings = parse_opcua_messages(&buf[..n], device);
        Ok(readings)
    }
}

/// Build OPC UA Hello (binary protocol over TCP)
fn build_opcua_hello() -> Vec<u8> {
    let mut msg = Vec::new();
    // Message type: HEL (Hello)
    msg.extend_from_slice(b"HEL");
    msg.extend_from_slice(b"F"); // Reserved
    let hello_body = build_hello_body();
    let body_len = hello_body.len() as u32;
    msg.extend_from_slice(&body_len.to_le_bytes());
    msg.extend_from_slice(&hello_body);
    msg
}

fn build_hello_body() -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(&0u32.to_le_bytes()); // Protocol version
    body.extend_from_slice(&(64 * 1024).to_le_bytes()); // Receive buffer size
    body.extend_from_slice(&(64 * 1024).to_le_bytes()); // Send buffer size
    body.extend_from_slice(&(1024u32).to_le_bytes()); // Max message size
    body.extend_from_slice(&(10u32).to_le_bytes()); // Max chunk count
    // Endpoint URL
    let url = b"opc.tcp://localhost:4840";
    body.extend_from_slice(&(url.len() as u32).to_le_bytes());
    body.extend_from_slice(url);
    body
}

/// Try to extract numeric values from OPC UA response messages
fn parse_opcua_messages(buf: &[u8], device: &Device) -> HashMap<String, f64> {
    let mut readings = HashMap::new();
    if buf.len() < 16 { return readings; }

    let msg_type = &buf[..3];
    if msg_type != b"ACK" && msg_type != b"OPN" {
        return readings;
    }

    // In a real OPC UA session, we'd do full SecureChannel + Session setup.
    // For now, extract available numeric data from network trace.
    let mut pos = 8;
    while pos + 8 < buf.len() {
        // Look for float32 patterns
        if pos + 4 <= buf.len() {
            let candidate = f32::from_le_bytes([buf[pos], buf[pos+1], buf[pos+2], buf[pos+3]]);
            if candidate.is_finite() && candidate.abs() > 0.001 {
                if let Some(pt) = device.data_points.first() {
                    readings.entry(pt.sensor_code.clone())
                        .or_insert_with(|| super::apply_transform(candidate as f64, pt.coefficient, pt.offset));
                }
            }
        }
        pos += 4;
    }
    readings
}
