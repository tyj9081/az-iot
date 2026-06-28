//! 交互式菜单导航

use crate::display;
use crate::runner;
use collector_model::{BusParam, BusType, Device, ProtocolType};
use console::style;
use dialoguer::{Input, Select};

/// 菜单主循环
pub async fn main_loop(devices: Vec<Device>) -> anyhow::Result<()> {
    // 打印 Banner
    print_banner(&devices);

    // 主菜单循环
    loop {
        let choice = device_list_menu(&devices);
        match choice {
            DeviceMenuChoice::SelectDevice(d) => {
                device_detail_menu(&d);
            }
            DeviceMenuChoice::ManualTest => {
                manual_test_menu();
            }
            DeviceMenuChoice::Quit => {
                println!("  {}", style("👋 再见!").dim());
                break;
            }
        }
    }

    Ok(())
}

// ─── Banner ──────────────────────────────────────────────

fn print_banner(devices: &[Device]) {
    let online = devices.iter().filter(|d| d.online).count();
    let offline = devices.len() - online;

    println!();
    println!(
        "{}",
        style("╔══════════════════════════════════════════════╗").cyan()
    );
    println!(
        "{}  {}  {}",
        style("║").cyan(),
        style("🔧 AZ-IOT Collector Debug Console").bold(),
        style("║").cyan()
    );
    println!(
        "{}  Devices: {}  |  {} online / {} offline  {}",
        style("║").cyan(),
        devices.len(),
        style(online).green(),
        style(offline).red(),
        style("║").cyan()
    );
    println!(
        "{}",
        style("╚══════════════════════════════════════════════╝").cyan()
    );
    println!();
}

// ─── Device List Menu ────────────────────────────────────

enum DeviceMenuChoice {
    SelectDevice(Device),
    ManualTest,
    Quit,
}

fn device_list_menu(devices: &[Device]) -> DeviceMenuChoice {
    if devices.is_empty() {
        println!("  {}  暂无设备, 可通过手动模式测试\n", style("📭").dim());

        let items = vec![
            "🛠  手动输入测试",
            "←  退出",
        ];
        let selection = Select::new()
            .with_prompt("选择操作")
            .items(&items)
            .default(0)
            .interact()
            .unwrap_or(1);

        return if selection == 0 {
            DeviceMenuChoice::ManualTest
        } else {
            DeviceMenuChoice::Quit
        };
    }

    let mut items: Vec<String> = devices
        .iter()
        .map(|d| display::device_summary(d))
        .collect();
    items.push("🛠  手动输入测试 (不依赖已有配置)".into());
    items.push("←  退出".into());

    let selection = Select::new()
        .with_prompt("选择设备")
        .items(&items)
        .default(0)
        .interact()
        .unwrap_or(items.len() - 1);

    if selection == items.len() - 1 {
        DeviceMenuChoice::Quit
    } else if selection == items.len() - 2 {
        DeviceMenuChoice::ManualTest
    } else {
        DeviceMenuChoice::SelectDevice(devices[selection].clone())
    }
}

// ─── Device Detail Menu ──────────────────────────────────

fn device_detail_menu(device: &Device) {
    loop {
        // 清屏
        print!("\x1B[2J\x1B[H");

        // 打印设备详情头
        println!("{}\n", display::device_header(device));

        let items = vec![
            "▶  全部采集测试".to_string(),
            "▶  选择单个点位测试".to_string(),
            format!("📋 查看点位列表 ({} 个)", device.data_points.len()),
            "←  返回设备列表".to_string(),
        ];

        let selection = Select::new()
            .with_prompt("选择操作")
            .items(&items)
            .default(0)
            .interact()
            .unwrap_or(3);

        match selection {
            0 => {
                // 全部采集测试
                println!();
                run_all_points_test(device);
                wait_enter();
            }
            1 => {
                // 选择单个点位
                if device.data_points.is_empty() {
                    println!("\n  {}  该设备没有配置测点\n", style("⚠").yellow());
                    wait_enter();
                    continue;
                }
                point_select_menu(device);
            }
            2 => {
                // 查看点位列表
                println!();
                list_all_points(device);
                wait_enter();
            }
            3 | _ => break,
        }
    }
}

// ─── Point Select & Test ─────────────────────────────────

