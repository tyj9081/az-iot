<template>
  <div class="device-page page-container">
    <div class="page-toolbar">
      <div class="filter-group">
        <el-select v-model="filterCollectorId" placeholder="采集器" clearable style="width:180px" @change="onFilterCollectorChange">
          <el-option v-for="c in collectorOptions" :key="c.id" :label="c.name + ' (' + c.code + ')'" :value="c.id" />
        </el-select>
        <el-select v-model="filterSerialPortId" placeholder="串口" clearable :disabled="!filterCollectorId" style="width:160px" @change="handleSearch">
          <el-option v-for="p in filterSerialPortOptions" :key="p.id" :label="p.portName" :value="p.id" />
        </el-select>
        <el-select v-model="filterModelId" placeholder="型号" clearable style="width:180px" @change="handleSearch">
          <el-option v-for="m in modelOptions" :key="m.id" :label="m.name + ' (' + m.code + ')'" :value="m.id" />
        </el-select>
        <el-select v-model="filterStatus" placeholder="状态" clearable style="width:120px" @change="handleSearch">
          <el-option label="在线" value="online" />
          <el-option label="离线" value="offline" />
          <el-option label="告警" value="alarm" />
        </el-select>
        <el-input v-model="searchKeyword" placeholder="搜索编码/名称" clearable style="width:220px" @keyup.enter="handleSearch">
          <template #prefix><el-icon><Search /></el-icon></template>
        </el-input>
      </div>
      <el-button type="primary" :disabled="!canCreateDevice" @click="openDialog()">新增设备</el-button>
    </div>

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
          <span :class="['inline-status', realtime.isDeviceOnline(row.id) ? 'online' : row.status]">
            {{ realtime.isDeviceOnline(row.id) ? '在线' : statusLabel(row.status) }}
          </span>
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

    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑设备' : '新增设备'" width="560px" :close-on-click-modal="false" @closed="resetForm">
      <el-form ref="formRef" :model="form" :rules="rules" label-width="110px">
        <el-form-item label="编码" prop="code"><el-input v-model="form.code" placeholder="请输入编码" /></el-form-item>
        <el-form-item label="名称" prop="name"><el-input v-model="form.name" placeholder="请输入名称" /></el-form-item>
        <el-form-item label="采集器" prop="collectorId">
          <el-select v-model="form.collectorId" placeholder="选择采集器" style="width:100%" @change="onDialogCollectorChange">
            <el-option v-for="c in dialogCollectorOptions" :key="c.id" :label="c.name + ' (' + c.code + ')'" :value="c.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="串口" prop="serialPortId" v-if="selectedProtocolBusType === 'serial'">
          <el-select v-model="form.serialPortId" placeholder="选择串口" :disabled="!form.collectorId" style="width:100%">
            <el-option v-for="p in dialogSerialPortOptions" :key="p.id" :label="p.portName" :value="p.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="型号" prop="deviceModelId">
          <el-select v-model="form.deviceModelId" placeholder="选择型号" style="width:100%" @change="onModelChange">
            <el-option v-for="m in dialogModelOptions" :key="m.id" :label="m.name + ' (' + m.code + ')'" :value="m.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="协议">
          <el-select v-model="form.protocolId" placeholder="请先选择型号" disabled style="width:100%">
            <el-option v-for="p in dialogProtocolOptions" :key="p.id" :label="p.name" :value="p.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="从站地址" prop="slaveAddr" v-if="selectedProtocolBusType === 'serial'">
          <el-input-number v-model="form.slaveAddr" :min="1" :max="255" style="width:100%" />
        </el-form-item>
        <el-form-item v-if="selectedProtocolBusType === 'tcp'" label="连接方式">
          <el-tag type="info">TCP / IP 连接，使用采集器 IP 地址</el-tag>
        </el-form-item>
        <el-form-item label="采集间隔(秒)" prop="collectInterval">
          <el-input v-model="form.collectInterval" placeholder="留空继承型号默认值" style="width:100%" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitLoading" @click="handleSubmit">确定</el-button>
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
import NextStepButton from '@/components/NextStepButton.vue'
import { useRealtime } from '@/composables/useRealtime'

const router = useRouter()
const realtime = useRealtime()
const searchKeyword = ref('')
const filterCollectorId = ref<number | null>(null)
const filterSerialPortId = ref<number | null>(null)
const filterModelId = ref<number | null>(null)
const filterStatus = ref('')
const loading = ref(false)
const tableData = ref<any[]>([])
const page = ref(1); const pageSize = ref(10); const total = ref(0)
const collectorOptions = ref<any[]>([]); const filterSerialPortOptions = ref<any[]>([]); const modelOptions = ref<any[]>([])

