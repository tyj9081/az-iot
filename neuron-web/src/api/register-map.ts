import request from '@/utils/request'

export const registerMapApi = {
  listByModelId: (modelId: number) => request.get(`/device-models/${modelId}/registers`),
  create: (modelId: number, data: any) => request.post(`/device-models/${modelId}/registers`, data),
  batchCreate: (modelId: number, data: any[]) => request.post(`/device-models/${modelId}/registers/batch`, data),
  update: (modelId: number, id: number, data: any) => request.put(`/device-models/${modelId}/registers/${id}`, data),
  remove: (modelId: number, id: number) => request.delete(`/device-models/${modelId}/registers/${id}`)
}
