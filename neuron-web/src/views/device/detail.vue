<template>
  <div class="detail-page" v-loading="loading">
    <!-- 设备头部信息 -->
    <div class="detail-header">
      <div class="header-main">
        <h2 class="header-name">{{ device?.name }}</h2>
        <span class="header-info">{{ device?.code }} &middot; {{ device?.modelName }} &middot; {{ device?.protocolName }}</span>
        <el-tag :type="statusType" size="small">{{ statusText }}</el-tag>
      </div>
    </div>

    <!-- 点表驱动渲染器 -->
    <div class="readings-grid">
      <div v-for="item in readings" :key="item.sensorCode" class="reading-card">
        <!-- 数值类型 → 数值卡片 -->
        <template v-if="isNumeric(item.dataType)">
          <p class="card-label">{{ item.sensorName || item.sensorCode }}</p>
          <p class="card-value">
            {{ formatValue(item.value) }}
            <span v-if="item.unit" class="card-unit">{{ item.unit }}</span>
          </p>
          <span class="card-trend" @click="openTrend(item)">趋势</span>
        </template>

        <!-- bool 类型 → 状态灯 -->
        <template v-else-if="item.dataType === 'bool'">
          <div :class="['status-dot', item.value > 0 ? 'on' : 'off']"></div>
          <p class="card-label">{{ item.sensorName || item.sensorCode }}</p>
          <p :class="['status-text', item.value > 0 ? 'on' : 'off']">
            {{ item.value > 0 ? 'ON' : 'OFF' }}
          </p>
          <!-- bool + RW → 控制开关 -->
          <div v-if="isWritable(item)" class="switch-row">
            <el-switch
              :model-value="item.value > 0"
              size="small"
              @change="(val: boolean) => handleSwitch(item, val)"
            />
          </div>
        </template>

        <!-- 其他类型默认展示 -->
        <template v-else>
          <p class="card-label">{{ item.sensorName || item.sensorCode }}</p>
          <p class="card-value text-value">{{ item.value ?? '-' }}</p>
        </template>
      </div>
    </div>

    <!-- 空状态 -->
    <el-empty v-if="!loading && readings.length === 0" description="暂无采集数据" />

    <!-- 告警阈值设置 -->
    <div class="alarm-section">
      <h3 style="font-size:14px;font-weight:500;margin-bottom:12px;">告警阈值设置</h3>
      <div class="alarm-grid">
        <div v-for="item in readings" :key="'alarm-' + item.sensorCode" class="alarm-row">
          <span class="alarm-sensor">{{ item.sensorName || item.sensorCode }}</span>
          <span class="alarm-unit" v-if="item.unit">{{ item.unit }}</span>
          <el-input-number v-model="alarmForm[item.sensorCode].minValue"
            :disabled="alarmForm[item.sensorCode].alarmEnabled !== 1"
            controls-position="right" size="small" placeholder="下限" style="width:130px"/>
          <span style="color:#ccc;font-size:12px;">~</span>
          <el-input-number v-model="alarmForm[item.sensorCode].maxValue"
            :disabled="alarmForm[item.sensorCode].alarmEnabled !== 1"
            controls-position="right" size="small" placeholder="上限" style="width:130px"/>
          <el-switch v-model="alarmForm[item.sensorCode].alarmEnabled"
            :active-value="1" :inactive-value="0" size="small"/>
          <el-select v-model="alarmForm[item.sensorCode].alarmLevel" size="small" style="width:90px">
            <el-option label="提示" value="info"/>
            <el-option label="警告" value="warning"/>
            <el-option label="严重" value="critical"/>
          </el-select>
          <el-button size="small" type="primary" @click="saveAlarm(item.sensorCode)">保存</el-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { readingApi } from '@/api/reading'
import { deviceApi } from '@/api/device'
import { alarmConfigApi } from '@/api/alarm-config'

const route = useRoute()
const deviceId = Number(route.params.id)

const loading = ref(false)
const device = ref<any>(null)
const readings = ref<any[]>([])

const statusType = computed(() => {
  const s = device.value?.status
  if (s === 'online') return 'success'
  if (s === 'offline') return 'info'
  if (s === 'alarm') return 'danger'
  if (s === 'disabled') return 'warning'
  return 'info'
})

