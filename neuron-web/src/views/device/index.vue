<template>
  <div class="device-page page-container">
    <div class="page-toolbar">
      <div class="filter-group">
        <el-select v-model="filterSerialPortId" placeholder="串口" clearable style="width: 160px" @change="handleSearch">
          <el-option v-for="p in filterSerialPortOptions" :key="p.id" :label="p.portName" :value="p.id" />
        </el-select>
        <el-select v-model="filterModelId" placeholder="型号" clearable style="width: 180px" @change="handleSearch">
          <el-option v-for="m in modelOptions" :key="m.id" :label="m.name + ' (' + m.code + ')'" :value="m.id" />
        </el-select>
        <el-select v-model="filterStatus" placeholder="状态" clearable style="width: 120px" @change="handleSearch">
          <el-option label="全部" value="" />
          <el-option label="在线" value="online" />
          <el-option label="离线" value="offline" />
          <el-option label="告警" value="alarm" />
        </el-select>
        <el-input v-model="searchKeyword" placeholder="搜索编码/名称" clearable style="width: 220px" @keyup.enter="handleSearch">
          <template #prefix><el-icon><Search /></el-icon></template>
        </el-input>
      </div>
      <el-button type="primary" :disabled="!canCreateDevice" @click="openDialog()">新增设备</el-button>
    </div>

    <el-collapse v-model="activePanels" class="collector-panel">
      <el-collapse-item name="collector" :title="collectorPanelTitle">
        <template #title>
          <span class="panel-title-row">
            <span class="panel-dot" :class="collectorOnline ? 'online' : 'offline'"></span>
            <strong>{{ collectorInfo?.name ?? '采集器' }}</strong>
            <span class="panel-sub">{{ collectorOnline ? 'MQTT 在线' : '离线' }}</span>
            <span class="panel-meta" v-if="collectorInfo">MQTT: {{ collectorInfo.mqttClientId }} | IP: {{ collectorInfo.ipAddress }} | 采集周期: {{ collectorInfo.collectIntervalSec }}s</span>
          </span>
        </template>
        <div class="collector-config">
          <div class="config-row">
            <span class="config-label">名称</span>
            <el-input v-model="editCollectorName" size="small" style="width:160px" />
            <span class="config-label" style="margin-left:12px">编码</span>
            <el-input v-model="editCollectorCode" size="small" style="width:130px" />
            <span class="config-label" style="margin-left:12px">MQTT客户端ID</span>
            <el-input v-model="editMqttClientId" size="small" style="width:180px" />
          </div>
          <div class="config-row" style="margin-top:8px">
            <span class="config-label">采集周期(秒)</span>
            <el-input-number v-model="editInterval" :min="10" :max="86400" size="small" style="width:140px" />
            <span class="config-label" style="margin-left:12px">IP地址</span>
            <el-input v-model="editCollectorIp" size="small" style="width:140px" />
            <span class="config-label" style="margin-left:12px">描述</span>
            <el-input v-model="editCollectorDesc" size="small" style="width:160px" placeholder="备注" />
            <el-button type="primary" size="small" :loading="savingCollector" @click="saveCollectorConfig" style="margin-left:auto">保存</el-button>
          </div>
        </div>

        <el-divider content-position="left">串口列表</el-divider>
        <el-table :data="serialPortList" size="small" stripe>
          <el-table-column prop="portName" label="串口" width="100" />
          <el-table-column prop="portLabel" label="标签" width="120">
            <template #default="{ row }">{{ row.portLabel || '-' }}</template>
          </el-table-column>
          <el-table-column prop="portType" label="类型" width="100">
            <template #default="{ row }">
              <el-tag size="small" :type="row.portType === 'device' ? '' : 'info'">{{ portTypeLabel(row.portType) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="波特率" width="90">
            <template #default="{ row }">{{ busParamField(row, 'baud') }}</template>
          </el-table-column>
          <el-table-column label="数据位" width="70">
            <template #default="{ row }">{{ busParamField(row, 'data_bits') }}</template>
          </el-table-column>
          <el-table-column label="停止位" width="70">
            <template #default="{ row }">{{ busParamField(row, 'stop_bits') }}</template>
          </el-table-column>
          <el-table-column label="校验" width="70">
            <template #default="{ row }">{{ busParamField(row, 'parity') }}</template>
          </el-table-column>
          <el-table-column label="启用" width="70" align="center">
            <template #default="{ row }">
              <span :class="['inline-status', row.isActive ? 'online' : 'offline']">{{ row.isActive ? '启用' : '停用' }}</span>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="120" align="center">
            <template #default="{ row }">
              <el-button link type="primary" size="small" :disabled="row.portType !== 'device'" @click="openSerialPortEdit(row)">编辑</el-button>
            </template>
          </el-table-column>
        </el-table>
        <div style="margin-top:8px;font-size:12px;color:var(--color-gray-400)">波特率修改后通过 MQTT 实时下发到采集端生效</div>
      </el-collapse-item>
    </el-collapse>

    <el-table v-loading="loading" :data="tableData" stripe>
      <el-table-column prop="code" label="编码" min-width="120" />
      <el-table-column prop="name" label="名称" min-width="150">
        <template #default="{ row }">
          <router-link :to="'/device/' + row.id" class="device-link">{{ row.name }}</router-link>
        </template>
      </el-table-column>
      <el-table-column prop="modelName" label="型号" width="140" />
      <el-table-column prop="protocolName" label="协议" width="100" />
      <el-table-column prop="collectorName" label="采集器" width="120" />
      <el-table-column prop="serialPortName" label="串口" width="100" />
      <el-table-column prop="slaveAddr" label="从站地址" width="100" />
      <el-table-column prop="collectIntervalSec" label="采集间隔(s)" width="130">
        <template #default="{ row }">{{ row.collectIntervalSec ?? '-' }}</template>
      </el-table-column>
      <el-table-column label="状态" width="90" align="center">
        <template #default="{ row }">
          <span :class="['inline-status', row.status]">{{ statusLabel(row.status) }}</span>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="200" fixed="right" align="center">
        <template #default="{ row }">
          <el-button link type="success" size="small" @click="router.push('/device/' + row.id)">详情</el-button>
          <el-button link type="primary" size="small" @click="openDialog(row)">编辑</el-button>
          <el-button link type="danger" size="small" @click="handleDelete(row)">删除</el-button>
          <el-button link type="warning" size="small" @click="handleDisable(row)">
            {{ row.status === 'disabled' ? '启用' : '禁用' }}
          </el-button>
          <el-button link type="info" size="small" @click="handlePushConfig(row)">下发配置</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-pagination
      v-model:current-page="page" v-model:page-size="pageSize" :total="total"
      :page-sizes="[10, 20, 50]" layout="total, sizes, prev, pager, next, jumper"
      @current-change="fetchList" @size-change="fetchList"
    />

    <NextStepButton to="/dashboard" label="查看工作台" />

    <el-dialog
      v-model="dialogVisible" :title="isEdit ? '编辑设备' : '新增设备'"
      width="560px" :close-on-click-modal="false" @closed="resetForm"
    >
      <el-form ref="formRef" :model="form" :rules="rules" label-width="110px">
        <el-form-item label="编码" prop="code"><el-input v-model="form.code" placeholder="请输入编码" /></el-form-item>
        <el-form-item label="名称" prop="name"><el-input v-model="form.name" placeholder="请输入名称" /></el-form-item>
        <el-form-item label="采集器" prop="collectorId">
          <el-select v-model="form.collectorId" placeholder="选择采集器" style="width: 100%" @change="onDialogCollectorChange">
            <el-option v-for="c in dialogCollectorOptions" :key="c.id" :label="c.name + ' (' + c.code + ')'" :value="c.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="串口" prop="serialPortId">
          <el-select v-model="form.serialPortId" placeholder="选择串口" :disabled="!form.collectorId" style="width: 100%">
            <el-option v-for="p in dialogSerialPortOptions" :key="p.id" :label="p.portName" :value="p.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="型号" prop="deviceModelId">
          <el-select v-model="form.deviceModelId" placeholder="选择型号" style="width: 100%" @change="onModelChange">
            <el-option v-for="m in dialogModelOptions" :key="m.id" :label="m.name + ' (' + m.code + ')'" :value="m.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="协议">
          <el-select v-model="form.protocolId" placeholder="请先选择型号" disabled style="width: 100%">
            <el-option v-for="p in dialogProtocolOptions" :key="p.id" :label="p.name" :value="p.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="从站地址" prop="slaveAddr">
          <el-input-number v-model="form.slaveAddr" :min="1" :max="255" style="width: 100%" />
        </el-form-item>
        <el-form-item label="采集间隔(ms)" prop="collectInterval">
          <el-input v-model="form.collectInterval" placeholder="留空继承型号默认值" style="width: 100%" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitLoading" @click="handleSubmit">确定</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="serialEditVisible" title="编辑串口" width="480px" :close-on-click-modal="false">
      <el-form label-width="80px" label-position="left">
        <el-form-item label="标签">
          <el-input v-model="serialEditLabel" placeholder="如：RS485设备口" />
        </el-form-item>
        <el-form-item label="类型">
          <el-select v-model="serialEditType" style="width:100%">
            <el-option label="设备 (RS485)" value="device" />
            <el-option label="IO板卡" value="io_board" />
            <el-option label="短信猫" value="sms_modem" />
          </el-select>
        </el-form-item>
        <el-form-item label="启用">
          <el-switch v-model="serialEditActive" />
        </el-form-item>
        <el-divider content-position="left">总线参数</el-divider>
        <BusParamForm v-model="serialEditBusParam" />
      </el-form>
      <template #footer>
        <el-button @click="serialEditVisible = false">取消</el-button>
        <el-button type="primary" :loading="savingSerialPort" @click="saveSerialPortEdit">确认</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Search } from '@element-plus/icons-vue'
import type { FormInstance, FormRules } from 'element-plus'
import { deviceApi } from '@/api/device'
import { collectorApi } from '@/api/collector'
import { deviceModelApi } from '@/api/device-model'
import { protocolApi } from '@/api/protocol'
import BusParamForm from '@/components/BusParamForm.vue'
import NextStepButton from '@/components/NextStepButton.vue'

const router = useRouter()
const searchKeyword = ref('')
const filterSerialPortId = ref<number | null>(null)
const filterModelId = ref<number | null>(null)
const filterStatus = ref('')
const loading = ref(false)
const tableData = ref<any[]>([])
const page = ref(1); const pageSize = ref(10); const total = ref(0)
const modelOptions = ref<any[]>([])
const filterSerialPortOptions = ref<any[]>([])

// 采集器（单例）
const collectorInfo = ref<any>(null)
const collectorOnline = ref(false)
const activePanels = ref<string[]>(['collector'])
const editInterval = ref(600)
const editCollectorName = ref('')
const editCollectorCode = ref('')
const editCollectorDesc = ref('')
const editMqttClientId = ref('')
const editCollectorIp = ref('')
const savingCollector = ref(false)
const serialPortList = ref<any[]>([])

const collectorPanelTitle = computed(() => `${collectorInfo.value?.name ?? '采集器'} · ${collectorOnline.value ? 'MQTT 在线' : '离线'}`)

const canCreateDevice = computed(() => !!collectorInfo.value && modelOptions.value.length > 0)

// 串口编辑
const serialEditVisible = ref(false)
const serialEditBusParam = ref('')
const serialEditLabel = ref('')
const serialEditType = ref('device')
const serialEditActive = ref(true)
const savingSerialPort = ref(false)
let serialEditPortId = 0

const dialogVisible = ref(false); const isEdit = ref(false); const submitLoading = ref(false)
const formRef = ref<FormInstance>()
const form = reactive({
  id: null as number | null, code: '', name: '', collectorId: null as number | null,
  serialPortId: null as number | null, deviceModelId: null as number | null, protocolId: null as number | null,
  slaveAddr: 1, collectInterval: ''
})

const rules: FormRules = {
  code: [{ required: true, message: '请输入编码', trigger: 'blur' }],
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }],
  collectorId: [{ required: true, message: '请选择采集器', trigger: 'change' }],
  serialPortId: [{ required: true, message: '请选择串口', trigger: 'change' }],
  deviceModelId: [{ required: true, message: '请选择型号', trigger: 'change' }],
  slaveAddr: [{ required: true, message: '请输入从站地址', trigger: 'blur' }]
}

