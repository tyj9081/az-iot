<template>
  <div class="device-model-page">
    <div class="page-header">
      <div class="filter-bar">
        <el-select
          v-model="filterManufacturerId"
          placeholder="选择厂商"
          clearable
          filterable
          style="width: 180px"
        >
          <el-option
            v-for="m in manufacturerList"
            :key="m.id"
            :label="m.name"
            :value="m.id"
          />
        </el-select>
        <el-select
          v-model="filterProtocolId"
          placeholder="选择协议"
          clearable
          filterable
          style="width: 160px; margin-left: 12px"
        >
          <el-option
            v-for="p in protocolList"
            :key="p.id"
            :label="p.name"
            :value="p.id"
          />
        </el-select>
        <el-input
          v-model="searchKeyword"
          placeholder="搜索型号编码/名称"
          clearable
          style="width: 220px; margin-left: 12px"
          @keyup.enter="handleSearch"
        >
          <template #prefix><el-icon><Search /></el-icon></template>
        </el-input>
      </div>
      <el-button type="primary" @click="openDialog()">新增型号</el-button>
    </div>

    <el-table v-loading="loading" :data="tableData" border stripe>
      <el-table-column prop="code" label="编码" min-width="120" />
      <el-table-column prop="name" label="名称" min-width="150" />
      <el-table-column prop="manufacturer_name" label="厂商" min-width="120" />
      <el-table-column prop="protocol_name" label="协议" min-width="100" />
      <el-table-column prop="created_at" label="创建时间" min-width="160" />
      <el-table-column label="操作" width="220" fixed="right">
        <template #default="{ row }">
          <el-button link type="primary" size="small" @click="openDialog(row)">编辑</el-button>
          <el-button link type="danger" size="small" @click="handleDelete(row)">删除</el-button>
          <el-button link type="success" size="small" @click="goRegisters(row)">点表管理</el-button>
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
      :title="isEdit ? '编辑型号' : '新增型号'"
      width="520px"
      :close-on-click-modal="false"
      @closed="resetForm"
    >
      <el-form ref="formRef" :model="form" :rules="rules" label-width="100px">
        <el-form-item label="厂商" prop="manufacturer_id">
          <el-select
            v-model="form.manufacturer_id"
            placeholder="选择厂商"
            filterable
            style="width: 100%"
          >
            <el-option
              v-for="m in manufacturerList"
              :key="m.id"
              :label="m.name"
              :value="m.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="协议" prop="protocol_id">
          <el-select
            v-model="form.protocol_id"
            placeholder="选择协议"
            filterable
            style="width: 100%"
          >
            <el-option
              v-for="p in protocolList"
              :key="p.id"
              :label="p.name"
              :value="p.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="编码" prop="code">
          <el-input v-model="form.code" placeholder="请输入编码" />
        </el-form-item>
        <el-form-item label="名称" prop="name">
          <el-input v-model="form.name" placeholder="请输入名称" />
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
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Search } from '@element-plus/icons-vue'
import type { FormInstance, FormRules } from 'element-plus'
import { deviceModelApi } from '@/api/device-model'
import { manufacturerApi } from '@/api/manufacturer'
import { protocolApi } from '@/api/protocol'

const router = useRouter()

const manufacturerList = ref<any[]>([])
const protocolList = ref<any[]>([])

const filterManufacturerId = ref<number | ''>('')
const filterProtocolId = ref<number | ''>('')
const searchKeyword = ref('')
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
  manufacturer_id: null as number | null,
  protocol_id: null as number | null,
  code: '',
  name: '',
  description: ''
})

const rules: FormRules = {
  manufacturer_id: [{ required: true, message: '请选择厂商', trigger: 'change' }],
  protocol_id: [{ required: true, message: '请选择协议', trigger: 'change' }],
  code: [{ required: true, message: '请输入编码', trigger: 'blur' }],
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }]
}

async function loadManufacturers() {
  const res: any = await manufacturerApi.list({ page: 1, pageSize: 9999 })
  manufacturerList.value = res.data?.records ?? []
}

async function loadProtocols() {
  const res: any = await protocolApi.listAll()
  protocolList.value = Array.isArray(res.data) ? res.data : res.data?.records ?? []
}

async function fetchList() {
  loading.value = true
  try {
    const params: any = {
      page: page.value,
      pageSize: pageSize.value
    }
    if (filterManufacturerId.value) params.manufacturerId = filterManufacturerId.value
    if (filterProtocolId.value) params.protocolId = filterProtocolId.value
    if (searchKeyword.value) params.keyword = searchKeyword.value
    const res: any = await deviceModelApi.list(params)
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

function openDialog(row?: any) {
  isEdit.value = !!row
  if (row) {
    form.id = row.id
    form.manufacturer_id = row.manufacturer_id
    form.protocol_id = row.protocol_id
    form.code = row.code
    form.name = row.name
    form.description = row.description ?? ''
  }
  dialogVisible.value = true
}

function resetForm() {
  formRef.value?.resetFields()
  form.id = null
  form.manufacturer_id = null
  form.protocol_id = null
  form.code = ''
  form.name = ''
  form.description = ''
}

async function handleSubmit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return
  submitLoading.value = true
  try {
    const payload = {
      manufacturerId: form.manufacturer_id,
      protocolId: form.protocol_id,
      code: form.code,
      name: form.name,
      description: form.description
    }
    if (isEdit.value && form.id) {
      await deviceModelApi.update(form.id, payload)
      ElMessage.success('更新成功')
    } else {
      await deviceModelApi.create(payload)
      ElMessage.success('创建成功')
    }
    dialogVisible.value = false
    fetchList()
  } finally {
    submitLoading.value = false
  }
}

async function handleDelete(row: any) {
  await ElMessageBox.confirm('确定删除该型号吗？', '提示', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  })
  await deviceModelApi.remove(row.id)
  ElMessage.success('删除成功')
  fetchList()
}

function goRegisters(row: any) {
  router.push(`/device-model/${row.id}/registers`)
}

onMounted(() => {
  loadManufacturers()
  loadProtocols()
  fetchList()
})
</script>

<style scoped>
.device-model-page {
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
.filter-bar {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
}
</style>
