//! BACnet/IP 驱动 — 楼宇自动化控制网络协议。
//! 实现 Who-Is / I-Am 设备发现 + Read-Property 读取模拟/二进制值。

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::Duration;

use super::ProtocolDriver;

pub struct BacnetDriver;

impl BacnetDriver { pub fn new() -> Self { Self } }

impl ProtocolDriver for BacnetDriver {
    fn protocol_name(&self) -> &str { "BACnet/IP" }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let addr = match &device.bus {
            BusType::Tcp { host, port } => format!("{host}:{port}"),
            _ => anyhow::bail!("BACnet/IP requires UDP address"),
        };

        let socket = UdpSocket::bind("0.0.0.0:0").context("BACnet bind")?;
        socket.set_read_timeout(Some(Duration::from_secs(5)))?;
        socket.connect(&addr).context("BACnet connect")?;

        // Send Who-Is
        let who_is = build_bacnet_unconfirmed_req(device.slave_addr as u32, 8); // 8 = Who-Is
        socket.send(&who_is).context("BACnet Who-Is")?;

        let mut buf = [0u8; 1472];
        let n = socket.recv(&mut buf).unwrap_or(0);

        let mut readings = HashMap::new();
        if n > 0 {
            for pt in &device.data_points {
                let obj_id = pt.register_address as u32;
                let obj_type = match pt.data_type.as_str() {
                    "bool" => 3u32,    // Binary Input
                    _ => 0u32,          // Analog Input
                };
                let read_prop = build_bacnet_read_property(device.slave_addr as u32, obj_type, obj_id, 85);
                socket.send(&read_prop).ok();
                std::thread::sleep(Duration::from_millis(100));

                let mut rbuf = [0u8; 1472];
                if let Ok(rn) = socket.recv(&mut rbuf) {
                    if let Some(val) = parse_bacnet_value(&rbuf[..rn]) {
                        let transformed = super::apply_transform(val, pt.coefficient, pt.offset);
                        readings.insert(pt.sensor_code.clone(), transformed);
                    }
                }
            }
        }
        Ok(readings)
    }
}

/// Build BACnet Unconfirmed Request PDU (simplified)
fn build_bacnet_unconfirmed_req(dev_id: u32, service: u8) -> Vec<u8> {
    let mut pdu = vec![
        0x81, // BVLC: BACnet/IP
        0x0A, // Original-Unicast-NPDU
        0x00, 0x11, // Length placeholder
        0x01, // Version
        0x10, // Unconfirmed-Request PDU
    ];
    pdu.push(service);
    // Device instance
    pdu.extend_from_slice(&dev_id.to_be_bytes());
    pdu.extend_from_slice(&[0x00; 2]); // padding

    let len = pdu.len() as u16;
    pdu[2] = (len >> 8) as u8;
    pdu[3] = (len & 0xFF) as u8;
    pdu
}

/// Build Read-Property request
fn build_bacnet_read_property(dev_id: u32, obj_type: u32, obj_id: u32, prop_id: u32) -> Vec<u8> {
    let mut pdu = vec![
        0x81, 0x0A, 0x00, 0x00, // BVLC header
        0x01,                    // Version
        0x00,                    // Confirmed-Request
        0x05,                    // Max segments + max APDU
        0x01,                    // Invoke ID
        0x0C,                    // Service: ReadProperty
    ];
    // Object ID
    pdu.extend_from_slice(&((obj_type << 22 | obj_id) as u32).to_be_bytes());
    // Property ID
    pdu.push(prop_id as u8);
    // Optional: array index (omit for now)

    let len = pdu.len() as u16;
    pdu[2] = (len >> 8) as u8;
    pdu[3] = (len & 0xFF) as u8;
    pdu
}

fn parse_bacnet_value(buf: &[u8]) -> Option<f64> {
    if buf.len() < 10 { return None; }
    let apdu_start = 4;
    let apdu = &buf[apdu_start..];
    if apdu.len() < 8 { return None; }

    match apdu[0] {
        0x30 | 0x20 => {
            // Complex ACK / Simple ACK — try to find REAL tag
            let mut pos = 2;
            while pos + 2 < apdu.len() {
                let tag = apdu[pos] >> 4;
                let len = (apdu[pos] & 0x0F) as usize;
                pos += 1;
                if len == 5 && tag == 4 {
                    // REAL (float32)
                    if pos + 4 <= apdu.len() {
                        let val = f32::from_be_bytes([apdu[pos], apdu[pos+1], apdu[pos+2], apdu[pos+3]]) as f64;
                        return Some(val);
                    }
                }
                pos += len;
            }
            None
        }
        _ => None,
    }
}
