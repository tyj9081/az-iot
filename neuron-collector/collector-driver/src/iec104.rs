//! IEC 60870-5-104 (IEC104) 远动协议 TCP 驱动。
//!
//! 电力系统标准规约，用于变电站与调度中心的数据通信。

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use super::ProtocolDriver;

pub struct IEC104Driver;

impl IEC104Driver { pub fn new() -> Self { Self } }

impl ProtocolDriver for IEC104Driver {
    fn protocol_name(&self) -> &str { "IEC 60870-5-104" }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let addr = match &device.bus {
            BusType::Tcp { host, port } => format!("{host}:{port}"),
            _ => anyhow::bail!("IEC104 requires TCP bus"),
        };

        let mut stream = TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            Duration::from_secs(5),
        ).context("IEC104 TCP connect")?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;

        // Send STARTDT activation
        iec104_send_u_frame(&mut stream, 0x07)?;
        std::thread::sleep(Duration::from_millis(200));

        let mut readings = HashMap::new();
        for pt in &device.data_points {
            let ioa = pt.register_address as u32;
            let cause = 6; // Activation
            let asdu_addr = device.slave_addr as u16;

            // Build C_IC_NA_1 (Interrogation) or C_RD_NA_1 (Read)
            let frame = build_iec104_i_frame(0, 0, cause, asdu_addr, ioa);
            stream.write_all(&frame).context("IEC104 write")?;

            let mut buf = [0u8; 256];
            let n = stream.read(&mut buf).context("IEC104 read")?;

            if let Some(val) = parse_iec104_i_frame(&buf[..n]) {
                let val = super::apply_transform(val, pt.coefficient, pt.offset);
                readings.insert(pt.sensor_code.clone(), val);
            }
        }

        // Send STOPDT activation
        iec104_send_u_frame(&mut stream, 0x13)?;
        Ok(readings)
    }
}

/// Send a U-format frame (STARTDT / STOPDT / TESTFR)
fn iec104_send_u_frame(stream: &mut TcpStream, ctrl: u8) -> Result<()> {
    let frame = [0x68, 0x04, ctrl, 0x00, 0x00, 0x00];
    stream.write_all(&frame)?;
    Ok(())
}

/// Build IEC104 I-format APDU with ASDU for reading single-point / measured value
fn build_iec104_i_frame(
    send_seq: u16, recv_seq: u16,
    cause: u8, asdu_addr: u16, ioa: u32,
) -> Vec<u8> {
    let asdu = build_asdu_interrogation(cause, asdu_addr, ioa);
    let apci_len = (asdu.len() + 2) as u8; // +2 for ASDU header

    let ctrl1 = ((send_seq & 0x7F) << 1) as u8;
    let ctrl2 = ((send_seq >> 7) & 0x7F) as u8;
    let ctrl3 = ((recv_seq & 0x7F) << 1) as u8;
    let ctrl4 = ((recv_seq >> 7) & 0x7F) as u8;

    let mut frame = vec![0x68, apci_len, ctrl1, ctrl2, ctrl3, ctrl4];
    frame.extend_from_slice(&asdu);
    frame
}

fn build_asdu_interrogation(cause: u8, asdu_addr: u16, ioa: u32) -> Vec<u8> {
    let mut asdu = Vec::new();
    // Type ID: 100 = C_IC_NA_1 (interrogation command)
    asdu.push(100);
    // VSQ: 1 element
    asdu.push(0x01);
    // COT
    asdu.push(cause);
    asdu.push(0x00);
    // Common address (2 bytes little-endian)
    asdu.extend_from_slice(&asdu_addr.to_le_bytes());
    // IOA (3 bytes)
    asdu.push((ioa & 0xFF) as u8);
    asdu.push(((ioa >> 8) & 0xFF) as u8);
    asdu.push(((ioa >> 16) & 0xFF) as u8);
    asdu
}

/// Parse IEC104 I-format response and extract measured value (Type 13 = measured short)
fn parse_iec104_i_frame(buf: &[u8]) -> Option<f64> {
    if buf.len() < 6 || buf[0] != 0x68 { return None; }
    let apci_len = buf[1] as usize;
    if buf.len() < 2 + apci_len { return None; }
    let asdu_start = 6;
    let asdu = &buf[asdu_start..2 + apci_len];
    if asdu.len() < 7 { return None; }
    let type_id = asdu[0];

    match type_id {
        13 => { // M_ME_NC_1 — measured short value
            if asdu.len() >= 10 {
                let raw = i16::from_le_bytes([asdu[7], asdu[8]]) as f64;
                Some(raw)
            } else { None }
        }
        1 => { // M_SP_NA_1 — single point
            if asdu.len() >= 7 {
                Some(if asdu[6] & 0x01 != 0 { 1.0 } else { 0.0 })
            } else { None }
        }
        9 => { // M_ME_NA_1 — measured normalized
            if asdu.len() >= 9 {
                let raw = i16::from_le_bytes([asdu[7], asdu[8]]) as f64 / 32768.0;
                Some(raw)
            } else { None }
        }
        _ => {
            // Try to extract any numeric value
            if asdu.len() >= 10 {
                let raw = i16::from_le_bytes([asdu[7], asdu[8]]) as f64;
                Some(raw)
            } else { None }
        }
    }
}
