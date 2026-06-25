<template>
  <div class="registers-page">
    <div class="page-header">
      <div class="header-left">
        <el-button @click="goBack">返回型号管理</el-button>
        <span class="page-title">点表管理 - {{ modelName }}</span>
      </div>
      <div class="header-right">
        <el-upload
          :auto-upload="false"
          :show-file-list="false"
          accept=".json,.csv"
          @change="handleImportFile"
        >
          <el-button>批量导入</el-button>
        </el-upload>
        <el-button type="primary" @click="openDialog()">新增点位</el-button>
      </div>
    </div>

    <el-table v-loading="loading" :data="tableData" border stripe>
      <el-table-column prop="sensor_code" label="传感器编码" min-width="130" />
      <el-table-column prop="sensor_name" label="传感器名称" min-width="150" />
      <el-table-column label="寄存器地址" min-width="140">
        <template #default="{ row }">
          <span class="register-address">
            <el-tag size="small" type="info">{{ toHex(row.register_address) }}</el-tag>
            <span class="addr-dec">{{ row.register_address }}</span>
          </span>
        </template>
      </el-table-column>
      <el-table-column prop="data_type" label="数据类型" min-width="100" />
      <el-table-column prop="byte_order" label="字节序" min-width="90" />
      <el-table-column prop="coefficient" label="系数" min-width="80" />
      <el-table-column prop="unit" label="单位" min-width="70" />
      <el-table-column prop="rw" label="读写" min-width="70">
        <template #default="{ row }">
          <el-tag v-if="row.rw === 'R'" size="small" type="info">只读</el-tag>
          <el-tag v-else-if="row.rw === 'W'" size="small" type="warning">只写</el-tag>
          <el-tag v-else size="small" type="success">读写</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="sort_order" label="排序" min-width="70" />
      <el-table-column label="操作" width="120" fixed="right">
        <template #default="{ row }">
          <el-button link type="primary" size="small" @click="openDialog(row)">编辑</el-button>
          <el-button link type="danger" size="small" @click="handleDelete(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog
      v-model="dialogVisible"
      :title="isEdit ? '编辑点位' : '新增点位'"
      width="560px"
      :close-on-click-modal="false"
      @closed="resetForm"
    >
      <el-form ref="formRef" :model="form" :rules="rules" label-width="110px">
        <el-form-item label="传感器编码" prop="sensor_code">
          <el-input v-model="form.sensor_code" placeholder="请输入传感器编码" />
        </el-form-item>
        <el-form-item label="传感器名称" prop="sensor_name">
          <el-input v-model="form.sensor_name" placeholder="请输入传感器名称" />
        </el-form-item>
        <el-form-item label="寄存器地址" prop="register_address">
          <el-input-number
            v-model="form.register_address"
            :min="0"
            :max="65535"
            style="width: 100%"
            placeholder="请输入寄存器地址(十进制)"
          />
        </el-form-item>
        <el-form-item label="数据类型" prop="data_type">
          <el-select v-model="form.data_type" placeholder="选择数据类型" style="width: 100%">
            <el-option label="int16" value="int16" />
            <el-option label="uint16" value="uint16" />
            <el-option label="int32" value="int32" />
            <el-option label="uint32" value="uint32" />
            <el-option label="float32" value="float32" />
            <el-option label="float64" value="float64" />
            <el-option label="string" value="string" />
          </el-select>
        </el-form-item>
        <el-form-item label="字节序" prop="byte_order">
          <el-select v-model="form.byte_order" placeholder="选择字节序" style="width: 100%">
            <el-option label="AB (大端)" value="AB" />
            <el-option label="BA (小端)" value="BA" />
            <el-option label="ABCD" value="ABCD" />
            <el-option label="CDAB" value="CDAB" />
            <el-option label="BADC" value="BADC" />
            <el-option label="DCBA" value="DCBA" />
          </el-select>
        </el-form-item>
        <el-form-item label="系数" prop="coefficient">
          <el-input-number
            v-model="form.coefficient"
            :min="0"
            :precision="6"
            style="width: 100%"
            placeholder="请输入系数"
          />
        </el-form-item>
        <el-form-item label="单位" prop="unit">
          <el-input v-model="form.unit" placeholder="请输入单位" />
        </el-form-item>
        <el-form-item label="读写" prop="rw">
          <el-select v-model="form.rw" placeholder="选择读写属性" style="width: 100%">
            <el-option label="只读 (R)" value="R" />
            <el-option label="只写 (W)" value="W" />
            <el-option label="读写 (RW)" value="RW" />
          </el-select>
        </el-form-item>
        <el-form-item label="排序" prop="sort_order">
          <el-input-number v-model="form.sort_order" :min="0" style="width: 100%" />
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
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import type { UploadFile } from 'element-plus'
import { registerMapApi } from '@/api/register-map'
import { deviceModelApi } from '@/api/device-model'

