<template>
  <div class="manufacturer-page">
    <div class="page-header">
      <el-input
        v-model="searchKeyword"
        placeholder="搜索品牌名称/编码"
        clearable
        style="width: 260px"
        @keyup.enter="handleSearch"
      >
        <template #prefix><el-icon><Search /></el-icon></template>
      </el-input>
      <el-button type="primary" @click="openDialog()">新增品牌</el-button>
    </div>

    <el-table v-loading="loading" :data="tableData" border stripe>
      <el-table-column prop="code" label="编码" min-width="120" />
      <el-table-column prop="name" label="名称" min-width="150" />
      <el-table-column prop="country" label="国家" min-width="100" />
      <el-table-column prop="website" label="官网" min-width="180">
        <template #default="{ row }">
          <a v-if="row.website" :href="row.website" target="_blank">{{ row.website }}</a>
        </template>
      </el-table-column>
      <el-table-column prop="created_at" label="创建时间" min-width="160" />
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
      :title="isEdit ? '编辑品牌' : '新增品牌'"
      width="520px"
      :close-on-click-modal="false"
      @closed="resetForm"
    >
      <el-form ref="formRef" :model="form" :rules="rules" label-width="80px">
        <el-form-item label="编码" prop="code">
          <el-input v-model="form.code" placeholder="请输入编码" />
        </el-form-item>
        <el-form-item label="名称" prop="name">
          <el-input v-model="form.name" placeholder="请输入名称" />
        </el-form-item>
        <el-form-item label="国家" prop="country">
          <el-input v-model="form.country" placeholder="请输入国家" />
        </el-form-item>
        <el-form-item label="官网" prop="website">
          <el-input v-model="form.website" placeholder="请输入官网地址" />
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
import { manufacturerApi } from '@/api/manufacturer'

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
  code: '',
  name: '',
  country: '',
  website: '',
  description: ''
})

const rules: FormRules = {
  code: [{ required: true, message: '请输入编码', trigger: 'blur' }],
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }]
}

async function fetchList() {
  loading.value = true
  try {
    const res: any = await manufacturerApi.list({
      page: page.value,
      pageSize: pageSize.value,
      keyword: searchKeyword.value || undefined
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

function openDialog(row?: any) {
  isEdit.value = !!row
  if (row) {
    form.id = row.id
    form.code = row.code
    form.name = row.name
    form.country = row.country ?? ''
    form.website = row.website ?? ''
    form.description = row.description ?? ''
  }
  dialogVisible.value = true
}

function resetForm() {
  formRef.value?.resetFields()
  form.id = null
  form.code = ''
  form.name = ''
  form.country = ''
  form.website = ''
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
      country: form.country,
      website: form.website,
      description: form.description
    }
    if (isEdit.value && form.id) {
      await manufacturerApi.update(form.id, payload)
      ElMessage.success('更新成功')
    } else {
      await manufacturerApi.create(payload)
      ElMessage.success('创建成功')
    }
    dialogVisible.value = false
    fetchList()
  } finally {
    submitLoading.value = false
  }
}

async function handleDelete(row: any) {
  await ElMessageBox.confirm('确定删除该品牌吗？', '提示', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  })
  await manufacturerApi.remove(row.id)
  ElMessage.success('删除成功')
  fetchList()
}

onMounted(() => {
  fetchList()
})
</script>

<style scoped>
.manufacturer-page {
  padding: 0;
}
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}
</style>
