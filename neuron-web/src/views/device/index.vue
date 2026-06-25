<template>
  <div class="device-page">
    <div class="page-header">
      <div class="filter-bar">
        <el-select
          v-model="filterCollectorId"
          placeholder="采集器"
          clearable
          style="width: 180px"
          @change="onFilterCollectorChange"
        >
          <el-option
            v-for="c in collectorOptions"
            :key="c.id"
            :label="c.name + ' (' + c.code + ')'"
            :value="c.id"
          />
        </el-select>
        <el-select
          v-model="filterSerialPortId"
          placeholder="串口"
          clearable
          :disabled="!filterCollectorId"
          style="width: 160px"
          @change="handleSearch"
        >
          <el-option
            v-for="p in filterSerialPortOptions"
            :key="p.id"
            :label="p.portName"
            :value="p.id"
          />
        </el-select>
        <el-select
          v-model="filterModelId"
          placeholder="型号"
          clearable
          style="width: 180px"
          @change="handleSearch"
        >
          <el-option
            v-for="m in modelOptions"
            :key="m.id"
            :label="m.name + ' (' + m.code + ')'"
            :value="m.id"
          />
        </el-select>
        <el-select
          v-model="filterStatus"
          placeholder="状态"
          clearable
          style="width: 120px"
          @change="handleSearch"
        >
          <el-option label="全部" value="" />
          <el-option label="在线" value="online" />
          <el-option label="离线" value="offline" />
          <el-option label="告警" value="alarm" />
        </el-select>
        <el-input
          v-model="searchKeyword"
          placeholder="搜索编码/名称"
          clearable
          style="width: 220px"
          @keyup.enter="handleSearch"
        >
          <template #prefix><el-icon><Search /></el-icon></template>
        </el-input>
      </div>
      <el-button type="primary" @click="openDialog()">新增设备</el-button>
    </div>

    <el-table v-loading="loading" :data="tableData" border stripe>
      <el-table-column prop="code" label="编码" min-width="120" />
      <el-table-column prop="name" label="名称" min-width="150" />
      <el-table-column prop="modelName" label="型号" width="140" />
      <el-table-column prop="protocolName" label="协议" width="100" />
      <el-table-column prop="collectorName" label="采集器" width="120" />
      <el-table-column prop="portName" label="串口" width="100" />
      <el-table-column prop="slaveAddr" label="从站地址" width="100" />
      <el-table-column prop="collectInterval" label="采集间隔(ms)" width="130">
        <template #default="{ row }">
          {{ row.collectInterval ?? '-' }}
        </template>
      </el-table-column>
      <el-table-column label="状态" width="90">
        <template #default="{ row }">
          <el-tag :type="statusTagType(row.status)">
            {{ statusLabel(row.status) }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="200" fixed="right">
        <template #default="{ row }">
          <el-button link type="success" size="small" @click="router.push('/device/' + row.id)">详情</el-button>
          <el-button link type="primary" size="small" @click="openDialog(row)">编辑</el-button>
          <el-button link type="danger" size="small" @click="handleDelete(row)">删除</el-button>
          <el-button link type="warning" size="small" @click="handleDisable(row)">
            {{ row.status === 'disabled' ? '启用' : '禁用' }}
          </el-button>
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
      :title="isEdit ? '编辑设备' : '新增设备'"
      width="560px"
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
        <el-form-item label="采集器" prop="collectorId">
          <el-select
            v-model="form.collectorId"
            placeholder="选择采集器"
            style="width: 100%"
            @change="onDialogCollectorChange"
          >
            <el-option
              v-for="c in dialogCollectorOptions"
              :key="c.id"
              :label="c.name + ' (' + c.code + ')'"
              :value="c.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="串口" prop="serialPortId">
          <el-select
            v-model="form.serialPortId"
            placeholder="选择串口"
            :disabled="!form.collectorId"
            style="width: 100%"
          >
            <el-option
              v-for="p in dialogSerialPortOptions"
              :key="p.id"
              :label="p.portName"
              :value="p.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="型号" prop="deviceModelId">
          <el-select
            v-model="form.deviceModelId"
            placeholder="选择型号"
            style="width: 100%"
          >
            <el-option
              v-for="m in dialogModelOptions"
              :key="m.id"
              :label="m.name + ' (' + m.code + ')'"
              :value="m.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="从站地址" prop="slaveAddr">
          <el-input-number v-model="form.slaveAddr" :min="1" :max="255" style="width: 100%" />
        </el-form-item>
        <el-form-item label="采集间隔(ms)" prop="collectInterval">
          <el-input
            v-model="form.collectInterval"
            placeholder="留空继承型号默认值"
            style="width: 100%"
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
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Search } from '@element-plus/icons-vue'
import type { FormInstance, FormRules } from 'element-plus'
import { deviceApi } from '@/api/device'
import { collectorApi } from '@/api/collector'
import { deviceModelApi } from '@/api/device-model'

