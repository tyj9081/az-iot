import request from '@/utils/request'
import type { ApiResponse, PageResult } from '@/types/api'

export const deviceModelApi = {
  list: (params: any) => request.get<any, ApiResponse<PageResult<any>>>('/device-models', { params }),
  getById: (id: number) => request.get('/device-models/' + id),
  create: (data: any) => request.post('/device-models', data),
  update: (id: number, data: any) => request.put('/device-models/' + id, data),
  remove: (id: number) => request.delete('/device-models/' + id)
}
