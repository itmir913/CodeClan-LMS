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

// ── Types ─────────────────────────────────────────────────────

export interface SetupStatus {
  needs_setup: boolean
}

export interface SetupRequest {
  school_name: string
  admin_name: string
  admin_username: string
  admin_password: string
}

// ── API object ────────────────────────────────────────────────

export const api = {
  setup: {
    status: () => request<SetupStatus>('GET', '/setup/status'),
    complete: (data: SetupRequest) => request<{ ok: boolean }>('POST', '/setup/complete', data),
  },
}