const router = useRouter()

const searchKeyword = ref('')
const filterCollectorId = ref<number | null>(null)
const filterSerialPortId = ref<number | null>(null)
const filterModelId = ref<number | null>(null)
const filterStatus = ref('')
const loading = ref(false)
const tableData = ref<any[]>([])
const page = ref(1)
const pageSize = ref(10)
const total = ref(0)

const collectorOptions = ref<any[]>([])
const filterSerialPortOptions = ref<any[]>([])
const modelOptions = ref<any[]>([])

const dialogVisible = ref(false)
const isEdit = ref(false)
const submitLoading = ref(false)
const formRef = ref<FormInstance>()
const form = reactive({
  id: null as number | null,
  code: '',
  name: '',
  collectorId: null as number | null,
  serialPortId: null as number | null,
  deviceModelId: null as number | null,
  slaveAddr: 1,
  collectInterval: ''
})

const rules: FormRules = {
  code: [{ required: true, message: '请输入编码', trigger: 'blur' }],
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }],
  collectorId: [{ required: true, message: '请选择采集器', trigger: 'change' }],
  serialPortId: [{ required: true, message: '请选择串口', trigger: 'change' }],
  deviceModelId: [{ required: true, message: '请选择型号', trigger: 'change' }],
  slaveAddr: [{ required: true, message: '请输入从站地址', trigger: 'blur' }]
}

const dialogCollectorOptions = ref<any[]>([])
const dialogSerialPortOptions = ref<any[]>([])
const dialogModelOptions = ref<any[]>([])

function statusTagType(status: string): '' | 'success' | 'info' | 'warning' | 'danger' {
  const map: Record<string, '' | 'success' | 'info' | 'warning' | 'danger'> = {
    online: 'success',
    offline: 'info',
    alarm: 'danger',
    disabled: 'warning'
  }
  return map[status] ?? 'info'
}

function statusLabel(status: string): string {
  const map: Record<string, string> = {
    online: '在线',
    offline: '离线',
    alarm: '告警',
    disabled: '已禁用'
  }
  return map[status] ?? status
}

async function fetchCollectorOptions() {
  try {
    const res: any = await collectorApi.list({ page: 1, pageSize: 999 })
    collectorOptions.value = res.data?.records ?? []
  } catch {
    collectorOptions.value = []
  }
}

async function fetchModelOptions() {
  try {
    const res: any = await deviceModelApi.list({ page: 1, pageSize: 999 })
    modelOptions.value = res.data?.records ?? []
  } catch {
    modelOptions.value = []
  }
}

async function onFilterCollectorChange(val: number | null) {
  filterSerialPortId.value = null
  filterSerialPortOptions.value = []
  if (val) {
    try {
      const res: any = await collectorApi.getSerialPorts(val)
      filterSerialPortOptions.value = res.data ?? []
    } catch {
      filterSerialPortOptions.value = []
    }
  }
  handleSearch()
}