const dialogCollectorOptions = ref<any[]>([]); const dialogSerialPortOptions = ref<any[]>([])
const dialogModelOptions = ref<any[]>([]); const dialogProtocolOptions = ref<any[]>([]); const displayProtocolName = ref('')

function statusLabel(status: string): string {
  const map: Record<string, string> = { online: '在线', offline: '离线', alarm: '告警', disabled: '已禁用' }
  return map[status] ?? status
}

function portTypeLabel(t: string): string {
  const map: Record<string, string> = { device: '设备 (RS485)', io_board: 'IO板卡', sms_modem: '短信猫' }
  return map[t] ?? t
}

function busParamField(row: any, key: string): string {
  try { const p = typeof row.busParam === 'string' ? JSON.parse(row.busParam) : row.busParam; return String(p?.[key] ?? '-') }
  catch { return '-' }
}

// 采集器 & 串口加载
async function loadCollectorAndPorts() {
  try {
    const res = await collectorApi.list({ page: 1, pageSize: 1 })
    const collectors = res.data?.records ?? []
    if (collectors.length > 0) {
      collectorInfo.value = collectors[0]
      collectorOnline.value = collectorInfo.value.status === 'online'
      editCollectorName.value = collectorInfo.value.name ?? ''
      editCollectorCode.value = collectorInfo.value.code ?? ''
      editCollectorDesc.value = collectorInfo.value.description ?? ''
      editMqttClientId.value = collectorInfo.value.mqttClientId ?? ''
      editCollectorIp.value = collectorInfo.value.ipAddress ?? ''
      editInterval.value = collectorInfo.value.collectIntervalSec ?? 600
      await loadSerialPorts(collectorInfo.value.id)
    }
  } catch { collectorInfo.value = null }
}