const route = useRoute()
const router = useRouter()
const modelId = Number(route.params.modelId)
const modelName = ref('')

const loading = ref(false)
const tableData = ref<any[]>([])

const dialogVisible = ref(false)
const isEdit = ref(false)
const submitLoading = ref(false)
const formRef = ref<FormInstance>()
const form = reactive({
  id: null as number | null,
  sensor_code: '',
  sensor_name: '',
  register_address: 0,
  data_type: 'int16',
  byte_order: 'AB',
  coefficient: 1,
  unit: '',
  rw: 'R',
  sort_order: 0
})

const rules: FormRules = {
  sensor_code: [{ required: true, message: '请输入传感器编码', trigger: 'blur' }],
  sensor_name: [{ required: true, message: '请输入传感器名称', trigger: 'blur' }],
  register_address: [{ required: true, message: '请输入寄存器地址', trigger: 'blur' }],
  data_type: [{ required: true, message: '请选择数据类型', trigger: 'change' }],
  byte_order: [{ required: true, message: '请选择字节序', trigger: 'change' }],
  coefficient: [{ required: true, message: '请输入系数', trigger: 'blur' }],
  rw: [{ required: true, message: '请选择读写属性', trigger: 'change' }]
}

function toHex(addr: number): string {
  return '0x' + addr.toString(16).toUpperCase().padStart(4, '0')
}

async function loadModelName() {
  try {
    const res: any = await deviceModelApi.getById(modelId)
    modelName.value = res.data?.name ?? ''
  } catch {
    // ignore
  }
}

async function fetchList() {
  loading.value = true
  try {
    const res: any = await registerMapApi.listByModelId(modelId)
    tableData.value = Array.isArray(res.data) ? res.data : res.data?.records ?? []
  } finally {
    loading.value = false
  }
}

function openDialog(row?: any) {
  isEdit.value = !!row
  if (row) {
    form.id = row.id
    form.sensor_code = row.sensor_code
    form.sensor_name = row.sensor_name
    form.register_address = row.register_address
    form.data_type = row.data_type
    form.byte_order = row.byte_order
    form.coefficient = row.coefficient ?? 1
    form.unit = row.unit ?? ''
    form.rw = row.rw
    form.sort_order = row.sort_order ?? 0
  }
  dialogVisible.value = true
}

function resetForm() {
  formRef.value?.resetFields()
  form.id = null
  form.sensor_code = ''
  form.sensor_name = ''
  form.register_address = 0
  form.data_type = 'int16'
  form.byte_order = 'AB'
  form.coefficient = 1
  form.unit = ''
  form.rw = 'R'
  form.sort_order = 0
}

async function handleSubmit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return
  submitLoading.value = true
  try {
    const payload = {
      sensorCode: form.sensor_code,
      sensorName: form.sensor_name,
      registerAddress: form.register_address,
      dataType: form.data_type,
      byteOrder: form.byte_order,
      coefficient: form.coefficient,
      unit: form.unit,
      rw: form.rw,
      sortOrder: form.sort_order
    }
    if (isEdit.value && form.id) {
      await registerMapApi.update(modelId, form.id, payload)
      ElMessage.success('更新成功')
    } else {
      await registerMapApi.create(modelId, payload)
      ElMessage.success('创建成功')
    }
    dialogVisible.value = false
    fetchList()
  } finally {
    submitLoading.value = false
  }
}

async function handleDelete(row: any) {
  await ElMessageBox.confirm('确定删除该点位吗？', '提示', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  })
  await registerMapApi.remove(modelId, row.id)
  ElMessage.success('删除成功')
  fetchList()
}

async function handleImportFile(file: UploadFile) {
  const raw = file.raw
  if (!raw) return
  try {
    const text = await raw.text()
    const data = JSON.parse(text)
    const items = Array.isArray(data) ? data : [data]
    await registerMapApi.batchCreate(modelId, items)
    ElMessage.success(`成功导入 ${items.length} 条数据`)
    fetchList()
  } catch {
    ElMessage.error('文件解析失败，请检查 JSON 格式')
  }
}

function goBack() {
  router.push('/device-model')
}

onMounted(() => {
  loadModelName()
  fetchList()
})
</script>

<style scoped>
.registers-page {
  padding: 0;
}
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  flex-wrap: wrap;
  gap: 12px;
}
.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}
.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}
.page-title {
  font-size: 16px;
  font-weight: 500;
}
.register-address {
  display: flex;
  align-items: center;
  gap: 6px;
}
.addr-dec {
  color: #909399;
  font-size: 12px;
}
</style>
