import request from '@/utils/request'

export const authApi = {
  login: (data: { username: string; password: string }) => request.post('/auth/login', data),
  refresh: (data: { refreshToken: string }) => request.post('/auth/refresh', data),
  logout: () => request.post('/auth/logout'),
  me: () => request.get('/auth/me')
}