fn point_select_menu(device: &Device) {
    let points = &device.data_points;
    if points.is_empty() {
        println!("\n  {}  该设备没有配置测点\n", style("⚠").yellow());
        return;
    }

    loop {
        let mut items: Vec<String> = points
            .iter()
            .enumerate()
            .map(|(i, p)| display::point_summary(p, i))
            .collect();
        items.push("←  返回上级菜单".into());

        let selection = Select::new()
            .with_prompt("选择要测试的测点 (按 Enter 执行采集)")
            .items(&items)
            .default(0)
            .interact()
            .unwrap_or(items.len() - 1);

        if selection >= points.len() {
            break;
        }

        // 执行单点测试
        let point = &points[selection];
        println!();

        // 打印测试中提示
        println!(
            "  {} 正在采集 {} ...",
            style("⏳").yellow(),
            style(&point.sensor_code).bold()
        );

        let result = runner::test_single_point(device, point);

        // 打印结果
        display::test_result(
            &result.sensor_code,
            point.register_address,
            &point.func_code,
            result.value,
            &result.unit,
            result.elapsed_ms,
            result.error.as_deref(),
        );

        println!("  [Enter] 继续测试  |  [Esc/Ctrl+C] 返回\n");
        wait_enter();
    }
}

// ─── All Points Test ─────────────────────────────────────

fn run_all_points_test(device: &Device) {
    println!(
        "  {} 正在采集全部 {} 个测点 ...\n",
        style("⏳").yellow(),
        device.data_points.len()
    );

    let results = runner::test_all_points(device);

    // 统计
    let ok = results.iter().filter(|r| r.error.is_none()).count();
    let fail = results.len() - ok;
    println!(
        "  结果: {} / {}   {} / {}",
        style(ok).green().bold(),
        style("OK").green(),
        style(fail).red().bold(),
        style("FAIL").red(),
    );
    println!(
        "  {}",
        style("──────────────────────────────────────────────").dim()
    );

    for r in &results {
        let status = if r.error.is_none() {
            style("✅").green()
        } else {
            style("❌").red()
        };
        match &r.error {
            None => {
                println!(
                    "  {} {:<24} = {:<12} {}  {}ms",
                    status,
                    style(&r.sensor_code).bold(),
                    style(format!("{:.4}", r.value.unwrap_or(0.0))).cyan(),
                    style(&r.unit).dim(),
                    style(r.elapsed_ms).yellow(),
                );
            }
            Some(e) => {
                println!(
                    "  {} {:<24} {}  {}ms",
                    status,
                    style(&r.sensor_code).bold(),
                    style(e).red(),
                    style(r.elapsed_ms).yellow(),
                );
            }
        }
    }
    println!();
}

// ─── Point List ──────────────────────────────────────────

fn list_all_points(device: &Device) {
    if device.data_points.is_empty() {
        println!("\n  (无测点)\n");
        return;
    }
    println!(
        "\n  {}\n  {}",
        style("点位列表:").bold().underlined(),
        style("──────────────────────────────────────────────").dim(),
    );
    for (i, p) in device.data_points.iter().enumerate() {
        println!(
            "  {:>3}. {:<24}  reg=0x{:04X}  fc={:<4}  type={:<8}  byte={:<4}  x{} +{} → {}",
            i + 1,
            style(&p.sensor_code).bold(),
            p.register_address,
            p.func_code,
            p.data_type,
            p.byte_order,
            p.coefficient,
            p.offset,
            p.unit,
        );
    }
    println!();
}

// ─── Manual Test Menu ────────────────────────────────────

fn manual_test_menu() {
    loop {
        print!("\x1B[2J\x1B[H");
        println!(
            "{}",
            style("═══ 手动测试模式 ═══").bold().cyan()
        );
        println!("  输入串口/TCP 参数, 构造指令并发送\n");

        let items = vec![
            "串口测试 (COM口 / 波特率 / 从站 / 寄存器)",
            "TCP 测试 (IP:端口 / 从站 / 寄存器)",
            "←  返回主菜单",
        ];

        let selection = Select::new()
            .with_prompt("选择连接方式")
            .items(&items)
            .default(0)
            .interact()
            .unwrap_or(2);

        match selection {
            0 => manual_serial_test(),
            1 => manual_tcp_test(),
            _ => break,
        }
    }
}

