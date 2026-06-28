import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/login/index.vue'),
    meta: { title: '登录' }
  },
  {
    path: '/',
    component: () => import('@/views/layout/index.vue'),
    redirect: '/dashboard',
    children: [
      {
        path: 'dashboard',
        name: 'Dashboard',
        component: () => import('@/views/dashboard/index.vue'),
        meta: { title: '工作台' }
      },
      {
        path: 'manufacturer',
        name: 'Manufacturer',
        component: () => import('@/views/manufacturer/index.vue'),
        meta: { title: '品牌管理' }
      },
      {
        path: 'device-model',
        name: 'DeviceModel',
        component: () => import('@/views/device-model/index.vue'),
        meta: { title: '型号管理' }
      },
      {
        path: 'device-model/:modelId/registers',
        name: 'DeviceModelRegisters',
        component: () => import('@/views/device-model/registers.vue'),
        meta: { title: '点表管理' }
      },
      {
        path: 'collector',
        name: 'Collector',
        component: () => import('@/views/collector/list.vue'),
        meta: { title: '采集器' }
      },
      {
        path: 'device',
        name: 'Device',
        component: () => import('@/views/device/index.vue'),
        meta: { title: '设备管理' }
      },
      {
        path: 'device/:id',
        name: 'DeviceDetail',
        component: () => import('@/views/device/detail.vue'),
        meta: { title: '设备详情' }
      }
    ]
  }
]

const router = createRouter({ history: createWebHistory(), routes })

router.beforeEach((to, from, next) => {
  const token = localStorage.getItem('access_token')
  if (to.path !== '/login' && !token) {
    next('/login')
  } else if (to.path === '/login' && token) {
    next('/dashboard')
  } else {
    next()
  }
})

export default router
