<template>
  <div class="detail-page" v-loading="loading">
    <!-- 设备头部信息 -->
    <div class="detail-hero">
      <div class="hero-top">
        <div class="hero-info">
          <h1 class="hero-name">{{ device?.name }}</h1>
          <div class="hero-meta">
            <span class="meta-item">
              <svg viewBox="0 0 14 14" fill="none" class="meta-icon"><rect x="2" y="2" width="10" height="10" rx="2" stroke="currentColor" stroke-width="1.2"/></svg>
              {{ device?.code }}
            </span>
            <span class="meta-sep">·</span>
            <span class="meta-item">{{ device?.modelName }}</span>
            <span class="meta-sep">·</span>
            <span class="meta-item">{{ device?.protocolName }}</span>
          </div>
        </div>
        <div class="hero-status">
          <span :class="['status-pill', device?.status]">
            <span class="pill-dot"></span>
            {{ statusText }}
          </span>
        </div>
      </div>
      <div class="hero-stats" v-if="device">
        <div class="stat-item">
          <span class="stat-label">采集器</span>
          <span class="stat-value">{{ device.collectorName }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">串口</span>
          <span class="stat-value">{{ device.serialPortName }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">从站地址</span>
          <span class="stat-value">{{ device.slaveAddr }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">采集间隔</span>
          <span class="stat-value">{{ device.collectIntervalSec ?? '-' }}s</span>
        </div>
      </div>
    </div>

    <!-- 实时读数卡片网格 -->
    <div class="content-block" v-if="readings.length > 0">
      <h3 class="section-heading">实时采集数据</h3>
      <div class="readings-grid">
        <div v-for="item in readings" :key="item.sensorCode" class="reading-card">
          <!-- 数值类型 → 数值卡片 -->
          <template v-if="isNumeric(item.dataType)">
            <div class="rc-header">
              <span class="rc-label">{{ item.sensorName || item.sensorCode }}</span>
              <button class="rc-action" @click="openTrend(item)" title="查看趋势">趋势</button>
            </div>
            <div class="rc-body">
              <span class="rc-value">
                {{ formatValue(item.value) }}
              </span>
              <span v-if="item.unit" class="rc-unit">{{ item.unit }}</span>
            </div>
            <div class="rc-footer">
              <span :class="['quality-dot', item.quality === 'good' ? 'good' : 'bad']"></span>
              <span class="quality-text">{{ item.quality === 'good' ? '数据正常' : '数据异常' }}</span>
            </div>
          </template>

          <!-- bool 类型 → 状态灯 -->
          <template v-else-if="item.dataType === 'bool'">
            <div :class="['bool-card', item.value > 0 ? 'on' : 'off']">
              <div class="bool-dot-wrapper">
                <div :class="['bool-dot', item.value > 0 ? 'on' : 'off']"></div>
              </div>
              <p class="bool-label">{{ item.sensorName || item.sensorCode }}</p>
              <p :class="['bool-text', item.value > 0 ? 'on' : 'off']">
                {{ item.value > 0 ? 'ON' : 'OFF' }}
              </p>
              <div v-if="isWritable(item)" class="bool-switch">
                <el-switch
                  :model-value="item.value > 0"
                  size="small"
                  @change="(val: boolean) => handleSwitch(item, val)"
                />
              </div>
            </div>
          </template>

          <!-- 其他类型默认展示 -->
          <template v-else>
            <div class="rc-header">
              <span class="rc-label">{{ item.sensorName || item.sensorCode }}</span>
            </div>
            <div class="rc-body">
              <span class="rc-value text-value">{{ item.value ?? '-' }}</span>
            </div>
          </template>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-if="!loading && readings.length === 0" class="content-block">
      <div class="empty-state">
        <svg viewBox="0 0 48 48" fill="none" class="empty-icon">
          <rect x="6" y="8" width="36" height="32" rx="3" stroke="currentColor" stroke-width="1.5"/>
          <path d="M18 22l4 4 8-8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        <p>暂无采集数据，请检查设备连接</p>
      </div>
    </div>

    <!-- 告警阈值设置 -->
    <div class="content-block">
      <div class="block-header">
        <h3 class="section-heading">告警阈值设置</h3>
        <el-button type="primary" size="small" @click="addAlarmDialog = true">
          <svg viewBox="0 0 16 16" fill="none" style="width:14px;height:14px;margin-right:4px;">
            <path d="M8 3v10M3 8h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          添加告警规则
        </el-button>
      </div>

      <el-table :data="alarmConfigs" size="small" v-if="alarmConfigs.length > 0">
        <el-table-column prop="sensorCode" label="采集点" width="160"/>
        <el-table-column prop="alarmType" label="告警类型" width="110">
          <template #default="{row}">{{ alarmTypeLabels[row.alarmType] || row.alarmType }}</template>
        </el-table-column>
        <el-table-column label="参数" min-width="200">
          <template #default="{row}">{{ formatAlarmParams(row.alarmType, row.params) }}</template>
        </el-table-column>
        <el-table-column prop="alarmLevel" label="等级" width="80" align="center">
          <template #default="{row}">
            <span :class="['level-badge', row.alarmLevel]">
              {{ row.alarmLevel === 'critical' ? '严重' : row.alarmLevel === 'warning' ? '警告' : '提示' }}
            </span>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="120" align="center">
          <template #default="{row}">
            <el-button size="small" link type="primary" @click="editAlarm(row)">编辑</el-button>
            <el-button size="small" link type="danger" @click="deleteAlarm(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="empty-state" v-else>
        <p style="font-size:13px;color:var(--color-gray-400)">暂无告警规则，点击上方按钮添加</p>
      </div>

      <!-- 添加/编辑告警弹窗 -->
      <el-dialog :title="editingAlarm ? '编辑告警规则' : '添加告警规则'" v-model="addAlarmDialog" width="500px" @closed="resetAlarmForm">
        <el-form :model="alarmForm" label-width="100px" size="small">
          <el-form-item label="采集点">
            <el-select v-model="alarmForm.sensorCode" placeholder="选择采集点" :disabled="!!editingAlarm" style="width:100%">
              <el-option v-for="r in readings" :key="r.sensorCode" :label="r.sensorName || r.sensorCode" :value="r.sensorCode"/>
            </el-select>
          </el-form-item>
          <el-form-item label="告警类型">
            <el-select v-model="alarmForm.alarmType" placeholder="选择告警类型" @change="onAlarmTypeChange" style="width:100%">
              <el-option v-for="item in alarmTypes" :key="item.value" :label="item.label" :value="item.value"/>
            </el-select>
          </el-form-item>

          <el-form-item label="上限值" v-if="['limit_upper','limit_both'].includes(alarmForm.alarmType)">
            <el-input-number v-model="alarmForm.params.max" controls-position="right" style="width:100%"/>
          </el-form-item>
          <el-form-item label="下限值" v-if="['limit_lower','limit_both'].includes(alarmForm.alarmType)">
            <el-input-number v-model="alarmForm.params.min" controls-position="right" style="width:100%"/>
          </el-form-item>
          <el-form-item label="回滞值" v-if="['limit_upper','limit_lower','limit_both','deviation'].includes(alarmForm.alarmType)">
            <el-input-number v-model="alarmForm.params.hysteresis" :min="0" controls-position="right" style="width:100%"/>
          </el-form-item>
          <el-form-item label="连续次数" v-if="['limit_upper','limit_lower','limit_both','deviation'].includes(alarmForm.alarmType)">
            <el-input-number v-model="alarmForm.params.delayCount" :min="1" :max="10" controls-position="right" style="width:100%"/>
          </el-form-item>
          <el-form-item label="变化率" v-if="['rate_rise','rate_fall'].includes(alarmForm.alarmType)">
            <div style="display:flex;gap:8px;align-items:center">
              <el-input-number v-model="alarmForm.params.rate" :min="0" controls-position="right" placeholder="单位/秒"/>
              <span style="font-size:12px;color:var(--color-gray-500);flex-shrink:0">时间窗口</span>
              <el-input-number v-model="alarmForm.params.windowSec" :min="10" :max="3600" controls-position="right" style="width:100px"/> s
            </div>
          </el-form-item>
          <el-form-item label="额定值" v-if="alarmForm.alarmType === 'deviation'">
            <el-input-number v-model="alarmForm.params.expected" controls-position="right" style="width:100%"/>
          </el-form-item>
          <el-form-item label="偏差%" v-if="alarmForm.alarmType === 'deviation'">
            <el-input-number v-model="alarmForm.params.percent" :min="1" :max="100" controls-position="right" style="width:100%"/>
          </el-form-item>
          <el-form-item label="触发值" v-if="alarmForm.alarmType === 'di_change'">
            <el-switch v-model="alarmForm.params.triggerOn" :active-value="1" :inactive-value="0" active-text="0→1告警" inactive-text="1→0告警"/>
          </el-form-item>
          <el-form-item label="超时(s)" v-if="alarmForm.alarmType === 'timeout'">
            <el-input-number v-model="alarmForm.params.timeoutSec" :min="5" :max="3600" controls-position="right" style="width:100%"/>
          </el-form-item>

          <el-form-item label="告警等级">
            <el-select v-model="alarmForm.alarmLevel" style="width:100%">
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

    <!-- 设备指令管理 -->
    <div class="content-block">
      <div class="block-header">
        <h3 class="section-heading">设备指令管理</h3>
        <el-button type="primary" size="small" @click="openInstrDialog()">
          <svg viewBox="0 0 16 16" fill="none" style="width:14px;height:14px;margin-right:4px;">
            <path d="M8 3v10M3 8h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          添加指令
        </el-button>
      </div>

      <el-table :data="instructions" size="small" v-if="instructions.length > 0">
        <el-table-column prop="instructionCode" label="指令编码" width="140"/>
        <el-table-column prop="instructionName" label="指令名称" width="150"/>
        <el-table-column prop="instructionType" label="类型" width="90" align="center">
          <template #default="{row}">
            <span :class="['instr-type', row.instructionType]">{{ instrTypeLabel(row.instructionType) }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="funcCode" label="功能码" width="80" align="center"/>
        <el-table-column label="寄存器地址" width="130" align="center">
          <template #default="{row}">
            <code class="addr-code">0x{{ row.registerAddress?.toString(16)?.toUpperCase() }}</code>
          </template>
        </el-table-column>
        <el-table-column prop="registerCount" label="数量" width="70" align="center"/>
        <el-table-column prop="sortOrder" label="排序" width="70" align="center"/>
        <el-table-column label="操作" width="120" align="center">
          <template #default="{row}">
            <el-button size="small" link type="primary" @click="openInstrDialog(row)">编辑</el-button>
            <el-button size="small" link type="danger" @click="deleteInstruction(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="empty-state" v-else>
        <p style="font-size:13px;color:var(--color-gray-400)">暂无配置指令</p>
      </div>

      <!-- 指令弹窗 -->
      <el-dialog :title="editingInstr ? '编辑指令' : '添加指令'" v-model="instrDialogVisible" width="560px" @closed="resetInstrForm">
        <el-form :model="instrForm" label-width="110px" size="small">
          <el-form-item label="指令编码" required>
            <el-input v-model="instrForm.instructionCode" placeholder="如: read_temp, write_relay"/>
          </el-form-item>
          <el-form-item label="指令名称" required>
            <el-input v-model="instrForm.instructionName" placeholder="如: 读取温度"/>
          </el-form-item>
          <el-form-item label="指令类型" required>
            <el-select v-model="instrForm.instructionType" style="width:100%">
              <el-option label="读取 (READ)" value="READ"/>
              <el-option label="写入 (WRITE)" value="WRITE"/>
              <el-option label="控制 (CONTROL)" value="CONTROL"/>
              <el-option label="配置 (CONFIG)" value="CONFIG"/>
            </el-select>
          </el-form-item>
          <el-form-item label="功能码">
            <el-select v-model="instrForm.funcCode" style="width:100%">
              <el-option v-for="fc in funcCodes" :key="fc.value" :label="fc.label" :value="fc.value"/>
            </el-select>
          </el-form-item>
          <el-row :gutter="12">
            <el-col :span="12">
              <el-form-item label="寄存器地址">
                <el-input-number v-model="instrForm.registerAddress" :min="0" :max="65535" style="width:100%"/>
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item label="寄存器数量">
                <el-input-number v-model="instrForm.registerCount" :min="1" :max="125" style="width:100%"/>
              </el-form-item>
            </el-col>
          </el-row>
          <el-form-item label="扩展参数(JSON)">
            <el-input v-model="instrForm.params" type="textarea" :rows="2" placeholder='如: {"dataType":"float32","byteOrder":"ABCD"}'/>
          </el-form-item>
          <el-form-item label="排序">
            <el-input-number v-model="instrForm.sortOrder" :min="0" :max="999" style="width:100%"/>
          </el-form-item>
          <el-form-item label="描述">
            <el-input v-model="instrForm.description" placeholder="指令说明" type="textarea" :rows="2"/>
          </el-form-item>
        </el-form>
        <template #footer>
          <el-button @click="instrDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="saveInstruction" :loading="savingInstr">保存</el-button>
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
import { instructionApi } from '@/api/device-instruction'

const route = useRoute()
const deviceId = Number(route.params.id)

const loading = ref(false)
const device = ref<any>(null)
const readings = ref<any[]>([])

const statusText = computed(() => {
  const map: Record<string, string> = { online: '在线', offline: '离线', alarm: '告警', disabled: '已禁用' }
  return map[device.value?.status] ?? device.value?.status ?? '-'
})

const isNumeric = (type: string) => ['float32', 'float64', 'uint16', 'int16', 'uint32', 'int32'].includes(type)
const isWritable = (item: any) => item.dataType === 'bool' && (item.access === 'RW' || item.permission === 'RW')

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
  { label: '上限告警', value: 'limit_upper' }, { label: '下限告警', value: 'limit_lower' },
  { label: '上下限告警', value: 'limit_both' }, { label: '升速率告警', value: 'rate_rise' },
  { label: '降速率告警', value: 'rate_fall' }, { label: '偏差告警', value: 'deviation' },
  { label: 'DI跳变告警', value: 'di_change' }, { label: '通信超时告警', value: 'timeout' },
  { label: '死区告警', value: 'deadband' }, { label: '自定义', value: 'custom' }
]

const alarmTypeLabels: Record<string, string> = {
  limit_upper: '上限', limit_lower: '下限', limit_both: '上下限',
  rate_rise: '升速率', rate_fall: '降速率', deviation: '偏差',
  di_change: 'DI跳变', timeout: '超时', deadband: '死区', custom: '自定义'
}

const onAlarmTypeChange = () => { alarmForm.params = {} }

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
  } finally { savingAlarm.value = false }
}

const deleteAlarm = async (row: any) => {
  await ElMessageBox.confirm('确认删除此告警规则?', '删除确认', { type: 'warning' })
  try {
    await alarmConfigApi.remove(deviceId, row.alarmType, row.sensorCode)
    ElMessage.success('已删除')
    await loadAlarmConfig()
  } catch(e) {}
}

// ─── 设备指令 ───
const instructions = ref<any[]>([])
const instrDialogVisible = ref(false)
const savingInstr = ref(false)
const editingInstr = ref<any>(null)
const instrForm = reactive({
  instructionCode: '', instructionName: '', instructionType: 'READ', funcCode: '0x03',
  registerAddress: 0, registerCount: 1, params: '', sortOrder: 0, description: ''
})

const funcCodes = [
  { label: '0x01 - 读线圈', value: '0x01' }, { label: '0x02 - 读离散输入', value: '0x02' },
  { label: '0x03 - 读保持寄存器', value: '0x03' }, { label: '0x04 - 读输入寄存器', value: '0x04' },
  { label: '0x05 - 写单线圈', value: '0x05' }, { label: '0x06 - 写单寄存器', value: '0x06' },
  { label: '0x0F - 写多线圈', value: '0x0F' }, { label: '0x10 - 写多寄存器', value: '0x10' }
]

const instrTypeLabel = (t: string) => {
  const map: Record<string, string> = { READ: '读取', WRITE: '写入', CONTROL: '控制', CONFIG: '配置' }
  return map[t] ?? t
}

const loadInstructions = async () => {
  try { const res: any = await instructionApi.list(deviceId); instructions.value = res?.data ?? [] }
  catch { instructions.value = [] }
}

const openInstrDialog = (row?: any) => {
  editingInstr.value = row ?? null
  if (row) {
    instrForm.instructionCode = row.instructionCode; instrForm.instructionName = row.instructionName
    instrForm.instructionType = row.instructionType ?? 'READ'; instrForm.funcCode = row.funcCode ?? '0x03'
    instrForm.registerAddress = row.registerAddress ?? 0; instrForm.registerCount = row.registerCount ?? 1
    instrForm.params = row.params ?? ''; instrForm.sortOrder = row.sortOrder ?? 0
    instrForm.description = row.description ?? ''
  }
  instrDialogVisible.value = true
}

const resetInstrForm = () => {
  editingInstr.value = null; instrForm.instructionCode = ''; instrForm.instructionName = ''
  instrForm.instructionType = 'READ'; instrForm.funcCode = '0x03'; instrForm.registerAddress = 0
  instrForm.registerCount = 1; instrForm.params = ''; instrForm.sortOrder = 0; instrForm.description = ''
}

const saveInstruction = async () => {
  savingInstr.value = true
  try {
    const payload = {
      instructionCode: instrForm.instructionCode, instructionName: instrForm.instructionName,
      instructionType: instrForm.instructionType, funcCode: instrForm.funcCode,
      registerAddress: instrForm.registerAddress, registerCount: instrForm.registerCount,
      params: instrForm.params || undefined, sortOrder: instrForm.sortOrder, description: instrForm.description
    }
    if (editingInstr.value) {
      await instructionApi.update(deviceId, editingInstr.value.id, payload)
      ElMessage.success('指令更新成功')
    } else {
      await instructionApi.create(deviceId, payload)
      ElMessage.success('指令创建成功')
    }
    instrDialogVisible.value = false
    await loadInstructions()
  } catch { ElMessage.error('保存失败') } finally { savingInstr.value = false }
}

const deleteInstruction = async (row: any) => {
  await ElMessageBox.confirm('确认删除此指令?', '删除确认', { type: 'warning' })
  try { await instructionApi.remove(deviceId, row.id); ElMessage.success('已删除'); await loadInstructions() }
  catch {}
}

onMounted(async () => {
  loading.value = true
  try {
    const [deviceRes, readingsRes]: [any, any] = await Promise.all([
      deviceApi.getById(deviceId), readingApi.latest(deviceId)
    ])
    device.value = deviceRes?.data ?? {}
    readings.value = readingsRes?.data ?? []
    await loadAlarmConfig()
    await loadInstructions()
  } catch {
    device.value = null; readings.value = []
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
.detail-page {
  animation: page-in 400ms var(--ease-out-expo);
}

@keyframes page-in {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}

/* ═══ Hero Header ═══ */
.detail-hero {
  background: var(--surface-primary);
  border: 1px solid var(--border-light);
  border-radius: var(--radius-lg);
  padding: var(--space-6);
  box-shadow: var(--shadow-xs);
  margin-bottom: var(--space-4);
}

.hero-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--space-4);
}

.hero-name {
  font-size: var(--text-xl);
  font-weight: var(--font-weight-bold);
  color: var(--color-gray-800);
  margin: 0 0 var(--space-2);
}

.hero-meta {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-sm);
  color: var(--color-gray-500);
}

.meta-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.meta-icon {
  width: 14px;
  height: 14px;
}

.meta-sep {
  color: var(--color-gray-300);
}

.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  font-weight: var(--font-weight-semibold);
}

