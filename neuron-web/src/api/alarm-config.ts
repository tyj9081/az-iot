import request from '@/utils/request'

export const alarmConfigApi = {
  list: (deviceId: number) => request.get(`/devices/${deviceId}/alarm-config`),
  save: (deviceId: number, sensorCode: string, data: any) =>
    request.put(`/devices/${deviceId}/alarm-config/${sensorCode}`, data),
  remove: (deviceId: number, sensorCode: string) =>
    request.delete(`/devices/${deviceId}/alarm-config/${sensorCode}`)
}
