import request from '@/utils/request'
import type { ApiResponse } from '@/types/api'

export const instructionApi = {
  list: (deviceId: number) =>
    request.get<any, ApiResponse<any[]>>(`/devices/${deviceId}/instructions`),

  getById: (deviceId: number, id: number) =>
    request.get(`/devices/${deviceId}/instructions/${id}`),

  create: (deviceId: number, data: any) =>
    request.post(`/devices/${deviceId}/instructions`, data),

  update: (deviceId: number, id: number, data: any) =>
    request.put(`/devices/${deviceId}/instructions/${id}`, data),

  remove: (deviceId: number, id: number) =>
    request.delete(`/devices/${deviceId}/instructions/${id}`)
}
