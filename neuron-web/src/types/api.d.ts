export interface ApiResponse<T = any> {
  code: number
  message: string
  data: T
  timestamp: number
}

export interface PageResult<T> {
  records: T[]
  total: number
  page: number
  pageSize: number
}

export interface LoginRequest {
  username: string
  password: string
}

export interface LoginResponse {
  accessToken: string
  refreshToken: string
}

export interface UserInfo {
  id: number
  username: string
  nickname: string
  roles: string[]
  permissions: string[]
}
