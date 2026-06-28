<template>
  <div class="dashboard-page" v-loading="loading">
    <!-- Welcome Row -->
    <div class="welcome-row">
      <div>
        <h1 class="page-title">工作台</h1>
        <p class="page-subtitle">实时监控园区设备状态与采集数据</p>
      </div>
      <div class="refresh-info" v-if="overview">
        <span class="refresh-dot"></span>
        实时监控中
      </div>
    </div>

    <div class="onboarding-card" v-if="overview && overview.totalDevices === 0">
      <div class="onboarding-icon">🚀</div>
      <h2>欢迎使用 AZ-IOT 能源管理平台</h2>
      <p>开始使用只需 4 步，按顺序完成设备接入配置</p>
      <div class="onboarding-steps">
        <div class="onboarding-step">
          <span class="step-num">1</span>
          <span class="step-text">录入品牌 — 添加设备厂商信息</span>
        </div>
        <div class="onboarding-step">
          <span class="step-num">2</span>
          <span class="step-text">创建型号 — 定义设备型号并导入点表</span>
        </div>
        <div class="onboarding-step">
          <span class="step-num">3</span>
          <span class="step-text">注册设备 — 将物理设备接入系统</span>
        </div>
        <div class="onboarding-step">
          <span class="step-num">4</span>
          <span class="step-text">查看工作台 — 监控设备运行状态</span>
        </div>
      </div>
      <el-button type="primary" @click="$router.push('/manufacturer')">开始配置 →</el-button>
    </div>

    <!-- 指标卡片 -->
    <div class="metrics-row">
      <div class="metric-card metric-card--total">
        <div class="metric-icon">
          <svg viewBox="0 0 24 24" fill="none">
            <rect x="3" y="3" width="7" height="7" rx="1.5" stroke="currentColor" stroke-width="1.5"/>
            <rect x="14" y="3" width="7" height="7" rx="1.5" stroke="currentColor" stroke-width="1.5"/>
            <rect x="3" y="14" width="7" height="7" rx="1.5" stroke="currentColor" stroke-width="1.5"/>
            <rect x="14" y="14" width="7" height="7" rx="1.5" stroke="currentColor" stroke-width="1.5"/>
          </svg>
        </div>
        <div class="metric-body">
          <span class="metric-label">设备总数</span>
          <span class="metric-value">{{ overview?.totalDevices ?? 0 }}</span>
        </div>
      </div>

      <div class="metric-card metric-card--online">
        <div class="metric-icon">
          <svg viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5"/>
            <path d="M8 12l3 3 5-5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>
        <div class="metric-body">
          <span class="metric-label">
            在线设备
            <span class="online-rate" v-if="overview?.totalDevices">
              {{ percent(overview.onlineDevices, overview.totalDevices) }}%
            </span>
          </span>
          <span class="metric-value online">{{ overview?.onlineDevices ?? 0 }}</span>
        </div>
        <div class="metric-bar">
          <div class="metric-bar-fill online-fill" :style="{ width: barWidth(overview?.onlineDevices, overview?.totalDevices) }"></div>
        </div>
      </div>

      <div class="metric-card metric-card--readings">
        <div class="metric-icon">
          <svg viewBox="0 0 24 24" fill="none">
            <polyline points="3,17 9,11 13,15 21,7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            <circle cx="21" cy="7" r="1.5" fill="currentColor"/>
          </svg>
        </div>
        <div class="metric-body">
          <span class="metric-label">今日读数</span>
          <span class="metric-value primary">{{ formatNum(overview?.todayReadings ?? 0) }}</span>
        </div>
      </div>

      <div class="metric-card metric-card--alarm" :class="{ 'has-alarm': (overview?.alarms ?? 0) > 0 }">
        <div class="metric-icon">
          <svg viewBox="0 0 24 24" fill="none">
            <path d="M12 2L2 20h20L12 2z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
            <path d="M12 10v3M12 16.5v.01" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </div>
        <div class="metric-body">
          <span class="metric-label">活跃告警</span>
          <span class="metric-value alarm">{{ overview?.alarms ?? 0 }}</span>
        </div>
        <div class="alarm-pulse" v-if="(overview?.alarms ?? 0) > 0"></div>
      </div>
    </div>

    <!-- 数据可视化行 -->
    <div class="viz-row">
      <!-- 设备状态分布 -->
      <div class="content-block viz-block">
        <h3 class="section-heading">设备状态分布</h3>
        <div class="status-distribution" v-if="overview">
          <div class="status-ring">
            <svg viewBox="0 0 120 120" class="ring-svg">
              <circle cx="60" cy="60" r="52" fill="none" stroke="var(--color-gray-100)" stroke-width="10"/>
              <!-- Online arc -->
              <circle cx="60" cy="60" r="52" fill="none" :stroke="onlineColor" stroke-width="10"
                :stroke-dasharray="ringDash(onlinePct)" stroke-dashoffset="0"
                stroke-linecap="round" transform="rotate(-90 60 60)"
                class="ring-arc"/>
              <!-- Offline arc -->
              <circle cx="60" cy="60" r="52" fill="none" :stroke="offlineColor" stroke-width="10"
                :stroke-dasharray="ringDash(offlinePct)" :stroke-dashoffset="ringOffset(onlinePct)"
                stroke-linecap="round" transform="rotate(-90 60 60)"
                class="ring-arc"/>
              <!-- Alarm arc -->
              <circle cx="60" cy="60" r="52" fill="none" :stroke="alarmColor" stroke-width="10"
                :stroke-dasharray="ringDash(alarmPct)" :stroke-dashoffset="ringOffset(onlinePct + offlinePct)"
                stroke-linecap="round" transform="rotate(-90 60 60)"
                class="ring-arc"/>
            </svg>
            <div class="ring-center">
              <span class="ring-total">{{ overview.totalDevices ?? 0 }}</span>
              <span class="ring-label">总计</span>
            </div>
          </div>
          <div class="status-legend">
            <div class="legend-item">
              <span class="legend-dot online-dot"></span>
              <span class="legend-label">在线</span>
              <span class="legend-value">{{ overview.onlineDevices ?? 0 }}</span>
            </div>
            <div class="legend-item">
              <span class="legend-dot offline-dot"></span>
              <span class="legend-label">离线</span>
              <span class="legend-value">{{ (overview.totalDevices ?? 0) - (overview.onlineDevices ?? 0) - (overview.alarms ?? 0) }}</span>
            </div>
            <div class="legend-item">
              <span class="legend-dot alarm-dot"></span>
              <span class="legend-label">告警</span>
              <span class="legend-value">{{ overview.alarms ?? 0 }}</span>
            </div>
          </div>
        </div>
        <div v-else class="viz-empty">暂无数据</div>
      </div>

      <!-- 采集趋势（简化柱状图） -->
      <div class="content-block viz-block viz-chart">
        <h3 class="section-heading">今日采集趋势</h3>
        <div class="bar-chart" v-if="hourlyBars.length > 0">
          <div class="chart-bars">
            <div v-for="(bar, idx) in hourlyBars" :key="idx" class="bar-col">
              <div class="bar-fill" :style="{ height: bar.height + '%' }">
                <span class="bar-tooltip">{{ bar.count }}</span>
              </div>
              <span class="bar-label">{{ bar.hour }}</span>
            </div>
          </div>
        </div>
        <div v-else class="viz-empty">暂无趋势数据</div>
      </div>
    </div>

    <!-- 最近采集数据 -->
    <div class="content-block">
      <h3 class="section-heading">最近采集数据</h3>
      <el-table :data="recentReadings" stripe v-if="recentReadings.length > 0">
        <el-table-column prop="deviceName" label="设备" min-width="150">
          <template #default="{ row }">
            <span class="device-name-cell">{{ row.deviceName }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="sensorCode" label="点位" min-width="120" />
        <el-table-column prop="value" label="数值" min-width="140" align="right">
          <template #default="{ row }">
            <span class="reading-value">{{ formatValue(row.value) }}</span>
            <span v-if="row.unit" class="reading-unit">{{ row.unit }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="quality" label="质量" width="80" align="center">
          <template #default="{ row }">
            <span :class="['quality-badge', row.quality === 'good' ? 'good' : 'bad']">
              {{ row.quality === 'good' ? '正常' : '异常' }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="ts" label="采集时间" min-width="170">
          <template #default="{ row }">
            {{ formatTime(row.readAt ?? row.ts) }}
          </template>
        </el-table-column>
      </el-table>
      <div v-else class="empty-state">
        <svg viewBox="0 0 48 48" fill="none" class="empty-icon">
          <rect x="6" y="10" width="36" height="28" rx="3" stroke="currentColor" stroke-width="1.5"/>
          <line x1="6" y1="18" x2="42" y2="18" stroke="currentColor" stroke-width="1.5"/>
          <line x1="14" y1="10" x2="14" y2="38" stroke="currentColor" stroke-width="1.5"/>
        </svg>
        <p>暂无采集数据</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { dashboardApi } from '@/api/dashboard'

const loading = ref(false)
const overview = ref<any>(null)
const recentReadings = ref<any[]>([])
const hourlyBars = ref<any[]>([])

function formatValue(v: number): string {
  if (v == null) return '-'
  return Number(v).toFixed(3)
}

function formatNum(n: number): string {
  if (n == null) return '0'
  if (n >= 10000) return (n / 10000).toFixed(1) + 'w'
  return n.toLocaleString('zh-CN')
}

function formatTime(ts: string): string {
  if (!ts) return '-'
  try {
    return new Date(ts).toLocaleString('zh-CN', {
      month: '2-digit', day: '2-digit',
      hour: '2-digit', minute: '2-digit', second: '2-digit'
    })
  } catch { return ts }
}

function percent(part: number, total: number): number {
  if (!total) return 0
  return Math.round((part / total) * 100)
}

function barWidth(part: number, total: number): string {
  if (!total || !part) return '0%'
  return (part / total * 100).toFixed(0) + '%'
}

// Ring chart calculations
const onlinePct = computed(() => {
  if (!overview.value?.totalDevices) return 0
  return ((overview.value.onlineDevices ?? 0) / overview.value.totalDevices) * 100
})
const alarmPct = computed(() => {
  if (!overview.value?.totalDevices) return 0
  return ((overview.value.alarms ?? 0) / overview.value.totalDevices) * 100
})
const offlinePct = computed(() => {
  return Math.max(0, 100 - onlinePct.value - alarmPct.value)
})

const ringTotal = 2 * Math.PI * 52
const onlineColor = 'var(--color-success-500)'
const offlineColor = 'var(--color-gray-300)'
const alarmColor = 'var(--color-danger-500)'

function ringDash(pct: number): string {
  return ((pct / 100) * ringTotal).toFixed(1) + ' ' + ringTotal.toFixed(1)
}

function ringOffset(pct: number): string {
  return (-(pct / 100) * ringTotal).toFixed(1)
}

function generateHourlyBars(readings: any[]) {
  const counts = new Map<string, number>()
  readings.forEach(item => {
    const ts = item.readAt ?? item.ts
    if (!ts) return
    const date = new Date(ts)
    if (Number.isNaN(date.getTime())) return
    const hour = date.getHours().toString().padStart(2, '0') + ':00'
    counts.set(hour, (counts.get(hour) ?? 0) + 1)
  })
  const maxCount = Math.max(...Array.from(counts.values()), 1)

  return Array.from(counts.entries()).sort(([a], [b]) => a.localeCompare(b)).map(([hour, count]) => {
    return {
      hour,
      count,
      height: Math.round((count / maxCount) * 100)
    }
  })
}

onMounted(async () => {
  loading.value = true
  try {
    const res: any = await dashboardApi.overview()
    overview.value = res.data ?? {}
    recentReadings.value = res.data?.recentReadings ?? []
    hourlyBars.value = generateHourlyBars(recentReadings.value)
  } catch {
    overview.value = null
    recentReadings.value = []
    hourlyBars.value = []
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
.dashboard-page {
  animation: page-in 400ms var(--ease-out-expo);
}

@keyframes page-in {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}

/* ═══ Welcome Row ═══ */
.welcome-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--space-6);
}

.page-title {
  font-size: var(--text-2xl);
  font-weight: var(--font-weight-bold);
  color: var(--color-gray-800);
  margin: 0 0 var(--space-1);
  letter-spacing: var(--tracking-tight);
}

.page-subtitle {
  font-size: var(--text-sm);
  color: var(--color-gray-500);
  margin: 0;
}

.refresh-info {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--text-xs);
  color: var(--color-success-600);
  font-weight: var(--font-weight-medium);
  background: var(--color-success-50);
  padding: 4px 12px;
  border-radius: var(--radius-full);
}

.refresh-dot {
  width: 6px;
  height: 6px;
  background: var(--color-success-500);
  border-radius: 50%;
  animation: refresh-pulse 2s ease-in-out infinite;
}

@keyframes refresh-pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.6; transform: scale(1.5); }
}

/* ═══ Metric Cards ═══ */
.metrics-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--space-4);
  margin-bottom: var(--space-6);
}

