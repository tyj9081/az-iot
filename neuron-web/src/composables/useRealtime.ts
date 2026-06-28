import { ref, shallowRef } from 'vue'

export interface RealtimePoint {
  deviceId: number
  sensorCode: string
  value: number
  unit: string
  timestamp: number
}

interface RealtimeMessage {
  type: string
  data: RealtimePoint[]
  timestamp: string
}

/**
 * WebSocket 实时数据 composable。
 * 全局单例：同一页面多次调用返回同一实例。
 */
let singleton: ReturnType<typeof createRealtime> | null = null

function createRealtime() {
  const latestData = shallowRef<Map<number, Map<string, RealtimePoint>>>(new Map())
  const connected = ref(false)
  const lastDataTimestamps = new Map<number, number>()

  let ws: WebSocket | null = null
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null
  let manualClose = false

  function connect() {
    if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) return

    manualClose = false
    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
    const url = `${protocol}//${location.host}/ws/realtime`

    ws = new WebSocket(url)

    ws.onopen = () => {
      connected.value = true
    }

    ws.onmessage = (event) => {
      try {
        const msg: RealtimeMessage = JSON.parse(event.data)
        if (msg.type === 'realtime' && Array.isArray(msg.data)) {
          const newMap = new Map(latestData.value)
          const now = Date.now()

          for (const point of msg.data) {
            if (!newMap.has(point.deviceId)) {
              newMap.set(point.deviceId, new Map())
            }
            const deviceMap = newMap.get(point.deviceId)!
            deviceMap.set(point.sensorCode, point)
            lastDataTimestamps.set(point.deviceId, now)
          }

          latestData.value = newMap
        }
      } catch {
        // 忽略解析错误
      }
    }

    ws.onclose = () => {
      connected.value = false
      ws = null
      if (!manualClose) {
        // 5 秒后自动重连
        reconnectTimer = setTimeout(connect, 5000)
      }
    }

    ws.onerror = () => {
      // onclose 会自动触发，不重复处理
    }
  }

  function disconnect() {
    manualClose = true
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      reconnectTimer = null
    }
    ws?.close()
    ws = null
    connected.value = false
  }

  /**
   * 判断设备是否在线：最近 30 秒内有实时数据
   */
  function isDeviceOnline(deviceId: number): boolean {
    const ts = lastDataTimestamps.get(deviceId)
    if (!ts) return false
    return Date.now() - ts < 30_000
  }

  /**
   * 获取通过 WebSocket 判定在线的设备数
   */
  function getOnlineDeviceCount(): number {
    const now = Date.now()
    let count = 0
    lastDataTimestamps.forEach((ts) => {
      if (now - ts < 30_000) count++
    })
    return count
  }

  function getDeviceLatestSensors(deviceId: number): RealtimePoint[] {
    const deviceMap = latestData.value.get(deviceId)
    if (!deviceMap) return []
    return Array.from(deviceMap.values())
  }

  return { latestData, connected, connect, disconnect, isDeviceOnline, getOnlineDeviceCount, getDeviceLatestSensors }
}

export function useRealtime() {
  if (!singleton) {
    singleton = createRealtime()
  }
  return singleton
}
