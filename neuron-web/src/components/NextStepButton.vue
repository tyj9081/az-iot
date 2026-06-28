<template>
  <div class="next-step-row">
    <el-button type="primary" plain size="small" :icon="ArrowRight" @click="navigate">
      下一步：{{ label }}
    </el-button>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { ArrowRight } from '@element-plus/icons-vue'

const props = defineProps<{ to: string; label: string }>()
const router = useRouter()
const navigate = () => router.push(props.to)
</script>

<style scoped>
.next-step-row { display: flex; justify-content: flex-end; margin-top: var(--space-4); padding-top: var(--space-3); border-top: 1px solid var(--border-light); }
</style>