.metric-card {
  background: var(--surface-primary);
  border: 1px solid var(--border-light);
  border-radius: var(--radius-lg);
  padding: var(--space-5);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  box-shadow: var(--shadow-xs);
  transition: all var(--duration-normal) var(--ease-out-quart);
  position: relative;
  overflow: hidden;
}

.metric-card:hover {
  box-shadow: var(--shadow-md);
  transform: translateY(-2px);
}

.metric-card::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 3px;
  background: var(--color-brand-500);
  transform: scaleX(0);
  transform-origin: left;
  transition: transform var(--duration-slow) var(--ease-out-expo);
}

.metric-card:hover::after {
  transform: scaleX(1);
}

.metric-card--total::after { background: var(--color-brand-500); }
.metric-card--online::after { background: var(--color-success-500); }
.metric-card--readings::after { background: var(--color-info-500); }
.metric-card--alarm::after { background: var(--color-danger-500); }

.metric-icon {
  width: 36px;
  height: 36px;
  color: var(--color-brand-500);
}

.metric-card--total .metric-icon { color: var(--color-brand-500); }
.metric-card--online .metric-icon { color: var(--color-success-500); }
.metric-card--readings .metric-icon { color: var(--color-info-500); }
.metric-card--alarm .metric-icon { color: var(--color-danger-500); }