const canCreateDevice = computed(() => modelOptions.value.length > 0)

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
const selectedProtocolBusType = ref('serial') // serial / tcp, 选型号后自动判断

function statusLabel(status: string): string {
  const map: Record<string, string> = { online: '在线', offline: '离线', alarm: '告警', disabled: '已禁用' }
  return map[status] ?? status
}

async function fetchCollectorOptions() {
  try { const res: any = await collectorApi.list({ page: 1, pageSize: 999 }); collectorOptions.value = res.data?.records ?? [] }
  catch { collectorOptions.value = [] }
}
async function fetchModelOptions() {
  try { const res: any = await deviceModelApi.list({ page: 1, pageSize: 999 }); modelOptions.value = res.data?.records ?? [] }
  catch { modelOptions.value = [] }
}
async function onFilterCollectorChange(val: number | null) {
  filterSerialPortId.value = null; filterSerialPortOptions.value = []
  if (val) { try { const res: any = await collectorApi.getSerialPorts(val); filterSerialPortOptions.value = (res.data ?? []).filter((p: any) => p.portType === 'device') } catch { filterSerialPortOptions.value = [] } }
  handleSearch()
}
async function fetchList() {
  loading.value = true
  try {
    const res: any = await deviceApi.list({ page: page.value, pageSize: pageSize.value,
      keyword: searchKeyword.value || undefined, collectorId: filterCollectorId.value || undefined,
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
  form.protocolId = null; displayProtocolName.value = ''; selectedProtocolBusType.value = 'serial'
  if (val) {
    const model = dialogModelOptions.value.find((m: any) => m.id === val)
    if (model) {
      form.protocolId = model.protocol_id ?? model.protocolId ?? null
      displayProtocolName.value = model.protocol_name ?? model.protocolName ?? ''
      dialogProtocolOptions.value = [{ id: form.protocolId, name: displayProtocolName.value }]
      selectedProtocolBusType.value = getProtocolBusType(displayProtocolName.value)
    }
  }
}

/** 根据协议名判断总线类型 */
function getProtocolBusType(name: string): string {
  const tcpProtocols = ['MODBUS_TCP', 'IEC_60870_5_104', 'DNP3', 'OPC_UA', 'BACNET_IP', 'S7_COMM',
    'FINS_TCP', 'ETHERNET_IP', 'MITSUBISHI_MC', 'MQTT', 'SNMP_V2C', 'HTTP_JSON']
  return tcpProtocols.includes(name.toUpperCase()) ? 'tcp' : 'serial'
}
async function openDialog(row?: any) {
  isEdit.value = !!row
  try { const res: any = await collectorApi.list({ page: 1, pageSize: 999 }); dialogCollectorOptions.value = res.data?.records ?? [] } catch { dialogCollectorOptions.value = [] }
  if (!row && dialogCollectorOptions.value.length > 0) {
    form.collectorId = dialogCollectorOptions.value[0].id
    try { const res = await collectorApi.getSerialPorts(form.collectorId!); dialogSerialPortOptions.value = (res.data ?? []).filter((p: any) => p.portType === 'device') } catch { dialogSerialPortOptions.value = [] }
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
  form.slaveAddr = 1; form.collectInterval = ''; dialogSerialPortOptions.value = []; dialogProtocolOptions.value = []; displayProtocolName.value = ''; selectedProtocolBusType.value = 'serial'
}
async function handleSubmit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return
  submitLoading.value = true
  try {
    const payload: any = { code: form.code, name: form.name, modelId: form.deviceModelId, slaveAddr: form.slaveAddr, collectIntervalSec: form.collectInterval ? Number(form.collectInterval) : undefined }
    if (selectedProtocolBusType.value === 'serial') {
      payload.serialPortId = form.serialPortId
    }
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
  try { await deviceApi.pushConfig(row.id); ElMessage.success('配置下发中，采集端将自动接收') }
  catch { ElMessage.error('下发失败，请检查 MQTT 连接') }
}
onMounted(() => { fetchCollectorOptions(); fetchModelOptions(); fetchList(); realtime.connect() })
</script>

<style scoped>
.device-link { color: var(--color-brand-500); font-weight: var(--font-weight-medium); text-decoration: none; }
.device-link:hover { color: var(--color-brand-600); text-decoration: underline; }
.inline-status { font-size: 11px; font-weight: var(--font-weight-semibold); padding: 3px 10px; border-radius: var(--radius-full); }
.inline-status.online { background: var(--color-success-50); color: var(--color-success-600); }
.inline-status.offline { background: var(--color-gray-100); color: var(--color-gray-500); }
.inline-status.alarm { background: var(--color-danger-50); color: var(--color-danger-600); }
.inline-status.disabled { background: var(--color-warning-50); color: var(--color-warning-600); }
</style>
