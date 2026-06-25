import request from '@/utils/request'
import type { ApiResponse, PageResult } from '@/types/api'

export const manufacturerApi = {
  list: (params: { page: number; pageSize: number; keyword?: string }) =>
    request.get<any, ApiResponse<PageResult<any>>>('/manufacturers', { params }),
  getById: (id: number) => request.get('/manufacturers/' + id),
  create: (data: any) => request.post('/manufacturers', data),
  update: (id: number, data: any) => request.put('/manufacturers/' + id, data),
  remove: (id: number) => request.delete('/manufacturers/' + id)
}