.metric-body {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.metric-label {
  font-size: var(--text-xs);
  color: var(--color-gray-500);
  font-weight: var(--font-weight-medium);
  display: flex;
  align-items: center;
  gap: 6px;
}

.online-rate {
  background: var(--color-success-50);
  color: var(--color-success-600);
  padding: 1px 6px;
  border-radius: var(--radius-full);
  font-size: 11px;
}

.metric-value {
  font-size: var(--text-4xl);
  font-weight: var(--font-weight-bold);
  color: var(--color-gray-800);
  line-height: 1;
  font-variant-numeric: tabular-nums;
}

.metric-value.online { color: var(--color-success-600); }
.metric-value.primary { color: var(--color-brand-500); }
.metric-value.alarm { color: var(--color-danger-500); }

.metric-bar {
  height: 3px;
  background: var(--color-gray-100);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.metric-bar-fill {
  height: 100%;
  border-radius: var(--radius-full);
  transition: width 1s var(--ease-out-expo);
}

.online-fill { background: var(--color-success-500); }

/* Alarm Pulse Animation */
.alarm-pulse {
  position: absolute;
  top: var(--space-3);
  right: var(--space-3);
  width: 8px;
  height: 8px;
  background: var(--color-danger-500);
  border-radius: 50%;
  animation: alarm-pulse 2s ease-in-out infinite;
}

@keyframes alarm-pulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.4); }
  50% { box-shadow: 0 0 0 8px rgba(239, 68, 68, 0); }
}

