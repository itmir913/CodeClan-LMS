const BASE = '/api'

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method,
    credentials: 'include',
    headers: body ? { 'Content-Type': 'application/json' } : undefined,
    body: body ? JSON.stringify(body) : undefined,
  })

  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error(err.error ?? '알 수 없는 오류')
  }

  return res.json()
}

export const api = {
  setup: {
    status: () => request<{ needs_setup: boolean }>('GET', '/setup/status'),
    complete: (data: {
      school_name: string
      admin_name: string
      admin_username: string
      admin_password: string
    }) => request<{ ok: boolean }>('POST', '/setup/complete', data),
  },

  auth: {
    loginTeacher: (username: string, password: string) =>
      request<{ ok: boolean; user: { id: number; username: string; name: string; role: string } }>(
        'POST',
        '/auth/login/teacher',
        { username, password },
      ),
    logout: () => request<{ ok: boolean }>('POST', '/auth/logout'),
    me: () =>
      request<{ id: number; username: string; name: string; role: string }>('GET', '/auth/me'),
    schoolName: () => request<{ school_name: string }>('GET', '/auth/school-name'),
  },
}