async function loadSerialPorts(collectorId: number) {
  try {
    const res = await collectorApi.getSerialPorts(collectorId)
    serialPortList.value = res.data ?? []
    filterSerialPortOptions.value = serialPortList.value.filter((p: any) => p.portType === 'device')
  } catch { serialPortList.value = []; filterSerialPortOptions.value = [] }
}

async function saveCollectorConfig() {
  if (!collectorInfo.value) return
  savingCollector.value = true
  try {
    await collectorApi.update(collectorInfo.value.id, {
      name: editCollectorName.value,
      code: editCollectorCode.value,
      mqttClientId: editMqttClientId.value,
      ipAddress: editCollectorIp.value,
      collectIntervalSec: editInterval.value,
      description: editCollectorDesc.value
    })
    collectorInfo.value.name = editCollectorName.value
    collectorInfo.value.code = editCollectorCode.value
    collectorInfo.value.mqttClientId = editMqttClientId.value
    collectorInfo.value.ipAddress = editCollectorIp.value
    collectorInfo.value.collectIntervalSec = editInterval.value
    collectorInfo.value.description = editCollectorDesc.value
    ElMessage.success('采集器设置已保存')
  } catch { ElMessage.error('保存失败') }
  finally { savingCollector.value = false }
}

// 串口编辑
function openSerialPortEdit(port: any) {
  serialEditPortId = port.id
  serialEditLabel.value = port.portLabel ?? ''
  serialEditType.value = port.portType ?? 'device'
  serialEditActive.value = port.isActive === 1 || port.isActive === true
  serialEditBusParam.value = port.busParam ?? '{"baud":9600,"data_bits":8,"stop_bits":1,"parity":"none"}'
  serialEditVisible.value = true
}

