use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProtocolType {
    #[serde(rename = "MODBUS_RTU")]
    ModbusRTU,
    #[serde(rename = "MODBUS_TCP")]
    ModbusTCP,
    #[serde(rename = "DL_T645")]
    DL645,
    #[serde(rename = "AT_COMMAND")]
    ATCommand,
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
    pub baud: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: String,
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
        let json = r#"{"id":1,"code":"D","name":"x","protocol":"DL_T645","slave_addr":1,"bus":{"serial":{"port_name":"COM5","bus_param":{"baud":9600,"data_bits":8,"stop_bits":1,"parity":"none"}}},"collect_interval_sec":null,"data_points":[]}"#;
        let device: Device = serde_json::from_str(json).unwrap();
        assert_eq!(device.protocol, ProtocolType::DL645);
    }

    #[test]
    fn test_deserialize_latest_reading() {
        let json = r#"{"device_id":1,"sensor_code":"voltage_a","value":220.5,"unit":"V","read_at":"2026-06-26T00:00:00"}"#;
        let reading: LatestReading = serde_json::from_str(json).unwrap();
        assert_eq!(reading.value, 220.5);
        assert_eq!(reading.sensor_code, "voltage_a");
    }
}
