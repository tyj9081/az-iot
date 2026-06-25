import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAuthStore = defineStore('auth', () => {
  const token = ref(localStorage.getItem('access_token') || '')
  const userInfo = ref<any>(null)

  const setToken = (t: string) => { token.value = t; localStorage.setItem('access_token', t) }
  const clearAuth = () => { token.value = ''; userInfo.value = null; localStorage.removeItem('access_token') }
  const isLoggedIn = () => !!token.value

  return { token, userInfo, setToken, clearAuth, isLoggedIn }
})