.status-pill.online {
  background: var(--color-success-50);
  color: var(--color-success-600);
}

.status-pill.offline {
  background: var(--color-gray-100);
  color: var(--color-gray-600);
}

.status-pill.alarm {
  background: var(--color-danger-50);
  color: var(--color-danger-600);
}

.status-pill.disabled {
  background: var(--color-warning-50);
  color: var(--color-warning-600);
}

.pill-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
}

.status-pill.online .pill-dot { box-shadow: 0 0 4px var(--color-success-400); }
.status-pill.alarm .pill-dot { animation: dot-pulse 1.5s ease-in-out infinite; }

@keyframes dot-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.hero-stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--space-3);
  padding-top: var(--space-4);
  border-top: 1px solid var(--border-light);
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.stat-label {
  font-size: 11px;
  color: var(--color-gray-400);
  font-weight: var(--font-weight-medium);
}

.stat-value {
  font-size: var(--text-sm);
  color: var(--color-gray-700);
  font-weight: var(--font-weight-semibold);
}

/* ═══ Block Header ═══ */
.block-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.block-header .section-heading {
  margin: 0;
}

/* ═══ Reading Cards Grid ═══ */
.readings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: var(--space-3);
}

.reading-card {
  background: var(--surface-secondary);
  border: 1px solid var(--border-light);
  border-radius: var(--radius-md);
  padding: var(--space-4);
  transition: all var(--duration-normal) var(--ease-out-quart);
}

