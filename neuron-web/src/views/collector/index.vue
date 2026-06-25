<template>
  <div class="collector-page">
    <div class="page-header">
      <div class="filter-bar">
        <el-select v-model="statusFilter" placeholder="状态" clearable style="width: 140px" @change="handleSearch">
          <el-option label="全部" value="" />
          <el-option label="在线" value="online" />
          <el-option label="离线" value="offline" />
          <el-option label="告警" value="alarm" />
        </el-select>
        <el-input
          v-model="searchKeyword"
          placeholder="搜索编码/名称"
          clearable
          style="width: 260px"
          @keyup.enter="handleSearch"
        >
          <template #prefix><el-icon><Search /></el-icon></template>
        </el-input>
      </div>
      <el-button type="primary" @click="openDialog()">新增采集器</el-button>
    </div>

    <el-table
      v-loading="loading"
      :data="tableData"
      border
      stripe
      row-key="id"
      @expand-change="handleExpandChange"
    >
      <el-table-column type="expand">
        <template #default="{ row }">
          <div class="expand-panel">
            <h4>串口列表</h4>
            <el-table :data="row.serialPorts" border size="small">
              <el-table-column prop="portName" label="串口名" width="120" />
              <el-table-column label="串口标签" min-width="160">
                <template #default="{ row: port }">
                  <template v-if="port._editing">
                    <el-input v-model="port._portLabel" size="small" />
                  </template>
                  <template v-else>
                    {{ port.portLabel ?? '-' }}
                  </template>
                </template>
              </el-table-column>
              <el-table-column prop="busType" label="总线类型" width="110" />
              <el-table-column label="总线参数" min-width="200">
                <template #default="{ row: port }">
                  <template v-if="port._editing">
                    <el-input v-model="port._busParam" type="textarea" :rows="2" size="small" />
                  </template>
                  <template v-else>
                    <code>{{ port.busParam }}</code>
                  </template>
                </template>
              </el-table-column>
              <el-table-column label="启用" width="80" align="center">
                <template #default="{ row: port }">
                  <template v-if="port._editing">
                    <el-switch v-model="port._isActive" size="small" />
                  </template>
                  <template v-else>
                    <el-switch :model-value="port.isActive" disabled size="small" />
                  </template>
                </template>
              </el-table-column>
              <el-table-column prop="portType" label="端口类型" width="100">
                <template #default="{ row: port }">
                  <el-tag size="small">{{ port.portType }}</el-tag>
                </template>
              </el-table-column>
              <el-table-column label="操作" width="120" fixed="right">
                <template #default="{ row: port }">
                  <template v-if="port._editing">
                    <el-button link type="primary" size="small" @click="saveSerialPort(row, port)">保存</el-button>
                    <el-button link size="small" @click="cancelEditPort(port)">取消</el-button>
                  </template>
                  <template v-else>
                    <el-button link type="primary" size="small" @click="editSerialPort(port)">编辑</el-button>
                  </template>
                </template>
              </el-table-column>
            </el-table>
          </div>
        </template>
      </el-table-column>
      <el-table-column prop="code" label="编码" min-width="120" />
      <el-table-column prop="name" label="名称" min-width="150" />
      <el-table-column prop="type" label="型号" width="120" />
      <el-table-column prop="mqttClientId" label="MQTT客户端ID" min-width="180" />
      <el-table-column prop="ipAddress" label="IP地址" width="140" />
      <el-table-column label="状态" width="100">
        <template #default="{ row }">
          <el-tag :type="statusTagType(row.status)">
            {{ statusLabel(row.status) }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="lastHeartbeat" label="最后心跳" min-width="160" />
      <el-table-column prop="createdAt" label="创建时间" min-width="160" />
      <el-table-column label="操作" width="150" fixed="right">
        <template #default="{ row }">
          <el-button link type="primary" size="small" @click="openDialog(row)">编辑</el-button>
          <el-button link type="danger" size="small" @click="handleDelete(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-pagination
      v-model:current-page="page"
      v-model:page-size="pageSize"
      :total="total"
      :page-sizes="[10, 20, 50]"
      layout="total, sizes, prev, pager, next, jumper"
      @current-change="fetchList"
      @size-change="fetchList"
    />

    <el-dialog
      v-model="dialogVisible"
      :title="isEdit ? '编辑采集器' : '新增采集器'"
      width="520px"
      :close-on-click-modal="false"
      @closed="resetForm"
    >
      <el-form ref="formRef" :model="form" :rules="rules" label-width="110px">
        <el-form-item label="编码" prop="code">
          <el-input v-model="form.code" placeholder="请输入编码" />
        </el-form-item>
        <el-form-item label="名称" prop="name">
          <el-input v-model="form.name" placeholder="请输入名称" />
        </el-form-item>
        <el-form-item label="型号" prop="type">
          <el-input v-model="form.type" placeholder="默认BC-U101" />
        </el-form-item>
        <el-form-item label="MQTT客户端ID" prop="mqttClientId">
          <el-input v-model="form.mqttClientId" placeholder="请输入唯一MQTT客户端ID" />
        </el-form-item>
        <el-form-item label="IP地址" prop="ipAddress">
          <el-input v-model="form.ipAddress" placeholder="请输入IP地址" />
        </el-form-item>
        <el-form-item label="描述" prop="description">
          <el-input
            v-model="form.description"
            type="textarea"
            :rows="3"
            placeholder="请输入描述"
          />
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
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Search } from '@element-plus/icons-vue'
import type { FormInstance, FormRules } from 'element-plus'
import { collectorApi } from '@/api/collector'

