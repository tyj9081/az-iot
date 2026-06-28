<template>
  <div class="app-shell">
    <!-- 顶部导航栏 -->
    <header class="app-header">
      <div class="header-brand">
        <div class="brand-mark">
          <svg viewBox="0 0 32 32" fill="none" class="brand-icon">
            <rect width="32" height="32" rx="8" fill="#534AB7"/>
            <path d="M16 6L26 11v10L16 26 6 21V11L16 6z" stroke="#fff" stroke-width="1.5" fill="none" opacity="0.8"/>
            <circle cx="16" cy="16" r="4" fill="#fff" opacity="0.9"/>
            <circle cx="16" cy="16" r="1.5" fill="#534AB7"/>
          </svg>
        </div>
        <span class="brand-name">AZ-IOT</span>
        <span class="brand-subtitle">能源管理平台</span>
      </div>
      <div class="header-actions">
        <div class="header-status">
          <span class="status-indicator online">系统运行中</span>
        </div>
        <button class="btn-logout" @click="logout" title="退出登录">
          <svg viewBox="0 0 20 20" fill="none" class="logout-icon">
            <path d="M7 17H4a1 1 0 01-1-1V4a1 1 0 011-1h3m6 10l4-4-4-4m4 4H9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          <span>退出</span>
        </button>
      </div>
    </header>

    <!-- 主体区域 -->
    <div class="app-body">
      <!-- 侧栏导航 -->
      <aside class="app-sidebar">
        <nav class="sidebar-nav">
          <div class="nav-group">
            <div class="nav-group-label">设备接入</div>
            <router-link to="/manufacturer" class="nav-item nav-item--child" active-class="active">
              <span class="nav-step-badge">1</span>
              <svg viewBox="0 0 20 20" fill="none" class="nav-icon">
                <path d="M3 7l7-4 7 4v8l-7 4-7-4V7z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
              </svg>
              <span>品牌管理</span>
            </router-link>
            <router-link to="/device-model" class="nav-item nav-item--child" active-class="active">
              <span class="nav-step-badge">2</span>
              <svg viewBox="0 0 20 20" fill="none" class="nav-icon">
                <rect x="3" y="4" width="14" height="12" rx="2" stroke="currentColor" stroke-width="1.5"/>
                <line x1="7" y1="4" x2="7" y2="16" stroke="currentColor" stroke-width="1.5"/>
                <line x1="3" y1="10" x2="17" y2="10" stroke="currentColor" stroke-width="1.5"/>
              </svg>
              <span>型号管理</span>
            </router-link>
            <router-link to="/collector" class="nav-item nav-item--child" active-class="active">
              <span class="nav-step-badge">3</span>
              <svg viewBox="0 0 20 20" fill="none" class="nav-icon">
                <rect x="2" y="3" width="16" height="5" rx="1.5" stroke="currentColor" stroke-width="1.5"/>
                <rect x="4" y="8" width="3" height="3" rx="1" stroke="currentColor" stroke-width="1.5"/>
                <rect x="13" y="8" width="3" height="3" rx="1" stroke="currentColor" stroke-width="1.5"/>
                <rect x="4" y="14" width="3" height="3" rx="1" stroke="currentColor" stroke-width="1.5"/>
                <rect x="13" y="14" width="3" height="3" rx="1" stroke="currentColor" stroke-width="1.5"/>
              </svg>
              <span>采集器</span>
            </router-link>
            <router-link to="/device" class="nav-item nav-item--child" active-class="active">
              <span class="nav-step-badge">4</span>
              <svg viewBox="0 0 20 20" fill="none" class="nav-icon">
                <rect x="4" y="2" width="12" height="14" rx="2" stroke="currentColor" stroke-width="1.5"/>
                <circle cx="10" cy="11" r="2" stroke="currentColor" stroke-width="1.5"/>
                <path d="M10 18v-3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
              <span>设备管理</span>
            </router-link>
          </div>

          <div class="nav-group">
            <div class="nav-group-label">运行监控</div>
            <router-link to="/dashboard" class="nav-item nav-item--child" active-class="active">
              <span class="nav-step-badge">5</span>
              <svg viewBox="0 0 20 20" fill="none" class="nav-icon">
                <rect x="2" y="2" width="7" height="7" rx="1.5" stroke="currentColor" stroke-width="1.5"/>
                <rect x="11" y="2" width="7" height="7" rx="1.5" stroke="currentColor" stroke-width="1.5"/>
                <rect x="2" y="11" width="7" height="7" rx="1.5" stroke="currentColor" stroke-width="1.5"/>
                <rect x="11" y="11" width="7" height="7" rx="1.5" stroke="currentColor" stroke-width="1.5"/>
              </svg>
              <span>工作台</span>
            </router-link>
          </div>
        </nav>

        <div class="sidebar-footer">
          <div class="version-tag">v0.1.0 · MVP</div>
        </div>
      </aside>

      <!-- 内容区 -->
      <main class="app-main">
        <router-view v-slot="{ Component }">
          <transition name="page" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()