fn manual_serial_test() {
    print!("\x1B[2J\x1B[H");

    // 选择协议
    let proto_options: Vec<String> = display::protocol_options()
        .iter()
        .map(|(name, _)| name.clone())
        .collect();
    let proto_idx = Select::new()
        .with_prompt("选择协议")
        .items(&proto_options)
        .default(0)
        .interact()
        .unwrap_or(0);
    let protocol = display::protocol_options()[proto_idx].1.clone();

    // 串口参数
    let port: String = Input::new()
        .with_prompt("串口号")
        .default("COM3".into())
        .interact_text()
        .unwrap_or_else(|_| "COM3".into());

    let baud: u32 = Input::new()
        .with_prompt("波特率")
        .default(9600u32)
        .interact_text()
        .unwrap_or(9600);

    let slave_addr: u8 = Input::new()
        .with_prompt("从站地址 (十六进制)")
        .default("01".into())
        .interact_text()
        .ok()
        .and_then(|s: String| u8::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or(1);

    // 寄存器参数
    let (register, count, func_code, data_type, byte_order) = input_register_params();

    let bus = BusType::Serial {
        port_name: port,
        bus_param: BusParam {
            baud,
            data_bits: 8,
            stop_bits: 1,
            parity: "none".into(),
        },
    };

    let device = runner::build_manual_device(
        protocol, bus, slave_addr,
        register, count, &func_code, &data_type, &byte_order,
    );

    println!();
    run_manual_test(&device);
}

fn manual_tcp_test() {
    print!("\x1B[2J\x1B[H");

    // 选择协议
    let proto_options: Vec<String> = display::protocol_options()
        .iter()
        .filter(|(_, p)| p.is_tcp())
        .map(|(name, _)| name.clone())
        .collect();
    let proto_idx = Select::new()
        .with_prompt("选择协议")
        .items(&proto_options)
        .default(0)
        .interact()
        .unwrap_or(0);

    let tcp_protocols: Vec<ProtocolType> = display::protocol_options()
        .into_iter()
        .filter(|(_, p)| p.is_tcp())
        .map(|(_, p)| p)
        .collect();
    let protocol = tcp_protocols[proto_idx].clone();

    let host: String = Input::new()
        .with_prompt("IP 地址")
        .default("192.168.1.100".into())
        .interact_text()
        .unwrap_or_else(|_| "192.168.1.100".into());

    let port: u16 = Input::new()
        .with_prompt("端口")
        .default(502u16)
        .interact_text()
        .unwrap_or(502);

    let slave_addr: u8 = Input::new()
        .with_prompt("从站地址 (十六进制)")
        .default("01".into())
        .interact_text()
        .ok()
        .and_then(|s: String| u8::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or(1);

    let (register, count, func_code, data_type, byte_order) = input_register_params();

    let bus = BusType::Tcp { host, port };
    let device = runner::build_manual_device(
        protocol, bus, slave_addr,
        register, count, &func_code, &data_type, &byte_order,
    );

    println!();
    run_manual_test(&device);
}

fn input_register_params() -> (u16, u16, String, String, String) {
    let register: u16 = Input::new()
        .with_prompt("寄存器地址 (十六进制)")
        .default("0000".into())
        .interact_text()
        .ok()
        .and_then(|s: String| u16::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0);

    let count: u16 = Input::new()
        .with_prompt("寄存器数量")
        .default(2u16)
        .interact_text()
        .unwrap_or(2);

    let func_code: String = Input::new()
        .with_prompt("功能码")
        .default("3".into())
        .interact_text()
        .unwrap_or_else(|_| "3".into());

    let data_type: String = Input::new()
        .with_prompt("数据类型")
        .default("float32".into())
        .interact_text()
        .unwrap_or_else(|_| "float32".into());

    let byte_order: String = Input::new()
        .with_prompt("字节序")
        .default("ABCD".into())
        .interact_text()
        .unwrap_or_else(|_| "ABCD".into());

    (register, count, func_code, data_type, byte_order)
}

fn run_manual_test(device: &Device) {
    let point = &device.data_points[0];

    println!(
        "  {} 发送指令到 {} ...",
        style("⏳").yellow(),
        match &device.bus {
            BusType::Serial { port_name, .. } => port_name.clone(),
            BusType::Tcp { host, port } => format!("{}:{}", host, port),
        },
    );

    let result = runner::test_single_point(device, point);

    display::test_result(
        &result.sensor_code,
        point.register_address,
        &point.func_code,
        result.value,
        &result.unit,
        result.elapsed_ms,
        result.error.as_deref(),
    );

    wait_enter();
}

// ─── Helpers ─────────────────────────────────────────────

fn wait_enter() {
    println!("  按 [Enter] 继续...");
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf);
}
