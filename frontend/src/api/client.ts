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
    const err = await res.json().catch(() => ({ error: 'ERR_UNKNOWN' }))
    throw new Error(err.error ?? 'ERR_UNKNOWN')
  }

  return res.json()
}

// ── Types ─────────────────────────────────────────────────────

export interface SetupStatus {
  needs_setup: boolean
  locale: string | null
}

export interface SetupRequest {
  school_name: string
  admin_name: string
  admin_username: string
  admin_password: string
  locale: string
}

export interface TeacherUser {
  id: number
  username: string
  name: string
  role: string
}

export interface StudentUser {
  id: number
  username: string
  name: string
  grade: number
  class_no: number
  number: number
  password_reset_required: boolean
}

export interface TeacherLoginResponse {
  user: TeacherUser
  locale: string
}

export interface StudentLoginResponse {
  user: StudentUser
  locale: string
}

export interface SchoolNameResponse {
  school_name: string
}

// ── API object ────────────────────────────────────────────────

export const api = {
  setup: {
    status: () => request<SetupStatus>('GET', '/setup/status'),
    complete: (data: SetupRequest) => request<{ ok: boolean }>('POST', '/setup/complete', data),
  },
  auth: {
    loginTeacher: (username: string, password: string) =>
      request<TeacherLoginResponse>('POST', '/auth/login/teacher', { username, password }),
    logoutTeacher: () => request<{ ok: boolean }>('POST', '/auth/logout'),
    meTeacher: () => request<TeacherUser>('GET', '/auth/me'),
    schoolName: () => request<SchoolNameResponse>('GET', '/auth/school-name'),
    loginStudent: (username: string, password: string) =>
      request<StudentLoginResponse>('POST', '/auth/login/student', { username, password }),
    logoutStudent: () => request<{ ok: boolean }>('POST', '/auth/logout/student'),
    meStudent: () => request<StudentUser>('GET', '/auth/student/me'),
    changePasswordStudent: (currentPassword: string | null, newPassword: string) =>
      request<{ ok: boolean }>('POST', '/auth/student/change-password', {
        current_password: currentPassword,
        new_password: newPassword,
      }),
  },
}
