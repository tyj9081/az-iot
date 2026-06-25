import request from '@/utils/request'

export const protocolApi = {
  listAll: () => request.get('/protocols')
}
