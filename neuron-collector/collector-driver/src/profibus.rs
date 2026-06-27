//! PROFIBUS DP 驱动 — 过程现场总线协议（串口）。
//! 使用 DP-V0 主站轮询从站 I/O 数据。

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::time::Duration;

use super::ProtocolDriver;

pub struct ProfibusDPDriver;

impl ProfibusDPDriver { pub fn new() -> Self { Self } }

impl ProtocolDriver for ProfibusDPDriver {
    fn protocol_name(&self) -> &str { "PROFIBUS DP" }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let (port_name, baud) = match &device.bus {
            BusType::Serial { port_name, bus_param } => (port_name.clone(), bus_param.baud),
            _ => anyhow::bail!("PROFIBUS DP requires serial port"),
        };

        let runtime = tokio::runtime::Handle::current();
        let _guard = runtime.enter();

        let mut port = tokio_serial::new(&port_name, baud)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::Even)
            .stop_bits(tokio_serial::StopBits::One)
            .timeout(Duration::from_secs(5))
            .open_native()
            .context("PROFIBUS open port")?;

        let mut readings = HashMap::new();
        let req = build_profibus_dp_read(device.slave_addr);
        port.write_all(&req).context("PROFIBUS write")?;
        std::thread::sleep(Duration::from_millis(50));

        let mut buf = [0u8; 256];
        let n = port.read(&mut buf).context("PROFIBUS read")?;

        let input_data = parse_profibus_response(&buf[..n]);
        for (i, pt) in device.data_points.iter().enumerate() {
            let byte_offset = i * 2;
            if byte_offset + 1 < input_data.len() {
                let raw = u16::from_le_bytes([input_data[byte_offset], input_data[byte_offset + 1]]);
                let val = decode_profibus_value(raw, &pt.data_type);
                readings.insert(pt.sensor_code.clone(), super::apply_transform(val, pt.coefficient, pt.offset));
            }
        }
        Ok(readings)
    }
}

fn build_profibus_dp_read(slave: u8) -> Vec<u8> {
    let mut frame = vec![0x68, 0x03, 0x03, 0x68, slave, 0x01, 0x7C];
    let fcs = frame.iter().fold(0u8, |a, &b| a.wrapping_add(b));
    frame.push(fcs);
    frame.push(0x16);
    frame
}

fn parse_profibus_response(buf: &[u8]) -> Vec<u8> {
    if buf.len() < 9 || buf[0] != 0x68 { return vec![]; }
    let len = buf[1] as usize;
    if buf.len() < 7 + len { return vec![]; }
    if buf[6] == 0x08 { buf[7..7 + len - 3].to_vec() } else { vec![] }
}

fn decode_profibus_value(raw: u16, dtype: &str) -> f64 {
    match dtype {
        "bool" => if raw != 0 { 1.0 } else { 0.0 },
        "uint16" => raw as f64,
        "int16" => raw as i16 as f64,
        _ => raw as f64,
    }
}