const searchKeyword = ref('')
const statusFilter = ref('')
const loading = ref(false)
const tableData = ref<any[]>([])
const page = ref(1)
const pageSize = ref(10)
const total = ref(0)

const dialogVisible = ref(false)
const isEdit = ref(false)
const submitLoading = ref(false)
const formRef = ref<FormInstance>()
const form = reactive({
  id: null as number | null,
  code: '',
  name: '',
  type: 'BC-U101',
  mqttClientId: '',
  ipAddress: '',
  description: ''
})

const rules: FormRules = {
  code: [{ required: true, message: '请输入编码', trigger: 'blur' }],
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }],
  type: [{ required: true, message: '请输入型号', trigger: 'blur' }],
  mqttClientId: [{ required: true, message: '请输入MQTT客户端ID', trigger: 'blur' }],
  ipAddress: [{ required: true, message: '请输入IP地址', trigger: 'blur' }]
}

function statusTagType(status: string): '' | 'success' | 'info' | 'warning' | 'danger' {
  const map: Record<string, '' | 'success' | 'info' | 'warning' | 'danger'> = {
    online: 'success',
    offline: 'info',
    alarm: 'danger'
  }
  return map[status] ?? 'info'
}

function statusLabel(status: string): string {
  const map: Record<string, string> = {
    online: '在线',
    offline: '离线',
    alarm: '告警'
  }
  return map[status] ?? status
}

async function fetchList() {
  loading.value = true
  try {
    const res: any = await collectorApi.list({
      page: page.value,
      pageSize: pageSize.value,
      keyword: searchKeyword.value || undefined,
      status: statusFilter.value || undefined
    })
    tableData.value = (res.data?.records ?? []).map((item: any) => ({
      ...item,
      serialPorts: []
    }))
    total.value = res.data?.total ?? 0
  } finally {
    loading.value = false
  }
}

function handleSearch() {
  page.value = 1
  fetchList()
}

async function handleExpandChange(row: any, expandedRows: any[]) {
  if (expandedRows.some((r: any) => r.id === row.id)) {
    try {
      const res: any = await collectorApi.getSerialPorts(row.id)
      row.serialPorts = (res.data ?? []).map((p: any) => ({
        ...p,
        _editing: false,
        _portLabel: p.portLabel ?? '',
        _busParam: p.busParam ?? '',
        _isActive: p.isActive ?? false
      }))
    } catch {
      row.serialPorts = []
    }
  }
}

function editSerialPort(port: any) {
  port._editing = true
  port._portLabel = port.portLabel ?? ''
  port._busParam = port.busParam ?? ''
  port._isActive = port.isActive ?? false
}

function cancelEditPort(port: any) {
  port._editing = false
}

async function saveSerialPort(collector: any, port: any) {
  try {
    const payload = {
      portLabel: port._portLabel,
      busParam: port._busParam,
      isActive: port._isActive
    }
    await collectorApi.updateSerialPort(collector.id, port.id, payload)
    port.portLabel = port._portLabel
    port.busParam = port._busParam
    port.isActive = port._isActive
    port._editing = false
    ElMessage.success('更新成功')
  } catch {
    // handled by interceptor
  }
}

function openDialog(row?: any) {
  isEdit.value = !!row
  if (row) {
    form.id = row.id
    form.code = row.code
    form.name = row.name
    form.type = row.type ?? 'BC-U101'
    form.mqttClientId = row.mqttClientId ?? ''
    form.ipAddress = row.ipAddress ?? ''
    form.description = row.description ?? ''
  }
  dialogVisible.value = true
}

function resetForm() {
  formRef.value?.resetFields()
  form.id = null
  form.code = ''
  form.name = ''
  form.type = 'BC-U101'
  form.mqttClientId = ''
  form.ipAddress = ''
  form.description = ''
}

async function handleSubmit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return
  submitLoading.value = true
  try {
    const payload = {
      code: form.code,
      name: form.name,
      type: form.type,
      mqttClientId: form.mqttClientId,
      ipAddress: form.ipAddress,
      description: form.description
    }
    if (isEdit.value && form.id) {
      await collectorApi.update(form.id, payload)
      ElMessage.success('更新成功')
    } else {
      await collectorApi.create(payload)
      ElMessage.success('创建成功')
    }
    dialogVisible.value = false
    fetchList()
  } finally {
    submitLoading.value = false
  }
}

async function handleDelete(row: any) {
  await ElMessageBox.confirm('确定删除该采集器吗？', '提示', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  })
  await collectorApi.remove(row.id)
  ElMessage.success('删除成功')
  fetchList()
}

onMounted(() => {
  fetchList()
})
</script>

<style scoped>
.collector-page {
  padding: 0;
}
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}
.filter-bar {
  display: flex;
  gap: 12px;
  align-items: center;
}
.expand-panel {
  padding: 12px 24px;
}
.expand-panel h4 {
  margin: 0 0 8px 0;
  font-size: 14px;
  font-weight: 500;
  color: #303133;
}
.expand-panel code {
  font-size: 12px;
  color: #606266;
  background: #f5f7fa;
  padding: 2px 6px;
  border-radius: 3px;
}
</style>