.metric-card.has-alarm {
  border-color: var(--color-danger-200);
}

/* ═══ Viz Row ═══ */
.viz-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-4);
  margin-bottom: var(--space-6);
}

.viz-block {
  min-height: 280px;
}

/* ═══ Ring Chart ═══ */
.status-distribution {
  display: flex;
  align-items: center;
  gap: var(--space-8);
}

.status-ring {
  position: relative;
  width: 120px;
  height: 120px;
  flex-shrink: 0;
}

.ring-svg {
  width: 100%;
  height: 100%;
}

.ring-arc {
  transition: stroke-dasharray 1s var(--ease-out-expo);
}

.ring-center {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.ring-total {
  font-size: var(--text-xl);
  font-weight: var(--font-weight-bold);
  color: var(--color-gray-800);
  line-height: 1;
}

.ring-label {
  font-size: 11px;
  color: var(--color-gray-500);
  margin-top: 2px;
}

.status-legend {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  flex: 1;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.legend-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.online-dot { background: var(--color-success-500); }
.offline-dot { background: var(--color-gray-300); }
.alarm-dot { background: var(--color-danger-500); }

.legend-label {
  font-size: var(--text-sm);
  color: var(--color-gray-600);
  flex: 1;
}

.legend-value {
  font-size: var(--text-sm);
  font-weight: var(--font-weight-semibold);
  color: var(--color-gray-800);
}

/* ═══ Bar Chart ═══ */
.bar-chart {
  height: 200px;
  display: flex;
  align-items: flex-end;
}

.chart-bars {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  width: 100%;
  height: 100%;
  gap: 4px;
}

.bar-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  height: 100%;
  justify-content: flex-end;
}

.bar-fill {
  width: 100%;
  max-width: 32px;
  background: linear-gradient(to top, var(--color-brand-500), var(--color-brand-300));
  border-radius: var(--radius-sm) var(--radius-sm) 0 0;
  transition: height 800ms var(--ease-out-expo);
  position: relative;
  min-height: 4px;
}

.bar-fill:hover {
  filter: brightness(1.1);
}

.bar-tooltip {
  position: absolute;
  top: -22px;
  left: 50%;
  transform: translateX(-50%);
  font-size: 11px;
  color: var(--color-gray-600);
  font-weight: var(--font-weight-medium);
  opacity: 0;
  transition: opacity var(--duration-fast) var(--ease-out-quart);
  white-space: nowrap;
}

.bar-fill:hover .bar-tooltip {
  opacity: 1;
}

.bar-label {
  font-size: 10px;
  color: var(--color-gray-400);
  font-weight: var(--font-weight-medium);
  font-variant-numeric: tabular-nums;
}

/* ═══ Empty States ═══ */
.viz-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--color-gray-400);
  font-size: var(--text-sm);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-12) 0;
  color: var(--color-gray-400);
}

