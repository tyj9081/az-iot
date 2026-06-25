//! SNMP v2c 驱动 — 网络设备监控。
//! 使用 SNMP GetRequest/GetNextRequest 读取 OID 对应的值。

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::Duration;

use super::ProtocolDriver;

pub struct SnmpV2cDriver;

impl SnmpV2cDriver { pub fn new() -> Self { Self } }

impl ProtocolDriver for SnmpV2cDriver {
    fn protocol_name(&self) -> &str { "SNMP v2c" }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let (host, port) = match &device.bus {
            BusType::Tcp { host, port: p } => (host.clone(), *p),
            BusType::Serial { .. } => anyhow::bail!("SNMP requires UDP host:port"),
            _ => anyhow::bail!("SNMP requires UDP host:port"),
        };

        let addr = format!("{host}:{port}");
        let socket = UdpSocket::bind("0.0.0.0:0").context("SNMP bind")?;
        socket.set_read_timeout(Some(Duration::from_secs(5)))?;

        let community = b"public";
        let mut readings = HashMap::new();

        for pt in &device.data_points {
            // DataPoint.register_address encodes the base OID suffix
            let oid = build_oid_from_register(pt.register_address);
            let req = build_snmp_get(community, &oid);
            socket.send_to(&req, &addr).context("SNMP send")?;

            let mut buf = [0u8; 1500];
            if let Ok(n) = socket.recv(&mut buf) {
                if let Some(val) = parse_snmp_response(&buf[..n]) {
                    let transformed = super::apply_transform(val, pt.coefficient, pt.offset);
                    readings.insert(pt.sensor_code.clone(), transformed);
                }
            }
        }
        Ok(readings)
    }
}

/// Build simple OID from register address: 1.3.6.1.2.1.X.Y
fn build_oid_from_register(addr: u16) -> Vec<u32> {
    let parent = addr as u32 / 100;
    let child = addr as u32 % 100;
    vec![1, 3, 6, 1, 2, 1, parent, child]
}

/// Build SNMP v2c GetRequest PDU (BER-encoded, minimal)
fn build_snmp_get(community: &[u8], oid: &[u32]) -> Vec<u8> {
    let mut pdu = Vec::new();
    pdu.push(0x30); // SEQUENCE
    let start = pdu.len();
    pdu.extend_from_slice(&[0x00, 0x00]); // placeholder length

    // Version
    pdu.extend_from_slice(&[0x02, 0x01, 0x01]); // INTEGER 1 (v2c)

    // Community
    pdu.push(0x04);
    pdu.push(community.len() as u8);
    pdu.extend_from_slice(community);

    // PDU: GetRequest (A0)
    pdu.push(0xA0);
    let pdu_start = pdu.len();
    pdu.extend_from_slice(&[0x00, 0x00]);

    // Request ID
    pdu.extend_from_slice(&[0x02, 0x02, 0x00, 0x01]);
    // Error
    pdu.extend_from_slice(&[0x02, 0x01, 0x00]);
    // Error index
    pdu.extend_from_slice(&[0x02, 0x01, 0x00]);
    // Varbind list
    pdu.push(0x30);
    let vb_start = pdu.len();
    pdu.push(0x00);
    // Varbind
    pdu.push(0x30);
    let v_start = pdu.len();
    pdu.push(0x00);
    // OID
    pdu.push(0x06);
    let mut oid_bytes = Vec::new();
    if !oid.is_empty() {
        oid_bytes.push((oid[0] * 40 + oid[1]) as u8);
        for &sub in &oid[2..] {
            encode_ber_subid(sub, &mut oid_bytes);
        }
    }
    pdu.push(oid_bytes.len() as u8);
    pdu.extend_from_slice(&oid_bytes);
    // NULL value
    pdu.extend_from_slice(&[0x05, 0x00]);

    let v_len = pdu.len() - v_start - 1;
    pdu[v_start] = v_len as u8 - 1; // adjust
    let vb_len = pdu.len() - vb_start - 1;
    pdu[vb_start] = vb_len as u8;

    let pdu_len = pdu.len() - pdu_start - 2;
    pdu[pdu_start] = (pdu_len >> 8) as u8;
    pdu[pdu_start + 1] = (pdu_len & 0xFF) as u8;

    let total = pdu.len() - start - 2;
    pdu[start] = total as u8;

    pdu
}

fn encode_ber_subid(sub: u32, out: &mut Vec<u8>) {
    if sub < 128 {
        out.push(sub as u8);
    } else {
        let mut parts = Vec::new();
        let mut v = sub;
        while v > 0 {
            parts.push((v & 0x7F) as u8);
            v >>= 7;
        }
        parts.reverse();
        for (i, p) in parts.iter().enumerate() {
            if i < parts.len() - 1 {
                out.push(p | 0x80);
            } else {
                out.push(*p);
            }
        }
    }
}

fn parse_snmp_response(buf: &[u8]) -> Option<f64> {
    if buf.len() < 10 || buf[0] != 0x30 { return None; }
    // Skip to the value field — search for INTEGER (0x02) or Counter (0x41) after OID+NULL
    let mut pos = 1;
    while pos + 2 < buf.len() {
        let tag = buf[pos];
        pos += 1;
        let mut len = buf[pos] as usize;
        pos += 1;
        if len & 0x80 != 0 {
            let llen = len & 0x7F;
            len = 0;
            for _ in 0..llen {
                len = (len << 8) | buf[pos] as usize;
                pos += 1;
            }
        }
        if tag == 0x02 || tag == 0x41 || tag == 0x42 {
            // INTEGER / Counter32 / Gauge32
            let mut val: u64 = 0;
            for _ in 0..len {
                val = (val << 8) | buf[pos] as u64;
                pos += 1;
            }
            return Some(val as f64);
        }
        pos += len;
    }
    None
}
