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

      <!-- 新增告警按钮 -->
      <el-button type="primary" size="small" @click="addAlarmDialog = true" style="margin-bottom:12px">+ 添加告警规则</el-button>

      <!-- 已有告警列表 -->
      <el-table :data="alarmConfigs" size="small" v-if="alarmConfigs.length > 0">
        <el-table-column prop="sensorCode" label="采集点" width="160"/>
        <el-table-column prop="alarmType" label="告警类型" width="120">
          <template #default="{row}">{{ alarmTypeLabels[row.alarmType] || row.alarmType }}</template>
        </el-table-column>
        <el-table-column label="参数" min-width="200">
          <template #default="{row}">{{ formatAlarmParams(row.alarmType, row.params) }}</template>
        </el-table-column>
        <el-table-column prop="alarmLevel" label="等级" width="80">
          <template #default="{row}">
            <el-tag :type="row.alarmLevel==='critical'?'danger':row.alarmLevel==='warning'?'warning':'info'" size="small">{{ row.alarmLevel }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="100">
          <template #default="{row}">
            <el-button size="small" text type="primary" @click="editAlarm(row)">编辑</el-button>
            <el-button size="small" text type="danger" @click="deleteAlarm(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
      <el-empty v-else description="暂无告警规则" :image-size="60"/>

      <!-- 添加/编辑告警弹窗 -->
      <el-dialog :title="editingAlarm ? '编辑告警规则' : '添加告警规则'" v-model="addAlarmDialog" width="500px" @closed="resetAlarmForm">
        <el-form :model="alarmForm" label-width="100px" size="small">
          <el-form-item label="采集点">
            <el-select v-model="alarmForm.sensorCode" placeholder="选择采集点" :disabled="!!editingAlarm">
              <el-option v-for="r in readings" :key="r.sensorCode" :label="r.sensorName || r.sensorCode" :value="r.sensorCode"/>
            </el-select>
          </el-form-item>
          <el-form-item label="告警类型">
            <el-select v-model="alarmForm.alarmType" placeholder="选择告警类型" @change="onAlarmTypeChange">
              <el-option v-for="item in alarmTypes" :key="item.value" :label="item.label" :value="item.value"/>
            </el-select>
          </el-form-item>

          <!-- 动态参数表单 -->
          <el-form-item label="上限值" v-if="['limit_upper','limit_both'].includes(alarmForm.alarmType)">
            <el-input-number v-model="alarmForm.params.max" controls-position="right"/>
          </el-form-item>
          <el-form-item label="下限值" v-if="['limit_lower','limit_both'].includes(alarmForm.alarmType)">
            <el-input-number v-model="alarmForm.params.min" controls-position="right"/>
          </el-form-item>
          <el-form-item label="回滞值" v-if="['limit_upper','limit_lower','limit_both','deviation'].includes(alarmForm.alarmType)">
            <el-input-number v-model="alarmForm.params.hysteresis" :min="0" controls-position="right"/>
          </el-form-item>
          <el-form-item label="连续次数" v-if="['limit_upper','limit_lower','limit_both','deviation'].includes(alarmForm.alarmType)">
            <el-input-number v-model="alarmForm.params.delayCount" :min="1" :max="10" controls-position="right"/>
          </el-form-item>
          <el-form-item label="变化率" v-if="['rate_rise','rate_fall'].includes(alarmForm.alarmType)">
            <el-input-number v-model="alarmForm.params.rate" :min="0" controls-position="right" placeholder="单位/秒"/>
            <span style="font-size:12px;color:#888;margin-left:8px;">时间窗口</span>
            <el-input-number v-model="alarmForm.params.windowSec" :min="10" :max="3600" controls-position="right" style="width:120px;margin-left:4px"/> 秒
          </el-form-item>
          <el-form-item label="额定值" v-if="alarmForm.alarmType === 'deviation'">
            <el-input-number v-model="alarmForm.params.expected" controls-position="right"/>
          </el-form-item>
          <el-form-item label="偏差%" v-if="alarmForm.alarmType === 'deviation'">
            <el-input-number v-model="alarmForm.params.percent" :min="1" :max="100" controls-position="right"/>
          </el-form-item>
          <el-form-item label="触发值" v-if="alarmForm.alarmType === 'di_change'">
            <el-switch v-model="alarmForm.params.triggerOn" :active-value="1" :inactive-value="0" active-text="0→1告警" inactive-text="1→0告警"/>
          </el-form-item>
          <el-form-item label="超时时间(秒)" v-if="alarmForm.alarmType === 'timeout'">
            <el-input-number v-model="alarmForm.params.timeoutSec" :min="5" :max="3600" controls-position="right"/>
          </el-form-item>

          <el-form-item label="告警等级">
            <el-select v-model="alarmForm.alarmLevel">
              <el-option label="提示" value="info"/>
              <el-option label="警告" value="warning"/>
              <el-option label="严重" value="critical"/>
            </el-select>
          </el-form-item>
        </el-form>
        <template #footer>
          <el-button @click="addAlarmDialog = false">取消</el-button>
          <el-button type="primary" @click="saveAlarm" :loading="savingAlarm">保存</el-button>
        </template>
      </el-dialog>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, reactive } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
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

