import request from '@/utils/request'

export const deviceApi = {
  list: (params: any) => request.get('/devices', { params }),
  getById: (id: number) => request.get('/devices/' + id),
  create: (data: any) => request.post('/devices', data),
  update: (id: number, data: any) => request.put('/devices/' + id, data),
  remove: (id: number) => request.delete('/devices/' + id),
  updateStatus: (id: number, status: string) => request.put('/devices/' + id + '/status', { status })
}
