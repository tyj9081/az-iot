import request from '@/utils/request'

export const readingApi = {
  latest: (deviceId: number) => request.get(`/devices/${deviceId}/readings/latest`),
  history: (deviceId: number, params: any) => request.get(`/devices/${deviceId}/readings/history`, { params })
}