async function saveSerialPortEdit() {
  if (!collectorInfo.value) return
  savingSerialPort.value = true
  try {
    await collectorApi.updateSerialPort(collectorInfo.value.id, serialEditPortId, {
      portLabel: serialEditLabel.value,
      portType: serialEditType.value,
      isActive: serialEditActive.value ? 1 : 0,
      busParam: serialEditBusParam.value
    })
    ElMessage.success('串口参数已保存')
    serialEditVisible.value = false
    await loadSerialPorts(collectorInfo.value.id)
  } catch { ElMessage.error('保存失败') }
  finally { savingSerialPort.value = false }
}

async function fetchModelOptions() {
  try { const res: any = await deviceModelApi.list({ page: 1, pageSize: 999 }); modelOptions.value = res.data?.records ?? [] }
  catch { modelOptions.value = [] }
}

async function fetchList() {
  loading.value = true
  try {
    const res: any = await deviceApi.list({ page: page.value, pageSize: pageSize.value,
      keyword: searchKeyword.value || undefined,
      collectorId: collectorInfo.value?.id || undefined,
      serialPortId: filterSerialPortId.value || undefined, modelId: filterModelId.value || undefined,
      status: filterStatus.value || undefined })
    tableData.value = res.data?.records ?? []; total.value = res.data?.total ?? 0
  } finally { loading.value = false }
}
function handleSearch() { page.value = 1; fetchList() }
async function onDialogCollectorChange(val: number | null) {
  form.serialPortId = null; dialogSerialPortOptions.value = []
  if (val) { try { const res: any = await collectorApi.getSerialPorts(val); dialogSerialPortOptions.value = (res.data ?? []).filter((p: any) => p.portType === 'device') } catch { dialogSerialPortOptions.value = [] } }
}
function onModelChange(val: number | null) {
  form.protocolId = null; displayProtocolName.value = ''
  if (val) {
    const model = dialogModelOptions.value.find((m: any) => m.id === val)
    if (model) { form.protocolId = model.protocol_id ?? model.protocolId ?? null; displayProtocolName.value = model.protocol_name ?? model.protocolName ?? ''; dialogProtocolOptions.value = [{ id: form.protocolId, name: displayProtocolName.value }] }
  }
}
async function openDialog(row?: any) {
  isEdit.value = !!row
  // 加载所有采集器（客户端 MQTT 自行注册）
  try { const res: any = await collectorApi.list({ page: 1, pageSize: 999 }); dialogCollectorOptions.value = res.data?.records ?? [] } catch { dialogCollectorOptions.value = [] }
  // 新建设备时默认选中第一个采集器
  if (!row && dialogCollectorOptions.value.length > 0) {
    form.collectorId = dialogCollectorOptions.value[0].id
    try { const res = await collectorApi.getSerialPorts(form.collectorId); dialogSerialPortOptions.value = (res.data ?? []).filter((p: any) => p.portType === 'device') } catch { dialogSerialPortOptions.value = [] }
  }
  try { const res: any = await deviceModelApi.list({ page: 1, pageSize: 999 }); dialogModelOptions.value = res.data?.records ?? [] } catch { dialogModelOptions.value = [] }
  if (row) {
    form.id = row.id; form.code = row.code; form.name = row.name
    form.collectorId = row.collectorId ?? null; form.serialPortId = row.serialPortId ?? null
    form.deviceModelId = row.modelId ?? null; form.protocolId = row.protocolId ?? null
    form.slaveAddr = row.slaveAddr ?? 1; form.collectInterval = row.collectIntervalSec ?? ''
    if (row.protocolId) { displayProtocolName.value = row.protocolName ?? row.protocol_name ?? ''; dialogProtocolOptions.value = [{ id: row.protocolId, name: displayProtocolName.value }] }
    if (row.collectorId) { try { const res: any = await collectorApi.getSerialPorts(row.collectorId); dialogSerialPortOptions.value = (res.data ?? []).filter((p: any) => p.portType === 'device') } catch { dialogSerialPortOptions.value = [] } }
  }
  dialogVisible.value = true
}
function resetForm() {
  formRef.value?.resetFields(); form.id = null; form.code = ''; form.name = ''
  form.collectorId = null; form.serialPortId = null; form.deviceModelId = null; form.protocolId = null
  form.slaveAddr = 1; form.collectInterval = ''; dialogSerialPortOptions.value = []; dialogProtocolOptions.value = []; displayProtocolName.value = ''
}
async function handleSubmit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return
  submitLoading.value = true
  try {
    const payload = { code: form.code, name: form.name, serialPortId: form.serialPortId, modelId: form.deviceModelId, slaveAddr: form.slaveAddr, collectIntervalSec: form.collectInterval ? Number(form.collectInterval) : undefined }
    if (isEdit.value && form.id) { await deviceApi.update(form.id, payload); ElMessage.success('更新成功') }
    else { await deviceApi.create(payload); ElMessage.success('创建设备成功，已触发MQTT下发') }
    dialogVisible.value = false; fetchList()
  } finally { submitLoading.value = false }
}
async function handleDelete(row: any) {
  await ElMessageBox.confirm('确定删除该设备吗？', '提示', { confirmButtonText: '确定', cancelButtonText: '取消', type: 'warning' })
  await deviceApi.remove(row.id); ElMessage.success('删除成功'); fetchList()
}
async function handleDisable(row: any) {
  const targetStatus = row.status === 'disabled' ? (row._prevStatus || 'offline') : 'disabled'
  const label = targetStatus === 'disabled' ? '禁用' : '启用'
  await ElMessageBox.confirm('确定' + label + '该设备吗？', '提示', { confirmButtonText: '确定', cancelButtonText: '取消', type: 'warning' })
  if (row.status !== 'disabled') row._prevStatus = row.status
  await deviceApi.updateStatus(row.id, targetStatus); ElMessage.success(label + '成功'); fetchList()
}
async function handlePushConfig(row: any) {
  try {
    await deviceApi.pushConfig(row.id)
    ElMessage.success('配置下发中，采集端将自动接收')
  } catch { ElMessage.error('下发失败，请检查 MQTT 连接') }
}
onMounted(() => { loadCollectorAndPorts(); fetchModelOptions(); fetchList() })
</script>

