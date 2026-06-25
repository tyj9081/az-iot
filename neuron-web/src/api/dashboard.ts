import request from '@/utils/request'

export const dashboardApi = {
  overview: () => request.get('/dashboard/overview')
}