// ─── 告警配置 ───
const alarmConfigs = ref<any[]>([])
const addAlarmDialog = ref(false)
const savingAlarm = ref(false)
const editingAlarm = ref<any>(null)
const alarmForm = reactive({
  sensorCode: '', alarmType: 'limit_upper',
  params: {} as Record<string, any>,
  alarmLevel: 'warning'
})

const alarmTypes = [
  { label: '上限告警', value: 'limit_upper' },
  { label: '下限告警', value: 'limit_lower' },
  { label: '上下限告警', value: 'limit_both' },
  { label: '升速率告警', value: 'rate_rise' },
  { label: '降速率告警', value: 'rate_fall' },
  { label: '偏差告警', value: 'deviation' },
  { label: 'DI跳变告警', value: 'di_change' },
  { label: '通信超时告警', value: 'timeout' },
  { label: '死区告警', value: 'deadband' },
  { label: '自定义', value: 'custom' }
]

const alarmTypeLabels: Record<string, string> = {
  limit_upper: '上限', limit_lower: '下限', limit_both: '上下限',
  rate_rise: '升速率', rate_fall: '降速率', deviation: '偏差',
  di_change: 'DI跳变', timeout: '超时', deadband: '死区', custom: '自定义'
}

const onAlarmTypeChange = () => {
  alarmForm.params = {}
}

const formatAlarmParams = (type: string, params: any) => {
  if (!params) return '-'
  const p = typeof params === 'string' ? JSON.parse(params) : params
  switch(type) {
    case 'limit_upper': return `上限 ${p.max}`
    case 'limit_lower': return `下限 ${p.min}`
    case 'limit_both': return `${p.min} ~ ${p.max}`
    case 'rate_rise': return `上升速率 ${p.rate}/s`
    case 'rate_fall': return `下降速率 ${p.rate}/s`
    case 'deviation': return `额定 ${p.expected} ±${p.percent}%`
    case 'di_change': return `触发: ${p.triggerOn === 1 ? '0→1' : '1→0'}`
    case 'timeout': return `超时 ${p.timeoutSec}s`
    default: return JSON.stringify(p)
  }
}

const loadAlarmConfig = async () => {
  try {
    const res = await alarmConfigApi.list(deviceId) as any
    alarmConfigs.value = (res?.data || []).map((c: any) => ({
      ...c,
      params: typeof c.params === 'string' ? JSON.parse(c.params) : c.params
    }))
  } catch(e) {}
}

const editAlarm = (row: any) => {
  editingAlarm.value = row
  alarmForm.sensorCode = row.sensorCode
  alarmForm.alarmType = row.alarmType
  alarmForm.params = { ...(typeof row.params === 'string' ? JSON.parse(row.params) : row.params) }
  alarmForm.alarmLevel = row.alarmLevel
  addAlarmDialog.value = true
}

const resetAlarmForm = () => {
  editingAlarm.value = null
  alarmForm.sensorCode = ''
  alarmForm.alarmType = 'limit_upper'
  alarmForm.params = {}
  alarmForm.alarmLevel = 'warning'
}

const saveAlarm = async () => {
  savingAlarm.value = true
  try {
    const data: any = {
      sensorCode: alarmForm.sensorCode,
      alarmType: alarmForm.alarmType,
      alarmEnabled: 1,
      params: JSON.stringify(alarmForm.params),
      alarmLevel: alarmForm.alarmLevel
    }
    await alarmConfigApi.save(deviceId, alarmForm.alarmType, alarmForm.sensorCode, data)
    ElMessage.success('告警规则已保存，即将下发至采集端')
    addAlarmDialog.value = false
    await loadAlarmConfig()
  } catch(e) {
    ElMessage.error('保存失败')
  } finally {
    savingAlarm.value = false
  }
}

const deleteAlarm = async (row: any) => {
  await ElMessageBox.confirm('确认删除此告警规则?', '删除确认', { type: 'warning' })
  try {
    await alarmConfigApi.remove(deviceId, row.alarmType, row.sensorCode)
    ElMessage.success('已删除')
    await loadAlarmConfig()
  } catch(e) {}
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
</style>
