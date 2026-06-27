//! IEC 60870-5-101 平衡模式串口驱动。
//! FT1.2 帧格式，支持 C_IC_NA_1 总召唤。

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::time::Duration;

use super::ProtocolDriver;

pub struct IEC101Driver;

impl IEC101Driver { pub fn new() -> Self { Self } }

impl ProtocolDriver for IEC101Driver {
    fn protocol_name(&self) -> &str { "IEC 60870-5-101" }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let (port_name, baud) = match &device.bus {
            BusType::Serial { port_name, bus_param } => (port_name.clone(), bus_param.baud),
            _ => anyhow::bail!("IEC101 requires serial bus"),
        };

        let runtime = tokio::runtime::Handle::current();
        let _guard = runtime.enter();

        let mut port = tokio_serial::new(&port_name, baud)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::Even)
            .stop_bits(tokio_serial::StopBits::One)
            .timeout(Duration::from_secs(5))
            .open_native()
            .context("IEC101 open port")?;

        let mut readings = HashMap::new();
        for pt in &device.data_points {
            let asdu_addr = device.slave_addr as u16;
            let ioa = pt.register_address as u32;
            let frame = build_iec101_frame(asdu_addr, 100, 6, ioa);
            port.write_all(&frame).context("IEC101 write")?;
            std::thread::sleep(Duration::from_millis(300));

            let mut buf = [0u8; 256];
            let n = port.read(&mut buf).context("IEC101 read")?;
            if let Some(val) = parse_iec101_response(&buf[..n]) {
                readings.insert(pt.sensor_code.clone(), super::apply_transform(val, pt.coefficient, pt.offset));
            }
        }
        Ok(readings)
    }
}

fn build_iec101_frame(asdu_addr: u16, type_id: u8, cause: u8, ioa: u32) -> Vec<u8> {
    let mut asdu = Vec::new();
    asdu.push(type_id);
    asdu.push(0x01);
    asdu.push(cause);
    asdu.push(0x00);
    asdu.extend_from_slice(&asdu_addr.to_le_bytes());
    asdu.push((ioa & 0xFF) as u8);
    asdu.push(((ioa >> 8) & 0xFF) as u8);
    asdu.push(((ioa >> 16) & 0xFF) as u8);

    let len = asdu.len() as u8;
    let cs = len.wrapping_add(asdu.iter().fold(0u8, |a, &b| a.wrapping_add(b)));

    let mut frame = vec![0x68, len, len, 0x68, 0x53];
    frame.extend_from_slice(&asdu);
    frame.push(cs);
    frame.push(0x16);
    frame
}

fn parse_iec101_response(buf: &[u8]) -> Option<f64> {
    if buf.len() < 6 || buf[0] != 0x68 { return None; }
    let len = buf[1] as usize;
    if buf.len() < len + 6 { return None; }
    let asdu = &buf[4..4 + len];
    if asdu.len() < 7 { return None; }
    match asdu[0] {
        13 | 9 => {
            if asdu.len() >= 9 { Some(i16::from_le_bytes([asdu[7], asdu[8]]) as f64) } else { None }
        }
        1 => {
            if asdu.len() >= 7 { Some(if asdu[6] & 0x01 != 0 { 1.0 } else { 0.0 }) } else { None }
        }
        _ => None,
    }
}