.empty-icon {
  width: 48px;
  height: 48px;
  margin-bottom: var(--space-4);
  color: var(--color-gray-300);
}

.empty-state p {
  margin: 0;
  font-size: var(--text-sm);
}

/* ═══ Table Cell Styles ═══ */
.device-name-cell {
  font-weight: var(--font-weight-medium);
  color: var(--color-gray-800);
}

.reading-value {
  font-weight: var(--font-weight-semibold);
  color: var(--color-gray-800);
  font-variant-numeric: tabular-nums;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}

.reading-unit {
  font-size: var(--text-xs);
  color: var(--color-gray-500);
  margin-left: 4px;
}

.quality-badge {
  font-size: 11px;
  font-weight: var(--font-weight-semibold);
  padding: 2px 10px;
  border-radius: var(--radius-full);
}

.quality-badge.good {
  color: var(--color-success-600);
  background: var(--color-success-50);
}

.quality-badge.bad {
  color: var(--color-danger-600);
  background: var(--color-danger-50);
}

/* ═══ Responsive ═══ */
@media (max-width: 1024px) {
  .metrics-row {
    grid-template-columns: repeat(2, 1fr);
  }

  .viz-row {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 640px) {
  .metrics-row {
    grid-template-columns: 1fr;
  }

  .status-distribution {
    flex-direction: column;
    align-items: center;
  }
}

.onboarding-card {
  padding: var(--space-8);
  background: var(--surface-primary);
  border: 1px solid var(--border-light);
  border-radius: var(--radius-xl);
  text-align: center;
  margin-bottom: var(--space-6);
}
.onboarding-icon { font-size: 36px; margin-bottom: var(--space-4); }
.onboarding-card h2 { font-size: var(--text-xl); font-weight: var(--font-weight-semibold); color: var(--color-gray-800); margin: 0 0 var(--space-2); }
.onboarding-card p { font-size: var(--text-sm); color: var(--color-gray-500); margin: 0 0 var(--space-6); }
.onboarding-steps { display: flex; flex-direction: column; gap: var(--space-3); max-width: 400px; margin: 0 auto var(--space-6); text-align: left; }
.onboarding-step { display: flex; align-items: center; gap: var(--space-3); padding: var(--space-2) var(--space-3); background: var(--color-gray-50); border-radius: var(--radius-md); }
.step-num { display: inline-flex; align-items: center; justify-content: center; width: 24px; height: 24px; border-radius: 50%; background: var(--color-brand-500); color: #fff; font-size: 12px; font-weight: var(--font-weight-semibold); flex-shrink: 0; }
.step-text { font-size: var(--text-sm); color: var(--color-gray-700); }
</style>
