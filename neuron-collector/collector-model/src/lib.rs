use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProtocolType {
    // === Serial Bus Protocols ===
    #[serde(rename = "MODBUS_RTU")]
    ModbusRTU,
    #[serde(rename = "DL_T645_2007")]
    DL645_2007,
    #[serde(rename = "DL_T645_1997")]
    DL645_1997,
    #[serde(rename = "IEC_60870_5_101")]
    IEC101,
    #[serde(rename = "CAN_BUS")]
    CanBus,
    #[serde(rename = "PROFIBUS_DP")]
    ProfibusDP,

    // === TCP/IP Protocols ===
    #[serde(rename = "MODBUS_TCP")]
    ModbusTCP,
    #[serde(rename = "IEC_60870_5_104")]
    IEC104,
    #[serde(rename = "DNP3")]
    DNP3,
    #[serde(rename = "OPC_UA")]
    OpcUa,
    #[serde(rename = "BACNET_IP")]
    BacnetIP,
    #[serde(rename = "S7_COMM")]
    S7Comm,
    #[serde(rename = "FINS_TCP")]
    FinsTcp,
    #[serde(rename = "ETHERNET_IP")]
    EthernetIP,
    #[serde(rename = "MITSUBISHI_MC")]
    MitsubishiMC,

    // === Message / API Protocols ===
    #[serde(rename = "MQTT")]
    Mqtt,
    #[serde(rename = "SNMP_V2C")]
    SnmpV2c,
    #[serde(rename = "HTTP_JSON")]
    HttpJson,
}

impl ProtocolType {
    /// 协议使用串口通信
    pub fn is_serial(&self) -> bool {
        matches!(self, Self::ModbusRTU | Self::DL645_2007 | Self::DL645_1997
            | Self::IEC101 | Self::CanBus | Self::ProfibusDP)
    }

    /// 协议使用 TCP/IP 通信
    pub fn is_tcp(&self) -> bool {
        matches!(self, Self::ModbusTCP | Self::IEC104 | Self::DNP3 | Self::OpcUa
            | Self::BacnetIP | Self::S7Comm | Self::FinsTcp | Self::EthernetIP
            | Self::MitsubishiMC)
    }

    /// 协议使用 MQTT/SNMP/HTTP 等上层协议
    pub fn is_app_layer(&self) -> bool {
        matches!(self, Self::Mqtt | Self::SnmpV2c | Self::HttpJson)
    }

