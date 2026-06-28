<template>
  <el-form label-width="80px" label-position="left">
    <el-form-item label="波特率">
      <el-select v-model="local.baud" style="width:100%">
        <el-option v-for="v in baudOptions" :key="v" :label="String(v)" :value="v" />
      </el-select>
    </el-form-item>
    <el-form-item label="数据位">
      <el-select v-model="local.data_bits" style="width:100%">
        <el-option v-for="v in [7,8]" :key="v" :label="String(v)" :value="v" />
      </el-select>
    </el-form-item>
    <el-form-item label="停止位">
      <el-select v-model="local.stop_bits" style="width:100%">
        <el-option v-for="v in [1,2]" :key="v" :label="String(v)" :value="v" />
      </el-select>
    </el-form-item>
    <el-form-item label="校验位">
      <el-select v-model="local.parity" style="width:100%">
        <el-option label="无" value="none" />
        <el-option label="偶校验" value="even" />
        <el-option label="奇校验" value="odd" />
      </el-select>
    </el-form-item>
  </el-form>
</template>

<script setup lang="ts">
import { reactive, watch, onMounted } from 'vue'

const props = defineProps<{ modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

const baudOptions = [1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200]

const local = reactive({ baud: 9600, data_bits: 8, stop_bits: 1, parity: 'none' })

function parseParam(raw: string) {
  try {
    const obj = JSON.parse(raw)
    if (typeof obj.baud === 'number') local.baud = obj.baud
    if (typeof obj.data_bits === 'number') local.data_bits = obj.data_bits
    if (typeof obj.stop_bits === 'number') local.stop_bits = obj.stop_bits
    if (typeof obj.parity === 'string') local.parity = obj.parity
  } catch { /* keep defaults */ }
}

function sync() {
  emit('update:modelValue', JSON.stringify({ baud: local.baud, data_bits: local.data_bits, stop_bits: local.stop_bits, parity: local.parity }))
}

onMounted(() => parseParam(props.modelValue))
watch(() => props.modelValue, (v) => parseParam(v))
watch(() => local, () => sync(), { deep: true })
</script>