<style scoped>
.device-link { color: var(--color-brand-500); font-weight: var(--font-weight-medium); text-decoration: none; }
.device-link:hover { color: var(--color-brand-600); text-decoration: underline; }

.inline-status {
  font-size: 11px; font-weight: var(--font-weight-semibold);
  padding: 3px 10px; border-radius: var(--radius-full);
}
.inline-status.online { background: var(--color-success-50); color: var(--color-success-600); }
.inline-status.offline { background: var(--color-gray-100); color: var(--color-gray-500); }
.inline-status.alarm { background: var(--color-danger-50); color: var(--color-danger-600); }
.inline-status.disabled { background: var(--color-warning-50); color: var(--color-warning-600); }

.collector-panel { margin-bottom: var(--space-4); background: var(--surface-primary); border: 1px solid var(--border-light); border-radius: var(--radius-lg); }
.collector-panel :deep(.el-collapse-item__header) { padding: 0 var(--space-4); height: 48px; font-size: var(--text-sm); }
.collector-panel :deep(.el-collapse-item__content) { padding: 0 var(--space-4) var(--space-4); }
.panel-title-row { display: flex; align-items: center; gap: var(--space-2); width: 100%; }
.panel-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
.panel-dot.online { background: var(--color-success-500); }
.panel-dot.offline { background: var(--color-gray-400); }
.panel-sub { font-size: 12px; color: var(--color-gray-400); font-weight: var(--font-weight-normal); }
.panel-meta { font-size: 12px; color: var(--color-gray-400); margin-left: auto; font-weight: var(--font-weight-normal); }
.collector-config { display: flex; flex-wrap: wrap; gap: var(--space-3) var(--space-6); align-items: center; }
.config-row { display: flex; align-items: center; gap: var(--space-3); }
.config-label { font-size: 13px; font-weight: var(--font-weight-medium); color: var(--color-gray-600); }
</style>
