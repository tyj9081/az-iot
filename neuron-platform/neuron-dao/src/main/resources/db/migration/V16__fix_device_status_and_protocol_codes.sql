UPDATE dev_device
SET status = CASE status
    WHEN '0' THEN 'offline'
    WHEN '1' THEN 'online'
    WHEN '2' THEN 'alarm'
    WHEN '3' THEN 'disabled'
    ELSE status
END;

ALTER TABLE dev_device
    MODIFY COLUMN status VARCHAR(16) NOT NULL DEFAULT 'offline'
    COMMENT 'offline/online/alarm/disabled';

INSERT INTO dev_protocol (code, name, bus_type, is_enabled)
VALUES
('MODBUS_RTU', 'Modbus RTU', 'serial', 1),
('MODBUS_TCP', 'Modbus TCP', 'tcp', 1),
('DL_T645_2007', 'DL/T645-2007', 'serial', 1),
('DL_T645_1997', 'DL/T645-1997', 'serial', 1),
('IEC_60870_5_101', 'IEC 60870-5-101', 'serial', 1),
('CAN_BUS', 'CAN Bus', 'serial', 1),
('PROFIBUS_DP', 'PROFIBUS DP', 'serial', 1),
('IEC_60870_5_104', 'IEC 60870-5-104', 'tcp', 1),
('DNP3', 'DNP3', 'tcp', 1),
('OPC_UA', 'OPC UA', 'tcp', 1),
('BACNET_IP', 'BACnet/IP', 'tcp', 1),
('S7_COMM', 'S7 Communication', 'tcp', 1),
('FINS_TCP', 'FINS TCP', 'tcp', 1),
('ETHERNET_IP', 'EtherNet/IP', 'tcp', 1),
('MITSUBISHI_MC', 'Mitsubishi MC', 'tcp', 1),
('MQTT', 'MQTT', 'tcp', 1),
('SNMP_V2C', 'SNMP v2c', 'tcp', 1),
('HTTP_JSON', 'HTTP JSON', 'tcp', 1)
ON DUPLICATE KEY UPDATE
    name = VALUES(name),
    bus_type = VALUES(bus_type),
    is_enabled = VALUES(is_enabled);

UPDATE dev_protocol
SET is_enabled = 0
WHERE code IN ('DL_T645', 'SNMP', 'AT_COMMAND');
