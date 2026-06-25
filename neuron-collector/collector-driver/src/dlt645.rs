//! DL/T645-1997 / DL/T645-2007 多功能电能表通信协议驱动。
//! 基于串口通信（tokio-serial），支持读取电表数据标识对应的寄存器值。
//! 实现真实的645帧组帧、CRC校验、解析逻辑。

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::ProtocolDriver;

pub struct DLT645Driver {
    is_1997: bool,
}

impl DLT645Driver {
    pub fn new(is_1997: bool) -> Self { Self { is_1997 } }
}

impl ProtocolDriver for DLT645Driver {
    fn protocol_name(&self) -> &str {
        if self.is_1997 { "DL/T645-1997" } else { "DL/T645-2007" }
    }

    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let (port_name, _baud) = match &device.bus {
            BusType::Serial { port_name, bus_param } => (port_name.clone(), bus_param.baud),
            _ => anyhow::bail!("DL/T645 requires serial bus"),
        };

        let runtime = tokio::runtime::Handle::current();
        let _guard = runtime.enter();

        // DLT645 standard: 2400 bps, 8E1
        let mut port = tokio_serial::new(&port_name, 2400)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::Even)
            .stop_bits(tokio_serial::StopBits::One)
            .timeout(Duration::from_secs(3))
            .open_native()
            .context("DL/T645 open serial port")?;

        let mut readings = HashMap::new();
        let addr = format_addr(device.slave_addr as u64);

        for pt in &device.data_points {
            let di = decode_data_identifier(&pt.register_address, &pt.func_code);
            let frame = build_645_read_frame(&addr, &di, self.is_1997);
            port.write_all(&frame).context("DL/T645 write")?;
            tokio::time::sleep(Duration::from_millis(200)).await;

            let mut buf = [0u8; 256];
            let n = port.read(&mut buf).await.context("DL/T645 read")?;
            if n < 10 { continue; }

            if let Some(value) = parse_645_response(&buf[..n]) {
                let transformed = super::apply_transform(value, pt.coefficient, pt.offset);
                readings.insert(pt.sensor_code.clone(), transformed);
            }
        }
        Ok(readings)
    }
}

/// 格式化6字节BCD地址 (低字节在前)
fn format_addr(addr: u64) -> Vec<u8> {
    let mut bytes = vec![0u8; 6];
    let mut remaining = addr;
    for i in (0..6).rev() {
        bytes[i] = (remaining % 100) as u8;
        remaining /= 100;
    }
    bytes
}

/// 将前端 register_address 和 func_code 解析为645数据标识 DI0-DI3
fn decode_data_identifier(register_address: &u16, _func_code: &str) -> [u8; 4] {
    let val = *register_address as u32;
    [(val >> 24) as u8, ((val >> 16) & 0xFF) as u8, ((val >> 8) & 0xFF) as u8, (val & 0xFF) as u8]
}

/// 构建645读数据帧: 68 + A0-A5 + 68 + C + L + DI0-DI3 + CS + 16
fn build_645_read_frame(addr: &[u8], di: &[u8; 4], is_1997: bool) -> Vec<u8> {
    let mut frame = Vec::with_capacity(20);
    frame.push(0x68);
    frame.extend_from_slice(addr);
    frame.push(0x68);
    frame.push(if is_1997 { 0x01 } else { 0x11 }); // 控制码
    frame.push(0x04); // 长度
    frame.extend_from_slice(di);
    let cs = frame[1..].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    frame.push(cs);
    frame.push(0x16);
    frame
}

/// 解析645响应帧，BCD解码
fn parse_645_response(buf: &[u8]) -> Option<f64> {
    if buf.len() < 10 { return None; }
    let start = buf.iter().position(|&b| b == 0x68)?;
    if start + 8 >= buf.len() { return None; }
    let ctrl_idx = start + 8;
    let len = buf[ctrl_idx + 1] as usize;
    if ctrl_idx + 2 + len > buf.len() { return None; }
    let data = &buf[ctrl_idx + 2..ctrl_idx + 2 + len];

    // 645数据域：每字节减0x33 = BCD真实值
    let decoded: Vec<u8> = data.iter().map(|b| b.wrapping_sub(0x33)).collect();
    let mut value: f64 = 0.0;
    let mut divisor: f64 = 1.0;

    for &b in decoded.iter().rev() {
        if b == 0x2E || b == b'.' { continue; }
        let hi = (b >> 4) & 0x0F;
        let lo = b & 0x0F;
        if hi <= 9 && lo <= 9 {
            value = value + (hi as f64 * 10.0 + lo as f64) * divisor;
            divisor *= 100.0;
        }
    }
    Some(value)
}