.reading-card:hover {
  border-color: var(--color-brand-300);
  box-shadow: var(--shadow-sm);
  transform: translateY(-1px);
}

.rc-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-2);
}

.rc-label {
  font-size: 11px;
  color: var(--color-gray-500);
  font-weight: var(--font-weight-medium);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.rc-action {
  font-size: 11px;
  color: var(--color-gray-400);
  background: none;
  border: none;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-family: var(--font-sans);
  transition: all var(--duration-fast) var(--ease-out-quart);
}

.rc-action:hover {
  color: var(--color-brand-500);
  background: var(--color-brand-50);
}

.rc-body {
  display: flex;
  align-items: baseline;
  gap: 4px;
  margin-bottom: var(--space-2);
}

.rc-value {
  font-size: var(--text-2xl);
  font-weight: var(--font-weight-bold);
  color: var(--color-gray-800);
  font-variant-numeric: tabular-nums;
  font-family: var(--font-mono);
  line-height: 1.2;
}

.rc-value.text-value {
  font-size: var(--text-base);
  font-family: var(--font-sans);
  color: var(--color-gray-600);
}

.rc-unit {
  font-size: var(--text-xs);
  color: var(--color-gray-500);
  font-weight: var(--font-weight-medium);
}

.rc-footer {
  display: flex;
  align-items: center;
  gap: 4px;
}

.quality-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.quality-dot.good { background: var(--color-success-500); }
.quality-dot.bad { background: var(--color-danger-500); }

.quality-text {
  font-size: 10px;
  color: var(--color-gray-400);
  font-weight: var(--font-weight-medium);
}

/* ═══ Bool Cards ═══ */
.bool-card {
  text-align: center;
  padding: var(--space-2) 0;
}

.bool-dot-wrapper {
  display: flex;
  justify-content: center;
  margin-bottom: var(--space-2);
}

.bool-dot {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  transition: all var(--duration-normal) var(--ease-out-quart);
}

.bool-dot.on {
  background: var(--color-success-500);
  box-shadow: 0 0 12px var(--color-success-400);
}

.bool-dot.off {
  background: var(--color-gray-300);
}

.bool-label {
  font-size: 11px;
  color: var(--color-gray-500);
  margin: 0;
  font-weight: var(--font-weight-medium);
}

.bool-text {
  font-size: var(--text-sm);
  font-weight: var(--font-weight-bold);
  margin: 4px 0;
}

.bool-text.on { color: var(--color-success-600); }
.bool-text.off { color: var(--color-gray-500); }

.bool-switch {
  display: flex;
  justify-content: center;
  margin-top: var(--space-2);
}

/* ═══ Level Badge ═══ */
.level-badge {
  font-size: 11px;
  font-weight: var(--font-weight-semibold);
  padding: 2px 10px;
  border-radius: var(--radius-full);
}

.level-badge.critical { color: var(--color-danger-600); background: var(--color-danger-50); }
.level-badge.warning { color: var(--color-warning-600); background: var(--color-warning-50); }
.level-badge.info { color: var(--color-info-600); background: var(--color-info-50); }

/* ═══ Instruction Type Tags ═══ */
.instr-type {
  font-size: 11px;
  font-weight: var(--font-weight-semibold);
  padding: 2px 8px;
  border-radius: var(--radius-sm);
}

.instr-type.READ { color: var(--color-info-600); background: var(--color-info-50); }
.instr-type.WRITE { color: var(--color-warning-600); background: var(--color-warning-50); }
.instr-type.CONTROL { color: var(--color-danger-600); background: var(--color-danger-50); }
.instr-type.CONFIG { color: var(--color-brand-600); background: var(--color-brand-50); }

/* ═══ Address Code ═══ */
.addr-code {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--color-brand-600);
  background: var(--color-brand-50);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
}

/* ═══ Empty State ═══ */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--space-8) 0;
}

.empty-icon {
  width: 48px;
  height: 48px;
  margin-bottom: var(--space-3);
  color: var(--color-gray-300);
}

/* ═══ Responsive ═══ */
@media (max-width: 768px) {
  .hero-stats {
    grid-template-columns: repeat(2, 1fr);
  }

  .readings-grid {
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  }
}
</style>
