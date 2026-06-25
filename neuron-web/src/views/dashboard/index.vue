<template>
  <div class="dashboard-page" v-loading="loading">
    <!-- 指标卡片 -->
    <div class="metrics-grid">
      <div class="metric-card">
        <p class="metric-label">设备总数</p>
        <p class="metric-value">{{ overview?.totalDevices ?? 0 }}</p>
      </div>
      <div class="metric-card">
        <p class="metric-label">在线设备</p>
        <p class="metric-value online">{{ overview?.onlineDevices ?? 0 }}</p>
      </div>
      <div class="metric-card">
        <p class="metric-label">今日读数</p>
        <p class="metric-value primary">{{ overview?.todayReadings ?? 0 }}</p>
      </div>
      <div class="metric-card">
        <p class="metric-label">告警</p>
        <p class="metric-value alarm">{{ overview?.alarms ?? 0 }}</p>
      </div>
    </div>

    <!-- 最近采集数据表格 -->
    <div class="recent-section">
      <h3 class="section-title">最近采集数据</h3>
      <el-table :data="recentReadings" border stripe>
        <el-table-column prop="deviceName" label="设备" min-width="140" />
        <el-table-column prop="sensorCode" label="点位" min-width="120" />
        <el-table-column prop="value" label="数值" min-width="120">
          <template #default="{ row }">
            {{ formatValue(row.value) }}
            <span v-if="row.unit" class="unit-text">{{ row.unit }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="quality" label="质量" width="80">
          <template #default="{ row }">
            <span :class="['quality-tag', row.quality === 'good' ? 'good' : 'bad']">
              {{ row.quality === 'good' ? '正常' : '异常' }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="ts" label="时间" min-width="160">
          <template #default="{ row }">
            {{ formatTime(row.ts) }}
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { dashboardApi } from '@/api/dashboard'

const loading = ref(false)
const overview = ref<any>(null)
const recentReadings = ref<any[]>([])

function formatValue(v: number): string {
  if (v == null) return '-'
  return Number(v).toFixed(3)
}

function formatTime(ts: string): string {
  if (!ts) return '-'
  try {
    return new Date(ts).toLocaleString('zh-CN')
  } catch {
    return ts
  }
}

onMounted(async () => {
  loading.value = true
  try {
    const res: any = await dashboardApi.overview()
    overview.value = res ?? {}
    recentReadings.value = (res as any)?.recentReadings ?? []
  } catch {
    overview.value = null
    recentReadings.value = []
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
.dashboard-page {
  padding: 0;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.metric-card {
  background: #fff;
  border: 0.5px solid #eee;
  border-radius: 8px;
  padding: 20px;
}

.metric-label {
  font-size: 13px;
  color: #888;
  margin: 0 0 8px;
}

.metric-value {
  font-size: 32px;
  font-weight: 600;
  margin: 0;
  color: #333;
}

.metric-value.online {
  color: #1D9E75;
}

.metric-value.primary {
  color: #534AB7;
}

.metric-value.alarm {
  color: #E24B4A;
}

.recent-section {
  background: #fff;
  border: 0.5px solid #eee;
  border-radius: 8px;
  padding: 20px;
}

.section-title {
  font-size: 15px;
  font-weight: 600;
  margin: 0 0 16px;
  color: #333;
}

.unit-text {
  font-size: 12px;
  color: #888;
  margin-left: 4px;
}

.quality-tag {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
}

.quality-tag.good {
  color: #1D9E75;
  background: rgba(29, 158, 117, 0.1);
}

.quality-tag.bad {
  color: #E24B4A;
  background: rgba(226, 75, 74, 0.1);
}
</style>
