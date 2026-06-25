package com.aziot.common.enums;

import lombok.Getter;

/**
 * 工业通信协议枚举 — 与采集端 Rust ProtocolType 严格一一对应。
 *
 * <p>新增协议流程:
 * <ol>
 *   <li>在此枚举添加常量</li>
 *   <li>在 collector-model ProtocolType 添加对应变体</li>
 *   <li>在 collector-driver 实现对应 ProtocolDriver</li>
 *   <li>在本 Flyway 迁移中追加 INSERT</li>
 * </ol>
 */
@Getter
public enum ProtocolType {

    // ── 串口协议 (Serial Bus) ──
    MODBUS_RTU("MODBUS_RTU", "Modbus RTU", "serial"),
    DL_T645_2007("DL_T645_2007", "DL/T645-2007", "serial"),
    DL_T645_1997("DL_T645_1997", "DL/T645-1997", "serial"),
    IEC_60870_5_101("IEC_60870_5_101", "IEC 60870-5-101", "serial"),
    CAN_BUS("CAN_BUS", "CAN Bus", "serial"),
    PROFIBUS_DP("PROFIBUS_DP", "PROFIBUS DP", "serial"),

    // ── TCP/IP 协议 ──
    MODBUS_TCP("MODBUS_TCP", "Modbus TCP", "tcp"),
    IEC_60870_5_104("IEC_60870_5_104", "IEC 60870-5-104", "tcp"),
    DNP3("DNP3", "DNP3", "tcp"),
    OPC_UA("OPC_UA", "OPC UA", "tcp"),
    BACNET_IP("BACNET_IP", "BACnet/IP", "tcp"),
    S7_COMM("S7_COMM", "S7 Communication", "tcp"),
    FINS_TCP("FINS_TCP", "FINS TCP", "tcp"),
    ETHERNET_IP("ETHERNET_IP", "EtherNet/IP", "tcp"),
    MITSUBISHI_MC("MITSUBISHI_MC", "Mitsubishi MC", "tcp"),

    // ── 上层协议 ──
    MQTT("MQTT", "MQTT", "tcp"),
    SNMP_V2C("SNMP_V2C", "SNMP v2c", "tcp"),
    HTTP_JSON("HTTP_JSON", "HTTP JSON", "tcp");

    /** 协议编码 — 与 Rust serde rename 以及数据库 dev_protocol.code 一致 */
    private final String code;
    /** 协议中文显示名称 */
    private final String displayName;
    /** 传输方式: serial / tcp */
    private final String busType;

    ProtocolType(String code, String displayName, String busType) {
        this.code = code;
        this.displayName = displayName;
        this.busType = busType;
    }

    /** 根据 code 查找枚举，找不到抛异常 */
    public static ProtocolType fromCode(String code) {
        for (ProtocolType pt : values()) {
            if (pt.code.equals(code)) {
                return pt;
            }
        }
        throw new IllegalArgumentException("Unknown protocol code: " + code);
    }

    /** 校验 code 是否为有效协议 */
    public static boolean isValidCode(String code) {
        for (ProtocolType pt : values()) {
            if (pt.code.equals(code)) {
                return true;
            }
        }
        return false;
    }
}
