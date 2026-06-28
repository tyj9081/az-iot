//! AZ-IOT Collector 调试控制台
//!
//! 交互式 TUI 调试工具, 从 devices.json 加载设备列表,
//! 支持选择设备 → 选择点位 → 发送指令并展示 TX/RX 结果.
//!
//! # 使用方式
//!
//! ```bash
//! neuron-collector debug              # 从 devices.json 加载
//! neuron-collector debug --file path  # 从指定文件加载
//! ```

mod display;
mod menu;
mod runner;

use anyhow::Result;
use collector_model::Device;
use std::path::Path;

/// 直接从 async main() 调用, 复用已有 tokio runtime
pub async fn run(devices_path: Option<&str>) -> Result<()> {
    let path = devices_path.unwrap_or("devices.json");

    let devices: Vec<Device> = if Path::new(path).exists() {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json)?
    } else {
        // 没有设备文件时仍启动, 允许手动输入模式
        println!(
            "{}  设备文件 {} 不存在, 将以空设备列表启动",
            console::style(" ⚠ ").yellow(),
            console::style(path).yellow().bold()
        );
        vec![]
    };

    menu::main_loop(devices).await
}