const logout = () => {
  authStore.clearAuth()
  localStorage.removeItem('refresh_token')
  router.push('/login')
}
</script>

<style scoped>
/* ═══ App Shell ═══ */
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-app);
}

/* ═══ Header ═══ */
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 56px;
  padding: 0 var(--space-6);
  background: var(--surface-primary);
  border-bottom: 1px solid var(--border-light);
  box-shadow: var(--shadow-xs);
  z-index: var(--z-sticky);
  flex-shrink: 0;
}

.header-brand {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.brand-mark {
  display: flex;
  align-items: center;
}

.brand-icon {
  width: 32px;
  height: 32px;
}

.brand-name {
  font-size: var(--text-lg);
  font-weight: var(--font-weight-bold);
  color: var(--color-gray-800);
  letter-spacing: var(--tracking-tight);
}

.brand-subtitle {
  font-size: var(--text-xs);
  color: var(--color-gray-400);
  background: var(--color-gray-50);
  padding: 2px 8px;
  border-radius: var(--radius-full);
  font-weight: var(--font-weight-medium);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}

.header-status {
  padding-right: var(--space-4);
  border-right: 1px solid var(--border-light);
}

.btn-logout {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  background: transparent;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-md);
  color: var(--color-gray-600);
  font-size: var(--text-sm);
  font-family: var(--font-sans);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out-quart);
}

.btn-logout:hover {
  background: var(--color-danger-50);
  border-color: var(--color-danger-200);
  color: var(--color-danger-600);
}

.logout-icon {
  width: 16px;
  height: 16px;
}

/* ═══ Body ═══ */
.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* ═══ Sidebar ═══ */
.app-sidebar {
  width: 220px;
  flex-shrink: 0;
  background: var(--surface-primary);
  border-right: 1px solid var(--border-light);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.sidebar-nav {
  padding: var(--space-4) var(--space-3);
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: 10px 12px;
  margin-bottom: 2px;
  border-radius: var(--radius-md);
  color: var(--color-gray-600);
  font-size: var(--text-sm);
  font-weight: var(--font-weight-medium);
  text-decoration: none;
  transition: all var(--duration-fast) var(--ease-out-quart);
  position: relative;
}

.nav-item:hover {
  background: var(--color-brand-50);
  color: var(--color-brand-600);
}

.nav-item.active {
  background: var(--color-brand-50);
  color: var(--color-brand-600);
  font-weight: var(--font-weight-semibold);
}

.nav-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 20px;
  background: var(--color-brand-500);
  border-radius: 0 2px 2px 0;
}

.nav-item--child {
  padding-left: 24px;
}

.nav-step-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  font-size: 11px;
  font-weight: var(--font-weight-semibold);
  color: var(--color-gray-400);
  background: var(--color-gray-100);
  border-radius: var(--radius-full);
  flex-shrink: 0;
  transition: all var(--duration-fast) var(--ease-out-quart);
}

.nav-item.active .nav-step-badge {
  color: #fff;
  background: var(--color-brand-500);
}

.nav-icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  opacity: 0.7;
}

.nav-item.active .nav-icon {
  opacity: 1;
}

.nav-group {
  margin: var(--space-3) 0;
}

.nav-group-label {
  font-size: 11px;
  font-weight: var(--font-weight-semibold);
  color: var(--color-gray-400);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  padding: 4px 12px 8px;
}

.sidebar-footer {
  padding: var(--space-3) var(--space-4);
  border-top: 1px solid var(--border-light);
}

.version-tag {
  font-size: var(--text-xs);
  color: var(--color-gray-400);
  font-weight: var(--font-weight-medium);
}

/* ═══ Main Content ═══ */
.app-main {
  flex: 1;
  padding: var(--space-6);
  overflow-y: auto;
  background: var(--bg-page);
}

/* ═══ Page Transition ═══ */
.page-enter-active {
  animation: page-in 400ms var(--ease-out-expo);
}

.page-leave-active {
  animation: page-out 200ms var(--ease-out-quart);
}

@keyframes page-in {
  from {
    opacity: 0;
    transform: translateY(12px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes page-out {
  from {
    opacity: 1;
    transform: translateY(0);
  }
  to {
    opacity: 0;
    transform: translateY(-6px);
  }
}
</style>
