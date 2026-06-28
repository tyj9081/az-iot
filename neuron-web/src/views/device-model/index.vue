<template>
  <div class="device-model-page page-container">
    <div class="page-toolbar">
      <div class="filter-group">
        <el-select v-model="filterManufacturerId" placeholder="选择厂商" clearable filterable style="width: 180px" @change="handleSearch">
          <el-option v-for="m in manufacturerList" :key="m.id" :label="m.name" :value="m.id" />
        </el-select>
        <el-select v-model="filterProtocolId" placeholder="选择协议" clearable filterable style="width: 160px" @change="handleSearch">
          <el-option v-for="p in protocolList" :key="p.id" :label="p.name" :value="p.id" />
        </el-select>
        <el-input v-model="searchKeyword" placeholder="搜索型号编码/名称" clearable style="width: 220px" @keyup.enter="handleSearch">
          <template #prefix><el-icon><Search /></el-icon></template>
        </el-input>
      </div>
      <el-button type="primary" @click="openDialog()">新增型号</el-button>
    </div>

    <el-table v-loading="loading" :data="tableData" stripe>
      <el-table-column prop="code" label="编码" min-width="120" />
      <el-table-column prop="name" label="名称" min-width="150" />
      <el-table-column label="厂商" min-width="120">
        <template #default="{ row }">{{ manufacturerList.find((m: any) => m.id === row.manufacturerId)?.name ?? '-' }}</template>
      </el-table-column>
      <el-table-column label="协议" min-width="100">
        <template #default="{ row }">{{ protocolList.find((p: any) => p.id === row.protocolId)?.name ?? '-' }}</template>
      </el-table-column>
      <el-table-column prop="createdAt" label="创建时间" min-width="160" />
      <el-table-column label="操作" width="220" fixed="right" align="center">
        <template #default="{ row }">
          <el-button link type="primary" size="small" @click="openDialog(row)">编辑</el-button>
          <el-button link type="danger" size="small" @click="handleDelete(row)">删除</el-button>
          <el-button link type="success" size="small" @click="goRegisters(row)">点表管理</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-pagination
      v-model:current-page="page" v-model:page-size="pageSize" :total="total"
      :page-sizes="[10, 20, 50]" layout="total, sizes, prev, pager, next, jumper"
      @current-change="fetchList" @size-change="fetchList"
    />

    <NextStepButton to="/device" label="注册设备" />

    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑型号' : '新增型号'" width="520px" :close-on-click-modal="false" @closed="resetForm">
      <el-form ref="formRef" :model="form" :rules="rules" label-width="100px">
        <el-form-item label="厂商" prop="manufacturer_id">
          <el-select v-model="form.manufacturer_id" placeholder="选择厂商" filterable style="width: 100%">
            <el-option v-for="m in manufacturerList" :key="m.id" :label="m.name" :value="m.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="协议" prop="protocol_id">
          <el-select v-model="form.protocol_id" placeholder="选择协议" filterable style="width: 100%">
            <el-option v-for="p in protocolList" :key="p.id" :label="p.name" :value="p.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="编码" prop="code"><el-input v-model="form.code" placeholder="请输入编码" /></el-form-item>
        <el-form-item label="名称" prop="name"><el-input v-model="form.name" placeholder="请输入名称" /></el-form-item>
        <el-form-item label="描述" prop="description">
          <el-input v-model="form.description" type="textarea" :rows="2" placeholder="请输入描述" />
        </el-form-item>

        <el-divider content-position="left">寄存器点表</el-divider>
        <el-form-item label="导入方式">
          <el-radio-group v-model="importMode" @change="onImportModeChange">
            <el-radio :value="'later'">稍后配置</el-radio>
            <el-radio :value="'now'" :disabled="isEdit">立即导入</el-radio>
          </el-radio-group>
        </el-form-item>

        <template v-if="importMode === 'now'">
          <el-form-item label="导入方式">
            <el-radio-group v-model="importTab" size="small">
              <el-radio-button :value="'json'">JSON 粘贴</el-radio-button>
              <el-radio-button :value="'file'">上传文件</el-radio-button>
            </el-radio-group>
          </el-form-item>

          <el-form-item v-if="importTab === 'json'" label="点位JSON">
            <el-input v-model="jsonText" type="textarea" :rows="6"
              placeholder='粘贴传感器点位 JSON数组，例如：[{"sensorCode":"humidity","sensorName":"湿度",...}]' />
          </el-form-item>

          <el-form-item v-if="importTab === 'file'" label="点位文件">
            <el-upload :auto-upload="false" :limit="1" accept=".json,.csv"
              :on-change="onFileChange" :on-remove="onFileRemove" drag>
              <el-icon style="font-size:28px"><Upload /></el-icon>
              <div style="margin-top:8px;font-size:13px;color:var(--color-gray-500)">拖拽或点击上传 .json / .csv 文件</div>
            </el-upload>
          </el-form-item>

          <el-form-item v-if="previewRegisters.length" label="预览">
            <div class="reg-preview">
              <el-tag v-for="(reg, idx) in previewRegisters" :key="idx" size="small" type="info"
                style="margin:2px">
                {{ reg.sensorCode }}: {{ reg.sensorName }}
                <template v-if="reg.unit">({{ reg.unit }})</template>
              </el-tag>
              <span class="reg-count">共 {{ previewRegisters.length }} 条点位</span>
            </div>
          </el-form-item>
        </template>
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
import { Search, Upload } from '@element-plus/icons-vue'
import type { FormInstance, FormRules, UploadFile } from 'element-plus'
import { deviceModelApi } from '@/api/device-model'
import { manufacturerApi } from '@/api/manufacturer'
import { protocolApi } from '@/api/protocol'
import { registerMapApi } from '@/api/register-map'
import NextStepButton from '@/components/NextStepButton.vue'

