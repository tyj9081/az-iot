//! CAN Bus 驱动 — Linux SocketCAN 原生接口。
//!
//! 仅 Linux 可用。Windows/macOS 编译通过，运行时会返回明确错误。

use anyhow::{Context, Result};
use collector_model::{BusType, Device};
use std::collections::HashMap;
use std::io::Read;
use std::time::Duration;

use super::ProtocolDriver;

pub struct CanBusDriver;

impl CanBusDriver { pub fn new() -> Self { Self } }

impl ProtocolDriver for CanBusDriver {
    fn protocol_name(&self) -> &str { "CAN Bus" }

    #[cfg(target_os = "linux")]
    fn collect(&self, device: &Device) -> Result<HashMap<String, f64>> {
        let iface = match &device.bus {
            BusType::Serial { port_name, .. } => port_name.clone(),
            _ => anyhow::bail!("CAN driver requires network interface name (e.g. can0)"),
        };

        let socket = open_can_socket(&iface)?;
        let mut can_socket = socket;
        can_socket.set_read_timeout(Some(Duration::from_secs(5)))?;

        let mut readings = HashMap::new();
        let mut frame_buf = [0u8; 16];

        for pt in &device.data_points {
            let can_id = pt.register_address as u32;
            match can_socket.read(&mut frame_buf) {
                Ok(n) if n >= 16 => {
                    let id = u32::from_le_bytes([frame_buf[0], frame_buf[1], frame_buf[2], frame_buf[3]]);
                    let dlc = (frame_buf[4] as usize).min(8);
                    let data = &frame_buf[8..8 + dlc];
                    if id == can_id && !data.is_empty() {
                        let val = decode_can_data(data, &pt.data_type);
                        readings.insert(pt.sensor_code.clone(), super::apply_transform(val, pt.coefficient, pt.offset));
                    }
                }
                Ok(_) => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(readings)
    }

    #[cfg(not(target_os = "linux"))]
    fn collect(&self, _device: &Device) -> Result<HashMap<String, f64>> {
        anyhow::bail!("CAN bus driver requires Linux with SocketCAN. Current platform is not supported.")
    }
}

#[cfg(target_os = "linux")]
fn open_can_socket(iface: &str) -> Result<std::fs::File> {
    use std::os::unix::io::FromRawFd;
    let iface_c = std::ffi::CString::new(iface).context("Invalid interface name")?;
    let idx = unsafe { libc::if_nametoindex(iface_c.as_ptr()) };
    if idx == 0 { anyhow::bail!("CAN interface {iface} not found"); }

    let fd = unsafe {
        libc::socket(libc::AF_CAN, libc::SOCK_RAW, libc::CAN_RAW)
    };
    if fd < 0 { anyhow::bail!("Failed to create CAN socket"); }

    let addr = libc::sockaddr_can {
        can_family: libc::AF_CAN as u16,
        can_ifindex: idx as i32,
        ..unsafe { std::mem::zeroed() }
    };

    let ret = unsafe {
        libc::bind(fd, &addr as *const _ as *const libc::sockaddr, std::mem::size_of::<libc::sockaddr_can>() as u32)
    };
    if ret < 0 { anyhow::bail!("Failed to bind CAN socket"); }

    Ok(unsafe { std::fs::File::from_raw_fd(fd) })
}

fn decode_can_data(data: &[u8], dtype: &str) -> f64 {
    match dtype {
        "float32" if data.len() >= 4 => f32::from_le_bytes([data[0], data[1], data[2], data[3]]) as f64,
        "uint16" if data.len() >= 2 => u16::from_le_bytes([data[0], data[1]]) as f64,
        "int16" if data.len() >= 2 => i16::from_le_bytes([data[0], data[1]]) as f64,
        "uint32" if data.len() >= 4 => u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as f64,
        "int32" if data.len() >= 4 => i32::from_le_bytes([data[0], data[1], data[2], data[3]]) as f64,
        "bool" => if data[0] != 0 { 1.0 } else { 0.0 },
        _ if !data.is_empty() => data.iter().fold(0u64, |a, &b| (a << 8) | b as u64) as f64,
        _ => 0.0,
    }
}
