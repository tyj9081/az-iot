import axios from 'axios'
import type { AxiosInstance, InternalAxiosRequestConfig } from 'axios'

const request: AxiosInstance = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api/v1',
  timeout: 15000,
  headers: { 'Content-Type': 'application/json' }
})

// 防止多个并发请求同时触发刷新
let isRefreshing = false

interface PendingRequest {
  resolve: (token: string) => void
  reject: (error: Error) => void
}
let pendingRequests: PendingRequest[] = []

request.interceptors.request.use(config => {
  const token = localStorage.getItem('access_token')
  if (token) config.headers.Authorization = `Bearer ${token}`
  return config
})

request.interceptors.response.use(
  response => response.data,
  async error => {
    const originalRequest = error.config as InternalAxiosRequestConfig & { _retry?: boolean }

    if (error.response?.status === 401 && !originalRequest._retry) {
      // 排除 refresh 接口自身，避免死循环
      const isRefreshRequest = originalRequest.url?.includes('/auth/refresh')

      if (!isRefreshRequest) {
        if (!isRefreshing) {
          isRefreshing = true
          originalRequest._retry = true

          try {
            // 动态导入避免循环依赖
            const { useAuthStore } = await import('@/stores/auth')
            const authStore = useAuthStore()
            const newToken = await authStore.refreshAccessToken()

            if (newToken) {
              isRefreshing = false
              // Replay all queued requests with new token
              pendingRequests.forEach(({ resolve }) => {
                resolve(newToken)
              })
              pendingRequests = []
              // Retry the original request
              originalRequest.headers!.Authorization = `Bearer ${newToken}`
              return request(originalRequest)
            }

            // 刷新失败，跳转登录
            isRefreshing = false
            // Reject all queued requests before clearing
            pendingRequests.forEach(({ reject }) => {
              reject(new Error('Token refresh failed, please login again'))
            })
            pendingRequests = []
            window.location.href = '/login'
            return Promise.reject(error)
          } catch {
            isRefreshing = false
            // Reject all queued requests before clearing
            pendingRequests.forEach(({ reject }) => {
              reject(new Error('Token refresh failed, please login again'))
            })
            pendingRequests = []
            window.location.href = '/login'
            return Promise.reject(error)
          }
        } else {
          // 已有刷新在进行中，排队等待
          originalRequest._retry = true
          return new Promise((resolve, reject) => {
            pendingRequests.push({
              resolve: (newToken: string) => {
                originalRequest.headers!.Authorization = `Bearer ${newToken}`
                resolve(request(originalRequest))
              },
              reject
            })
          })
        }
      }
    }

    return Promise.reject(error)
  }
)

export default request