const router = useRouter()
const manufacturerList = ref<any[]>([]); const protocolList = ref<any[]>([])
const filterManufacturerId = ref<number | ''>(''); const filterProtocolId = ref<number | ''>('')
const searchKeyword = ref(''); const loading = ref(false); const tableData = ref<any[]>([])
const page = ref(1); const pageSize = ref(10); const total = ref(0)
const dialogVisible = ref(false); const isEdit = ref(false); const submitLoading = ref(false)
const formRef = ref<FormInstance>()
const form = reactive({ id: null as number | null, manufacturer_id: null as number | null, protocol_id: null as number | null, code: '', name: '', description: '' })
const rules: FormRules = {
  manufacturer_id: [{ required: true, message: '请选择厂商', trigger: 'change' }],
  protocol_id: [{ required: true, message: '请选择协议', trigger: 'change' }],
  code: [{ required: true, message: '请输入编码', trigger: 'blur' }],
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }]
}

// 点表导入
const importMode = ref<'later' | 'now'>('later')
const importTab = ref<'json' | 'file'>('json')
const jsonText = ref('')
const uploadedRegisters = ref<any[]>([])
const previewRegisters = computed(() => {
  if (importTab.value === 'json' && jsonText.value.trim()) {
    try { const parsed = JSON.parse(jsonText.value); return Array.isArray(parsed) ? parsed : [] }
    catch { return [] }
  }
  return uploadedRegisters.value
})

async function loadManufacturers() { const res: any = await manufacturerApi.list({ page: 1, pageSize: 9999 }); manufacturerList.value = res.data?.records ?? [] }
async function loadProtocols() { const res: any = await protocolApi.listAll(); protocolList.value = Array.isArray(res.data) ? res.data : res.data?.records ?? [] }
async function fetchList() {
  loading.value = true
  try {
    const params: any = { page: page.value, pageSize: pageSize.value }
    if (filterManufacturerId.value) params.manufacturerId = filterManufacturerId.value
    if (filterProtocolId.value) params.protocolId = filterProtocolId.value
    if (searchKeyword.value) params.keyword = searchKeyword.value
    const res: any = await deviceModelApi.list(params)
    tableData.value = res.data?.records ?? []; total.value = res.data?.total ?? 0
  } finally { loading.value = false }
}
function handleSearch() { page.value = 1; fetchList() }
function openDialog(row?: any) {
  isEdit.value = !!row
  if (row) {
    form.id = row.id
    form.manufacturer_id = row.manufacturerId ?? row.manufacturer_id ?? null
    form.protocol_id = row.protocolId ?? row.protocol_id ?? null
    form.code = row.code
    form.name = row.name
    form.description = row.description ?? ''
  }
  dialogVisible.value = true
}
function resetForm() { formRef.value?.resetFields(); form.id = null; form.manufacturer_id = null; form.protocol_id = null; form.code = ''; form.name = ''; form.description = ''; importMode.value = 'later'; importTab.value = 'json'; jsonText.value = ''; uploadedRegisters.value = [] }

function onImportModeChange(_val: string) { jsonText.value = ''; uploadedRegisters.value = [] }

function onFileChange(file: UploadFile) {
  const reader = new FileReader()
  reader.onload = (e) => {
    try {
      const text = e.target?.result as string
      const data = JSON.parse(text)
      uploadedRegisters.value = Array.isArray(data) ? data : []
      jsonText.value = JSON.stringify(uploadedRegisters.value, null, 2)
    } catch {
      ElMessage.warning('文件解析失败，请检查 JSON 格式')
      uploadedRegisters.value = []
    }
  }
  reader.readAsText(file.raw as Blob)
}

function onFileRemove() { uploadedRegisters.value = [] }
async function handleSubmit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return
  submitLoading.value = true
  try {
    const payload = { manufacturerId: form.manufacturer_id, protocolId: form.protocol_id, code: form.code, name: form.name, description: form.description }
    if (isEdit.value && form.id) { await deviceModelApi.update(form.id, payload); ElMessage.success('更新成功') }
    else {
      const res: any = await deviceModelApi.create(payload)
      const modelId = res.data?.id
      ElMessage.success('创建成功')
      if (importMode.value === 'now' && modelId && previewRegisters.value.length) {
        try {
          await registerMapApi.batchCreate(modelId, previewRegisters.value)
          ElMessage.success(`已导入 ${previewRegisters.value.length} 条点位`)
        } catch (e: any) {
          ElMessage.warning('型号已创建，但点表导入失败: ' + (e?.message ?? ''))
        }
      }
    }
    dialogVisible.value = false; fetchList()
  } finally { submitLoading.value = false }
}
async function handleDelete(row: any) {
  await ElMessageBox.confirm('确定删除该型号吗？', '提示', { confirmButtonText: '确定', cancelButtonText: '取消', type: 'warning' })
  await deviceModelApi.remove(row.id); ElMessage.success('删除成功'); fetchList()
}
function goRegisters(row: any) { router.push(`/device-model/${row.id}/registers`) }
onMounted(() => { loadManufacturers(); loadProtocols(); fetchList() })
</script>

<style scoped>
.reg-preview { display: flex; flex-wrap: wrap; align-items: center; gap: 4px; padding: 8px 12px; background: var(--color-gray-50); border-radius: var(--radius-md); border: 1px solid var(--border-light); }
.reg-count { font-size: 12px; color: var(--color-gray-400); margin-left: auto; padding-left: 12px; white-space: nowrap; }
</style>