const statusText = computed(() => {
  const map: Record<string, string> = {
    online: '在线',
    offline: '离线',
    alarm: '告警',
    disabled: '已禁用'
  }
  return map[device.value?.status] ?? device.value?.status ?? '-'
})

const isNumeric = (type: string) =>
  ['float32', 'float64', 'uint16', 'int16', 'uint32', 'int32'].includes(type)

const isWritable = (item: any) =>
  item.dataType === 'bool' && (item.access === 'RW' || item.permission === 'RW')

const formatValue = (v: number) => {
  if (v == null) return '-'
  return Number(v).toFixed(3)
}

function openTrend(item: any) {
  ElMessage.info(`跳转至 ${item.sensorName || item.sensorCode} 趋势图（待开发）`)
}

function handleSwitch(item: any, val: boolean) {
  ElMessage.info(`写入 ${item.sensorCode} = ${val ? 1 : 0}（待对接 MQTT 写指令）`)
}

const alarmForm = ref<Record<string, any>>({})

const loadAlarmConfig = async () => {
  try {
    const res = await alarmConfigApi.list(deviceId) as any
    const configs = res?.data || []
    const map: Record<string, any> = {}
    readings.value.forEach((r: any) => {
      const existing = configs.find((c: any) => c.sensorCode === r.sensorCode)
      map[r.sensorCode] = existing || { alarmEnabled: 0, minValue: null, maxValue: null, hysteresis: 0, delayCount: 1, alarmLevel: 'warning' }
    })
    alarmForm.value = map
  } catch(e) {}
}

const saveAlarm = async (sensorCode: string) => {
  try {
    await alarmConfigApi.save(deviceId, sensorCode, alarmForm.value[sensorCode])
    ElMessage.success('告警配置已保存，即将下发至采集端')
  } catch(e) {
    ElMessage.error('保存失败')
  }
}

onMounted(async () => {
  loading.value = true
  try {
    const [deviceRes, readingsRes]: [any, any] = await Promise.all([
      deviceApi.getById(deviceId),
      readingApi.latest(deviceId)
    ])
    device.value = deviceRes ?? deviceRes?.data ?? {}
    readings.value = readingsRes?.data ?? readingsRes ?? []
    await loadAlarmConfig()
  } catch {
    device.value = null
    readings.value = []
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
.detail-page {
  padding: 0;
}

.detail-header {
  background: #fff;
  border: 0.5px solid #eee;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 16px;
}

.header-main {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.header-name {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
  color: #333;
}

.header-info {
  font-size: 13px;
  color: #888;
}

.readings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
}

.reading-card {
  background: #fff;
  border-radius: 8px;
  border: 0.5px solid #eee;
  padding: 16px;
  position: relative;
}

.card-label {
  font-size: 12px;
  color: #888;
  margin: 0;
}

.card-value {
  font-size: 24px;
  font-weight: 500;
  margin: 6px 0 0;
  color: #333;
}

.card-value.text-value {
  font-size: 16px;
  color: #666;
}

.card-unit {
  font-size: 13px;
  font-weight: 400;
  color: #888;
}

.card-trend {
  position: absolute;
  top: 16px;
  right: 16px;
  font-size: 11px;
  color: #ccc;
  cursor: pointer;
}

.card-trend:hover {
  color: #534AB7;
}

.status-dot {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  margin: 0 auto 8px;
}

.status-dot.on {
  background: #1D9E75;
}

.status-dot.off {
  background: #888;
}

.status-text {
  text-align: center;
  font-size: 12px;
  margin: 4px 0 0;
}

.status-text.on {
  color: #0F6E56;
}

.status-text.off {
  color: #888;
}

.switch-row {
  text-align: center;
  margin-top: 8px;
}

.alarm-section {
  background: #fff;
  border: 0.5px solid #eee;
  border-radius: 8px;
  padding: 20px;
  margin-top: 16px;
}

.alarm-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.alarm-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.alarm-sensor {
  font-size: 13px;
  font-weight: 500;
  color: #333;
  min-width: 80px;
}

.alarm-unit {
  font-size: 12px;
  color: #888;
  min-width: 30px;
}
</style>
