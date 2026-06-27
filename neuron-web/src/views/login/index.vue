<template>
  <div class="login-page">
    <!-- 背景装饰 -->
    <div class="bg-decoration">
      <div class="bg-grid"></div>
      <div class="bg-glow bg-glow--1"></div>
      <div class="bg-glow bg-glow--2"></div>
    </div>

    <!-- 登录卡片 -->
    <div class="login-panel">
      <div class="panel-header">
        <div class="logo-area">
          <svg viewBox="0 0 48 48" fill="none" class="logo-icon">
            <rect width="48" height="48" rx="12" fill="#534AB7"/>
            <path d="M24 9L39 16.5V31.5L24 39 9 31.5V16.5L24 9z" stroke="#fff" stroke-width="1.5" fill="none" opacity="0.7"/>
            <circle cx="24" cy="24" r="6" fill="#fff" opacity="0.85"/>
            <circle cx="24" cy="24" r="2.5" fill="#534AB7"/>
            <!-- 数据环 -->
            <circle cx="24" cy="24" r="14" stroke="#fff" stroke-width="0.5" fill="none" opacity="0.3" stroke-dasharray="12 8"/>
          </svg>
        </div>
        <h1 class="app-title">AZ-IOT</h1>
        <p class="app-desc">工业园区能源管理平台</p>
      </div>

      <div class="panel-body">
        <el-form
          :model="form"
          :rules="rules"
          ref="formRef"
          @keyup.enter="handleLogin"
        >
          <el-form-item prop="username">
            <el-input
              v-model="form.username"
              placeholder="请输入账号"
              size="large"
              :prefix-icon="UserFilled"
              class="login-input"
            />
          </el-form-item>
          <el-form-item prop="password">
            <el-input
              v-model="form.password"
              type="password"
              placeholder="请输入密码"
              size="large"
              :prefix-icon="Lock"
              show-password
              class="login-input"
            />
          </el-form-item>

          <el-button
            type="primary"
            size="large"
            :loading="loading"
            @click="handleLogin"
            class="login-btn"
          >
            <span v-if="!loading">登 录</span>
          </el-button>
        </el-form>

        <transition name="fade">
          <div v-if="error" class="error-msg">
            <svg viewBox="0 0 16 16" fill="none" class="error-icon">
              <circle cx="8" cy="8" r="7" stroke="#ef4444" stroke-width="1.5"/>
              <path d="M8 5v3.5M8 11.5v.01" stroke="#ef4444" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            {{ error }}
          </div>
        </transition>
      </div>
    </div>

    <p class="copyright">© 2026 AZ-IOT · 工业物联网能源管理</p>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { UserFilled, Lock } from '@element-plus/icons-vue'
import { authApi } from '@/api/auth'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()
const formRef = ref()
const loading = ref(false)
const error = ref('')

const form = reactive({ username: '', password: '' })
const rules = {
  username: [{ required: true, message: '请输入账号', trigger: 'blur' }],
  password: [{ required: true, message: '请输入密码', trigger: 'blur' }]
}

const handleLogin = async () => {
  const valid = await formRef.value.validate().catch(() => false)
  if (!valid) return
  loading.value = true
  error.value = ''
  try {
    const res = await authApi.login(form) as any
    authStore.setToken(res.data.accessToken)
    localStorage.setItem('refresh_token', res.data.refreshToken)
    router.push('/dashboard')
  } catch (e: any) {
    error.value = e?.response?.data?.message || '登录失败，请检查账号密码'
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
/* ═══ Login Page Layout ═══ */
.login-page {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background: var(--color-gray-25);
  overflow: hidden;
  font-family: var(--font-sans);
}

/* ═══ Background Decoration ═══ */
.bg-decoration {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.bg-grid {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(to right, var(--color-gray-200) 1px, transparent 1px),
    linear-gradient(to bottom, var(--color-gray-200) 1px, transparent 1px);
  background-size: 60px 60px;
  opacity: 0.4;
}

.bg-glow {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  opacity: 0.12;
}

.bg-glow--1 {
  width: 500px;
  height: 500px;
  background: var(--color-brand-400);
  top: -150px;
  right: -100px;
  animation: float-glow 8s ease-in-out infinite;
}

.bg-glow--2 {
  width: 400px;
  height: 400px;
  background: var(--color-success-400);
  bottom: -100px;
  left: -80px;
  animation: float-glow 10s ease-in-out infinite reverse;
}

@keyframes float-glow {
  0%, 100% { transform: translate(0, 0) scale(1); }
  33% { transform: translate(30px, -20px) scale(1.05); }
  66% { transform: translate(-20px, 20px) scale(0.95); }
}

/* ═══ Login Panel ═══ */
.login-panel {
  position: relative;
  z-index: 1;
  width: 420px;
  background: var(--surface-primary);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-xl);
  border: 1px solid var(--border-light);
  overflow: hidden;
  animation: panel-in 600ms var(--ease-out-expo);
}

@keyframes panel-in {
  from {
    opacity: 0;
    transform: translateY(20px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.panel-header {
  text-align: center;
  padding: var(--space-10) var(--space-8) var(--space-4);
}

.logo-area {
  margin-bottom: var(--space-4);
}

.logo-icon {
  width: 48px;
  height: 48px;
  animation: logo-pulse 3s var(--ease-in-out) infinite;
}

@keyframes logo-pulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.05); }
}

.app-title {
  font-size: var(--text-2xl);
  font-weight: var(--font-weight-bold);
  color: var(--color-gray-800);
  letter-spacing: 0.04em;
  margin: 0 0 var(--space-1);
}

.app-desc {
  font-size: var(--text-sm);
  color: var(--color-gray-500);
  margin: 0;
}

.panel-body {
  padding: var(--space-2) var(--space-8) var(--space-8);
}

/* ═══ Form Elements ═══ */
.login-input :deep(.el-input__wrapper) {
  border-radius: var(--radius-md);
  padding: 4px 14px;
  box-shadow: 0 0 0 1px var(--border-light) inset;
  transition: box-shadow var(--duration-fast) var(--ease-out-quart);
}

.login-input :deep(.el-input__wrapper:hover) {
  box-shadow: 0 0 0 1px var(--color-gray-300) inset;
}

.login-btn {
  width: 100%;
  height: 46px;
  font-size: var(--text-base);
  font-weight: var(--font-weight-semibold);
  letter-spacing: 0.08em;
  border-radius: var(--radius-md);
  margin-top: var(--space-2);
}

/* ═══ Error Message ═══ */
.error-msg {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  margin-top: var(--space-4);
  padding: var(--space-3);
  background: var(--color-danger-50);
  border-radius: var(--radius-md);
  color: var(--color-danger-600);
  font-size: var(--text-sm);
  font-weight: var(--font-weight-medium);
}

.error-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: all var(--duration-fast) var(--ease-out-quart);
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

/* ═══ Copyright ═══ */
.copyright {
  position: absolute;
  bottom: var(--space-8);
  font-size: var(--text-xs);
  color: var(--color-gray-400);
  z-index: 1;
}
</style>
