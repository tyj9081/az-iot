//! 格式化输出辅助

use collector_model::{BusType, DataPoint, Device, ProtocolType};
use console::{style, Emoji};

/// 设备摘要 (用于列表选择)
pub fn device_summary(d: &Device) -> String {
    let status = if d.online {
        style("●").green()
    } else {
        style("○").red()
    };
    let proto = style(d.protocol.display_name()).dim();
    let bus = bus_short(&d.bus);

    format!(
        "[{:>4}] {:<20} {:<18} {:<24} {}  失败:{:>2}",
        d.id,
        truncate(&d.name, 20),
        proto,
        truncate(&bus, 24),
        status,
        d.consecutive_failures,
    )
}

/// 设备详情头
pub fn device_header(d: &Device) -> String {
    let status = if d.online {
        style("● ONLINE").green().bold()
    } else {
        style("○ OFFLINE").red().bold()
    };

    let bus_detail = bus_full(&d.bus);
    let last_ok = d.last_success_at
        .map(|ts| chrono::DateTime::from_timestamp(ts, 0)
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "?".into()))
        .unwrap_or_else(|| "-".into());

    format!(
        "═══ 设备 #{}: {} ═══\n\
         协议: {} | 从站: 0x{:02X} | 采集间隔: {}\n\
         {} | 状态: {} | 上次成功: {}\n\
         测点数: {}",
        d.id,
        style(&d.name).bold(),
        style(d.protocol.display_name()).cyan(),
        d.slave_addr,
        d.collect_interval_sec.map(|s| format!("{}s", s)).unwrap_or_else(|| "默认".into()),
        bus_detail,
        status,
        last_ok,
        d.data_points.len(),
    )
}

/// 点位摘要 (用于列表选择)
pub fn point_summary(p: &DataPoint, idx: usize) -> String {
    format!(
        "[{}] {:<24} addr=0x{:04X} fc={:<4} {:<8} x{} +{} → {}",
        idx + 1,
        truncate(&p.sensor_code, 24),
        p.register_address,
        p.func_code,
        p.data_type,
        p.coefficient,
        p.offset,
        p.unit,
    )
}

/// 测试结果展示
pub fn test_result(
    sensor_code: &str,
    register: u16,
    func: &str,
    value: Option<f64>,
    unit: &str,
    elapsed_ms: u64,
    error: Option<&str>,
) {
    // 分隔线
    println!(
        "{}",
        style("──────────────────────────────────────────────────").dim()
    );

    // 指令信息
    println!(
        "{} 测试 {}  addr=0x{:04X}  func={}",
        Emoji("▶", ">"),
        style(sensor_code).bold().cyan(),
        register,
        func,
    );

    // 结果
    if let Some(v) = value {
        println!(
            "{} 值: {} {}  |  耗时: {}ms",
            Emoji("✅", "[OK]"),
            style(format!("{:.4}", v)).green().bold(),
            style(unit).dim(),
            style(elapsed_ms).yellow(),
        );
    } else if let Some(e) = error {
        println!(
            "{} 失败: {}  |  耗时: {}ms",
            Emoji("❌", "[ERR]"),
            style(e).red().bold(),
            style(elapsed_ms).yellow(),
        );
    }

    println!(
        "{}",
        style("──────────────────────────────────────────────────").dim()
    );
}

/// 协议列表选项
pub fn protocol_options() -> Vec<(String, ProtocolType)> {
    use collector_model::ProtocolType::*;
    vec![
        ("Modbus RTU".into(), ModbusRTU),
        ("Modbus TCP".into(), ModbusTCP),
        ("DL/T645-2007".into(), DL645_2007),
        ("DL/T645-1997".into(), DL645_1997),
        ("IEC 60870-5-101".into(), IEC101),
        ("IEC 60870-5-104".into(), IEC104),
        ("DNP3".into(), DNP3),
        ("OPC UA".into(), OpcUa),
        ("BACnet/IP".into(), BacnetIP),
        ("S7 Communication".into(), S7Comm),
        ("FINS TCP".into(), FinsTcp),
        ("EtherNet/IP".into(), EthernetIP),
        ("Mitsubishi MC".into(), MitsubishiMC),
    ]
}

// ─── Helpers ────────────────────────────────────────────

fn bus_short(bus: &BusType) -> String {
    match bus {
        BusType::Serial { port_name, bus_param } => {
            format!("{} @{}", port_name, bus_param.baud)
        }
        BusType::Tcp { host, port } => {
            format!("{}:{}", host, port)
        }
    }
}

fn bus_full(bus: &BusType) -> String {
    match bus {
        BusType::Serial { port_name, bus_param } => {
            format!(
                "串口: {} | {} {} {} {}",
                style(port_name).yellow(),
                bus_param.baud,
                bus_param.data_bits,
                parity_display(&bus_param.parity),
                bus_param.stop_bits,
            )
        }
        BusType::Tcp { host, port } => {
            format!("TCP: {} | port {}", style(host).yellow(), port)
        }
    }
}

fn parity_display(p: &str) -> &str {
    match p {
        "none" | "N" => "N",
        "even" | "E" => "E",
        "odd" | "O" => "O",
        "mark" | "M" => "M",
        "space" | "S" => "S",
        _ => p,
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        format!("{}…", &s.chars().take(max - 1).collect::<String>())
    } else {
        s.to_string()
    }
}