async function fetchList() {
  loading.value = true
  try {
    const res: any = await deviceApi.list({
      page: page.value,
      pageSize: pageSize.value,
      keyword: searchKeyword.value || undefined,
      collectorId: filterCollectorId.value || undefined,
      serialPortId: filterSerialPortId.value || undefined,
      deviceModelId: filterModelId.value || undefined,
      status: filterStatus.value || undefined
    })
    tableData.value = res.data?.records ?? []
    total.value = res.data?.total ?? 0
  } finally {
    loading.value = false
  }
}

function handleSearch() {
  page.value = 1
  fetchList()
}

async function onDialogCollectorChange(val: number | null) {
  form.serialPortId = null
  dialogSerialPortOptions.value = []
  if (val) {
    try {
      const res: any = await collectorApi.getSerialPorts(val)
      dialogSerialPortOptions.value = res.data ?? []
    } catch {
      dialogSerialPortOptions.value = []
    }
  }
}

async function openDialog(row?: any) {
  isEdit.value = !!row

  // load dialog dropdowns
  try {
    const res: any = await collectorApi.list({ page: 1, pageSize: 999 })
    dialogCollectorOptions.value = res.data?.records ?? []
  } catch {
    dialogCollectorOptions.value = []
  }
  try {
    const res: any = await deviceModelApi.list({ page: 1, pageSize: 999 })
    dialogModelOptions.value = res.data?.records ?? []
  } catch {
    dialogModelOptions.value = []
  }

  if (row) {
    form.id = row.id
    form.code = row.code
    form.name = row.name
    form.collectorId = row.collectorId ?? null
    form.serialPortId = row.serialPortId ?? null
    form.deviceModelId = row.deviceModelId ?? null
    form.slaveAddr = row.slaveAddr ?? 1
    form.collectInterval = row.collectInterval ?? ''

    // load serial ports for the selected collector
    if (row.collectorId) {
      try {
        const res: any = await collectorApi.getSerialPorts(row.collectorId)
        dialogSerialPortOptions.value = res.data ?? []
      } catch {
        dialogSerialPortOptions.value = []
      }
    }
  }

  dialogVisible.value = true
}

function resetForm() {
  formRef.value?.resetFields()
  form.id = null
  form.code = ''
  form.name = ''
  form.collectorId = null
  form.serialPortId = null
  form.deviceModelId = null
  form.slaveAddr = 1
  form.collectInterval = ''
  dialogSerialPortOptions.value = []
}

async function handleSubmit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return
  submitLoading.value = true
  try {
    const payload = {
      code: form.code,
      name: form.name,
      collectorId: form.collectorId,
      serialPortId: form.serialPortId,
      deviceModelId: form.deviceModelId,
      slaveAddr: form.slaveAddr,
      collectInterval: form.collectInterval ? Number(form.collectInterval) : undefined
    }
    if (isEdit.value && form.id) {
      await deviceApi.update(form.id, payload)
      ElMessage.success('更新成功')
    } else {
      await deviceApi.create(payload)
      ElMessage.success('创建设备成功，已触发MQTT下发')
    }
    dialogVisible.value = false
    fetchList()
  } finally {
    submitLoading.value = false
  }
}

async function handleDelete(row: any) {
  await ElMessageBox.confirm('确定删除该设备吗？', '提示', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  })
  await deviceApi.remove(row.id)
  ElMessage.success('删除成功')
  fetchList()
}

async function handleDisable(row: any) {
  const newStatus = row.status === 'disabled' ? 1 : 0
  const label = newStatus === 1 ? '启用' : '禁用'
  await ElMessageBox.confirm('确定' + label + '该设备吗？', '提示', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  })
  await deviceApi.updateStatus(row.id, newStatus)
  ElMessage.success(label + '成功')
  fetchList()
}

onMounted(() => {
  fetchCollectorOptions()
  fetchModelOptions()
  fetchList()
})
</script>

<style scoped>
.device-page {
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
  flex-wrap: wrap;
}
</style>
