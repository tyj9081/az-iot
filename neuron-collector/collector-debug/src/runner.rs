//! 采集执行器 — 构造临时设备并调用 DriverFactory 执行采集

use collector_driver::DriverFactory;
use collector_model::{BusType, DataPoint, Device, ProtocolType};
use std::time::Instant;

/// 单点测试结果
pub struct PointTestResult {
    pub sensor_code: String,
    pub value: Option<f64>,
    pub unit: String,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

/// 对单个测点执行采集测试
pub fn test_single_point(
    device: &Device,
    point: &DataPoint,
) -> PointTestResult {
    let mut test_device = device.clone();
    test_device.data_points = vec![point.clone()];

    let start = Instant::now();

    let driver = match DriverFactory::create(&test_device) {
        Some(d) => d,
        None => {
            return PointTestResult {
                sensor_code: point.sensor_code.clone(),
                value: None,
                unit: point.unit.clone(),
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: Some(format!(
                    "协议 {:?} 在当前平台不可用",
                    test_device.protocol
                )),
            };
        }
    };

    match driver.collect(&test_device) {
        Ok(values) => {
            if let Some(v) = values.get(&point.sensor_code) {
                PointTestResult {
                    sensor_code: point.sensor_code.clone(),
                    value: Some(*v),
                    unit: point.unit.clone(),
                    elapsed_ms: start.elapsed().as_millis() as u64,
                    error: None,
                }
            } else {
                PointTestResult {
                    sensor_code: point.sensor_code.clone(),
                    value: None,
                    unit: point.unit.clone(),
                    elapsed_ms: start.elapsed().as_millis() as u64,
                    error: Some("采集返回为空 (设备无响应或点位不存在)".into()),
                }
            }
        }
        Err(e) => {
            PointTestResult {
                sensor_code: point.sensor_code.clone(),
                value: None,
                unit: point.unit.clone(),
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("{:#}", e)),
            }
        }
    }
}

/// 对设备全部测点执行采集测试
pub fn test_all_points(device: &Device) -> Vec<PointTestResult> {
    let start = Instant::now();

    let driver = match DriverFactory::create(device) {
        Some(d) => d,
        None => {
            return device
                .data_points
                .iter()
                .map(|p| PointTestResult {
                    sensor_code: p.sensor_code.clone(),
                    value: None,
                    unit: p.unit.clone(),
                    elapsed_ms: 0,
                    error: Some(format!("协议 {:?} 在当前平台不可用", device.protocol)),
                })
                .collect();
        }
    };

    match driver.collect(device) {
        Ok(values) => {
            let elapsed = start.elapsed().as_millis() as u64;
            device
                .data_points
                .iter()
                .map(|p| {
                    if let Some(v) = values.get(&p.sensor_code) {
                        PointTestResult {
                            sensor_code: p.sensor_code.clone(),
                            value: Some(*v),
                            unit: p.unit.clone(),
                            elapsed_ms: elapsed,
                            error: None,
                        }
                    } else {
                        PointTestResult {
                            sensor_code: p.sensor_code.clone(),
                            value: None,
                            unit: p.unit.clone(),
                            elapsed_ms: elapsed,
                            error: Some("无返回".into()),
                        }
                    }
                })
                .collect()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            device
                .data_points
                .iter()
                .map(|p| PointTestResult {
                    sensor_code: p.sensor_code.clone(),
                    value: None,
                    unit: p.unit.clone(),
                    elapsed_ms: elapsed,
                    error: Some(format!("{:#}", e)),
                })
                .collect()
        }
    }
}

/// 构造手动测试设备
pub fn build_manual_device(
    protocol: ProtocolType,
    bus: BusType,
    slave_addr: u8,
    register: u16,
    count: u16,
    func_code: &str,
    data_type: &str,
    byte_order: &str,
) -> Device {
    let point = DataPoint {
        sensor_code: "manual_test".into(),
        sensor_name: "手动测试".into(),
        register_address: register,
        register_count: count,
        data_type: data_type.into(),
        byte_order: byte_order.into(),
        func_code: func_code.into(),
        coefficient: 1.0,
        offset: 0.0,
        unit: "raw".into(),
        extra_params: None,
    };

    Device {
        id: 9999,
        code: "MANUAL".into(),
        name: "手动测试设备".into(),
        protocol,
        slave_addr,
        bus,
        collect_interval_sec: None,
        data_points: vec![point],
        alarm_config: None,
        online: true,
        consecutive_failures: 0,
        last_success_at: None,
        last_error_at: None,
        last_error_msg: String::new(),
    }
}