    /// 协议对应的 serde 标识码
    pub fn code(&self) -> &'static str {
        match self {
            Self::ModbusRTU => "MODBUS_RTU",
            Self::ModbusTCP => "MODBUS_TCP",
            Self::DL645_2007 => "DL_T645_2007",
            Self::DL645_1997 => "DL_T645_1997",
            Self::IEC104 => "IEC_60870_5_104",
            Self::IEC101 => "IEC_60870_5_101",
            Self::DNP3 => "DNP3",
            Self::OpcUa => "OPC_UA",
            Self::Mqtt => "MQTT",
            Self::BacnetIP => "BACNET_IP",
            Self::SnmpV2c => "SNMP_V2C",
            Self::HttpJson => "HTTP_JSON",
            Self::CanBus => "CAN_BUS",
            Self::S7Comm => "S7_COMM",
            Self::ProfibusDP => "PROFIBUS_DP",
            Self::EthernetIP => "ETHERNET_IP",
            Self::FinsTcp => "FINS_TCP",
            Self::MitsubishiMC => "MITSUBISHI_MC",
        }
    }

    /// 协议的中文名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ModbusRTU => "Modbus RTU",
            Self::ModbusTCP => "Modbus TCP",
            Self::DL645_2007 => "DL/T645-2007",
            Self::DL645_1997 => "DL/T645-1997",
            Self::IEC104 => "IEC 60870-5-104",
            Self::IEC101 => "IEC 60870-5-101",
            Self::DNP3 => "DNP3",
            Self::OpcUa => "OPC UA",
            Self::Mqtt => "MQTT",
            Self::BacnetIP => "BACnet/IP",
            Self::SnmpV2c => "SNMP v2c",
            Self::HttpJson => "HTTP JSON",
            Self::CanBus => "CAN Bus",
            Self::S7Comm => "S7 Communication",
            Self::ProfibusDP => "PROFIBUS DP",
            Self::EthernetIP => "EtherNet/IP",
            Self::FinsTcp => "FINS TCP",
            Self::MitsubishiMC => "Mitsubishi MC",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BusType {
    #[serde(rename = "serial")]
    Serial {
        port_name: String,
        bus_param: BusParam,
    },
    #[serde(rename = "tcp")]
    Tcp {
        host: String,
        port: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusParam {
    #[serde(default = "default_baud")]
    pub baud: u32,
    #[serde(default = "default_data_bits")]
    pub data_bits: u8,
    #[serde(default = "default_stop_bits")]
    pub stop_bits: u8,
    #[serde(default = "default_parity")]
    pub parity: String,
}

fn default_baud() -> u32 { 9600 }
fn default_data_bits() -> u8 { 8 }
fn default_stop_bits() -> u8 { 1 }
fn default_parity() -> String { "none".into() }

impl Default for BusParam {
    fn default() -> Self {
        Self {
            baud: 9600,
            data_bits: 8,
            stop_bits: 1,
            parity: "none".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub sensor_code: String,
    pub sensor_name: String,
    pub register_address: u16,
    pub register_count: u16,
    pub data_type: String,
    pub byte_order: String,
    pub func_code: String,
    pub coefficient: f64,
    pub offset: f64,
    pub unit: String,
    #[serde(default)]
    pub extra_params: Option<serde_json::Value>,  // TCP 协议扩展参数: MQTT topic/json_path, OPC UA node_id 等
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAlarmThreshold {
    pub sensor_code: String,
    pub alarm_type: String,     // limit_upper, limit_lower, limit_both, rate_rise, rate_fall, deviation, di_change, timeout, deadband, custom
    pub enabled: bool,
    pub level: String,          // info, warning, critical
    pub params: serde_json::Value,  // 各类型参数 JSON
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub protocol: ProtocolType,
    pub slave_addr: u8,
    pub bus: BusType,
    pub collect_interval_sec: Option<u64>,
    pub data_points: Vec<DataPoint>,
    pub alarm_config: Option<Vec<DeviceAlarmThreshold>>,
    #[serde(skip)]
    pub online: bool,
    #[serde(skip)]
    pub consecutive_failures: u32,
    #[serde(skip)]
    pub last_success_at: Option<i64>,
    #[serde(skip)]
    pub last_error_at: Option<i64>,
    #[serde(skip)]
    pub last_error_msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDelta {
    pub version: u64,
    pub action: String,
    pub device: Option<Device>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestReading {
    pub device_id: i64,
    pub sensor_code: String,
    pub value: f64,
    pub unit: String,
    pub read_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedReading {
    pub device_id: i64,
    pub sensor_code: String,
    pub avg: f64,
    pub max: f64,
    pub min: f64,
    pub sample_count: u32,
    pub window_start: String,
    pub window_end: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_config_delta_add() {
        let json = r#"{
            "version": 1,
            "action": "add",
            "device": {
                "id": 123,
                "code": "DEV-001",
                "name": "test",
                "protocol": "MODBUS_RTU",
                "slave_addr": 1,
                "bus": {
                    "serial": {
                        "port_name": "COM5",
                        "bus_param": {"baud": 9600, "data_bits": 8, "stop_bits": 1, "parity": "none"}
                    }
                },
                "collect_interval_sec": null,
                "data_points": []
            }
        }"#;
        let delta: ConfigDelta = serde_json::from_str(json).unwrap();
        assert_eq!(delta.version, 1);
        assert_eq!(delta.action, "add");
        assert!(delta.device.is_some());
        let device = delta.device.unwrap();
        assert_eq!(device.id, 123);
        assert_eq!(device.protocol, ProtocolType::ModbusRTU);
    }

    #[test]
    fn test_deserialize_protocol_dl645() {
        let json = r#"{"id":1,"code":"D","name":"x","protocol":"DL_T645_2007","slave_addr":1,"bus":{"serial":{"port_name":"COM5","bus_param":{"baud":9600,"data_bits":8,"stop_bits":1,"parity":"none"}}},"collect_interval_sec":null,"data_points":[]}"#;
        let device: Device = serde_json::from_str(json).unwrap();
        assert_eq!(device.protocol, ProtocolType::DL645_2007);
    }

    #[test]
    fn test_deserialize_latest_reading() {
        let json = r#"{"device_id":1,"sensor_code":"voltage_a","value":220.5,"unit":"V","read_at":"2026-06-26T00:00:00"}"#;
        let reading: LatestReading = serde_json::from_str(json).unwrap();
        assert_eq!(reading.value, 220.5);
        assert_eq!(reading.sensor_code, "voltage_a");
    }
}
