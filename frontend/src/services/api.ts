const BASE_URL = '/api'

interface RequestOptions {
  method?: string
  body?: unknown
  headers?: Record<string, string>
  token?: string
}

async function request<T>(
  endpoint: string,
  options: RequestOptions = {},
): Promise<T> {
  const { method = 'GET', body, headers = {}, token } = options

  const fetchHeaders: Record<string, string> = {
    'Content-Type': 'application/json',
    ...headers,
  }

  if (token) {
    fetchHeaders['Authorization'] = `Bearer ${token}`
  }

  const response = await fetch(`${BASE_URL}${endpoint}`, {
    method,
    headers: fetchHeaders,
    body: body ? JSON.stringify(body) : undefined,
  })

  if (!response.ok) {
    const error = await response.text()
    throw new Error(`API error ${response.status}: ${error}`)
  }

  return response.json() as Promise<T>
}

// Mission API functions
export const missionApi = {
  fetchMissions: (status?: string) => {
    const query = status ? `?status=${status}` : ''
    return api.get<unknown[]>(`/missions${query}`)
  },
  createMission: (req: { start_node_id: string; end_node_id: string; priority?: number }) =>
    api.post<unknown>('/missions', req),
  assignMission: (id: string, robotId: string) =>
    api.post<unknown>(`/missions/${id}/assign`, { robot_id: robotId }),
  cancelMission: (id: string) =>
    api.post<unknown>(`/missions/${id}/cancel`, {}),
}

export const api = {
  get: <T>(endpoint: string, token?: string) =>
    request<T>(endpoint, { token }),

  post: <T>(endpoint: string, body: unknown, token?: string) =>
    request<T>(endpoint, { method: 'POST', body, token }),

  put: <T>(endpoint: string, body: unknown, token?: string) =>
    request<T>(endpoint, { method: 'PUT', body, token }),

  delete: <T>(endpoint: string, token?: string) =>
    request<T>(endpoint, { method: 'DELETE', token }),
}
