<template>
  <div class="collector-page page-container">
    <div class="page-toolbar">
      <h2 style="margin:0;font-size:16px;font-weight:500;color:var(--color-gray-800)">采集器</h2>
      <span class="collector-hint">采集器通过 MQTT 连接后自动注册，状态由 MQTT status 消息实时更新</span>
    </div>

    <el-table v-loading="loading" :data="tableData" stripe row-key="id">
      <el-table-column type="expand">
        <template #default="{ row }">
          <div class="expand-devices" v-loading="row._loadingDevices">
            <div v-if="(row._devices || []).length === 0 && !row._loadingDevices" class="expand-empty">暂无设备</div>
            <el-table :data="row._devices || []" size="small" stripe v-else>
              <el-table-column prop="code" label="编码" width="140" />
              <el-table-column prop="name" label="名称" min-width="160">
                <template #default="{ row: d }">
                  <router-link :to="'/device/' + d.id" class="device-link">{{ d.name }}</router-link>
                </template>
              </el-table-column>
              <el-table-column prop="modelName" label="型号" width="140" />
              <el-table-column prop="serialPortName" label="串口" width="100" />
              <el-table-column prop="slaveAddr" label="从站地址" width="100" />
              <el-table-column label="状态" width="90" align="center">
                <template #default="{ row: d }">
                  <span :class="['inline-status', d.status === 'online' ? 'online' : 'offline']">{{ d.status === 'online' ? '在线' : '离线' }}</span>
                </template>
              </el-table-column>
            </el-table>
            <div class="expand-actions" v-if="(row._devices || []).length > 0">
              <el-button link type="primary" size="small" @click="goDevices(row)">管理全部设备 →</el-button>
            </div>
          </div>
        </template>
      </el-table-column>
      <el-table-column label="名称" min-width="140">
        <template #default="{ row }">
          <span class="collector-name" @click="toggleExpand(row)">{{ row.name }}</span>
        </template>
      </el-table-column>
      <el-table-column prop="code" label="编码" width="140" />
      <el-table-column prop="mqttClientId" label="MQTT客户端ID" min-width="180" />
      <el-table-column prop="ipAddress" label="IP地址" width="150">
        <template #default="{ row }">{{ row.ipAddress || '-' }}</template>
      </el-table-column>
      <el-table-column label="状态" width="100" align="center">
        <template #default="{ row }">
          <span :class="['inline-status', row.status === 'online' ? 'online' : 'offline']">
            {{ row.status === 'online' ? '在线' : '离线' }}
          </span>
        </template>
      </el-table-column>
      <el-table-column label="最后心跳" min-width="160">
        <template #default="{ row }">{{ row.lastHeartbeat ? formatTime(row.lastHeartbeat) : '-' }}</template>
      </el-table-column>
      <el-table-column label="操作" width="160" align="center" fixed="right">
        <template #default="{ row }">
          <el-button link type="primary" size="small" @click="openEdit(row)">编辑</el-button>
          <el-button link type="info" size="small" @click="pushConfig(row)">下发配置</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="editVisible" title="编辑采集器" width="480px" :close-on-click-modal="false" @closed="resetEdit">
      <el-form ref="editFormRef" :model="editForm" label-width="100px">
        <el-form-item label="名称"><el-input v-model="editForm.name" /></el-form-item>
        <el-form-item label="编码"><el-input v-model="editForm.code" /></el-form-item>
        <el-form-item label="MQTT客户端ID"><el-input v-model="editForm.mqttClientId" /></el-form-item>
        <el-form-item label="IP地址"><el-input v-model="editForm.ipAddress" /></el-form-item>
        <el-form-item label="采集周期(秒)">
          <el-input-number v-model="editForm.collectIntervalSec" :min="1" :max="86400" style="width:100%" />
        </el-form-item>
        <el-form-item label="描述"><el-input v-model="editForm.description" type="textarea" :rows="2" /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="editVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="saveEdit">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { collectorApi } from '@/api/collector'
import { deviceApi } from '@/api/device'

const router = useRouter()
const loading = ref(false)
const tableData = ref<any[]>([])
const editVisible = ref(false)
const saving = ref(false)
const editId = ref(0)
const editForm = reactive({ name: '', code: '', mqttClientId: '', ipAddress: '', collectIntervalSec: 5, description: '' })

function formatTime(ts: string): string {
  if (!ts) return '-'
  try { return new Date(ts).toLocaleString('zh-CN') } catch { return ts }
}

async function toggleExpand(row: any) {
  if (!row._devices) {
    row._loadingDevices = true
    try {
      const res: any = await deviceApi.list({ page: 1, pageSize: 999, collectorId: row.id })
      row._devices = res.data?.records ?? []
    } finally { row._loadingDevices = false }
  }
}

function goDevices(row: any) {
  router.push({ path: '/device', query: { collectorId: row.id } })
}

function openEdit(row: any) {
  editId.value = row.id
  editForm.name = row.name ?? ''
  editForm.code = row.code ?? ''
  editForm.mqttClientId = row.mqttClientId ?? ''
  editForm.ipAddress = row.ipAddress ?? ''
  editForm.collectIntervalSec = row.collectIntervalSec || 5
  editForm.description = row.description ?? ''
  editVisible.value = true
}

function resetEdit() {
  editId.value = 0
  editForm.name = ''; editForm.code = ''; editForm.mqttClientId = ''
  editForm.ipAddress = ''; editForm.collectIntervalSec = 5; editForm.description = ''
}

async function saveEdit() {
  saving.value = true
  try {
    await collectorApi.update(editId.value, { ...editForm })
    const item = tableData.value.find((r: any) => r.id === editId.value)
    if (item) {
      item.name = editForm.name; item.code = editForm.code
      item.mqttClientId = editForm.mqttClientId; item.ipAddress = editForm.ipAddress
      item.collectIntervalSec = editForm.collectIntervalSec; item.description = editForm.description
    }
    ElMessage.success('已保存')
    editVisible.value = false
  } catch { ElMessage.error('保存失败') }
  finally { saving.value = false }
}

async function pushConfig(row: any) {
  try {
    await collectorApi.pushConfig(row.id)
    ElMessage.success('配置下发中，采集端将自动接收')
  } catch { ElMessage.error('下发失败') }
}

async function fetchList() {
  loading.value = true
  try {
    const res: any = await collectorApi.list({ page: 1, pageSize: 999 })
    tableData.value = res.data?.records ?? []
  } finally { loading.value = false }
}

onMounted(() => fetchList())
</script>

<style scoped>
.collector-hint { font-size: 12px; color: var(--color-gray-400); }
.collector-name { color: var(--color-brand-500); cursor: pointer; font-weight: var(--font-weight-medium); }
.collector-name:hover { text-decoration: underline; }
.inline-status { font-size: 11px; font-weight: var(--font-weight-semibold); padding: 3px 10px; border-radius: var(--radius-full); }
.inline-status.online { background: var(--color-success-50); color: var(--color-success-600); }
.inline-status.offline { background: var(--color-gray-100); color: var(--color-gray-500); }
.expand-devices { padding: var(--space-3); }
.expand-empty { font-size: 13px; color: var(--color-gray-400); padding: var(--space-4); text-align: center; }
.expand-actions { margin-top: var(--space-2); text-align: right; }
.device-link { color: var(--color-brand-500); text-decoration: none; font-weight: var(--font-weight-medium); }
.device-link:hover { text-decoration: underline; }
</style>
