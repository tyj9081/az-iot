import request from '@/utils/request'
import type { ApiResponse, PageResult } from '@/types/api'

export const collectorApi = {
  list: (params: any) => request.get<any, ApiResponse<PageResult<any>>>('/collectors', { params }),
  getById: (id: number) => request.get('/collectors/' + id),
  create: (data: any) => request.post('/collectors', data),
  update: (id: number, data: any) => request.put('/collectors/' + id, data),
  remove: (id: number) => request.delete('/collectors/' + id),
  getSerialPorts: (id: number) => request.get('/collectors/' + id + '/serial-ports'),
  updateSerialPort: (cid: number, pid: number, data: any) => request.put('/collectors/' + cid + '/serial-ports/' + pid, data),
  pushConfig: (id: number) => request.post('/collectors/' + id + '/push-config')
}
