<template>
  <el-container>
    <el-header style="display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid #eee;">
      <span style="font-weight:500;font-size:16px;">AZ-IOT</span>
      <el-button type="text" @click="logout">退出</el-button>
    </el-header>
    <el-container>
      <el-aside width="200px" style="border-right:1px solid #eee;min-height:calc(100vh - 60px);">
        <el-menu router default-active="/dashboard">
          <el-menu-item index="/dashboard">工作台</el-menu-item>
          <el-sub-menu index="device-tpl">
            <template #title>设备模板</template>
            <el-menu-item index="/manufacturer">品牌管理</el-menu-item>
            <el-menu-item index="/device-model">型号管理</el-menu-item>
          </el-sub-menu>
          <el-menu-item index="/collector">采集器管理</el-menu-item>
          <el-menu-item index="/device">设备管理</el-menu-item>
        </el-menu>
      </el-aside>
      <el-main><router-view /></el-main>
    </el-container>
  </el-container>
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
