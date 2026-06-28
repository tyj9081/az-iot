import { defineStore } from 'pinia'
import { ref } from 'vue'
import { authApi } from '@/api/auth'

export const useAuthStore = defineStore('auth', () => {
  const token = ref(localStorage.getItem('access_token') || '')
  const refreshToken = ref(localStorage.getItem('refresh_token') || '')
  const userInfo = ref<any>(null)

  const setToken = (access: string, refresh?: string) => {
    token.value = access
    localStorage.setItem('access_token', access)
    if (refresh) {
      refreshToken.value = refresh
      localStorage.setItem('refresh_token', refresh)
    }
  }

  const clearAuth = () => {
    token.value = ''
    refreshToken.value = ''
    userInfo.value = null
    localStorage.removeItem('access_token')
    localStorage.removeItem('refresh_token')
  }

  const isLoggedIn = () => !!token.value

  async function refreshAccessToken(): Promise<string | null> {
    if (!refreshToken.value) return null
    try {
      const res: any = await authApi.refresh({ refreshToken: refreshToken.value })
      const newAccess = res?.data?.accessToken
      const newRefresh = res?.data?.refreshToken
      if (newAccess) {
        setToken(newAccess, newRefresh || undefined)
        return newAccess
      }
      return null
    } catch {
      clearAuth()
      return null
    }
  }

  return { token, refreshToken, userInfo, setToken, clearAuth, isLoggedIn, refreshAccessToken }
})
