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

export interface Subject {
  id: number
  name: string
}

export interface ClassItem {
  id: number
  name: string
  subject_id: number
  subject_name: string
  teacher_id: number
  teacher_name: string
  student_count: number
  has_active_session: boolean
  created_at: string
}

export interface AdminTeacher {
  id: number
  username: string
  name: string
  role: string
  created_at: string
}

export interface ClassDetail {
  id: number
  name: string
  subject_id: number
  subject_name: string
  teacher_id: number
  teacher_name: string
  student_count: number
  created_at: string
}

export interface StudentItem {
  id: number
  username: string
  name: string
  grade: number
  class_no: number
  number: number
  password_reset_required: boolean
}

export interface AddStudentBody {
  name: string
  grade: number
  class_no: number
  number: number
}

export interface BulkResult {
  inserted: number
  skipped: number
}

export interface ImportResult {
  imported: number
}

export interface ImportTeacherRow {
  username: string
  name: string
  password: string
  role?: string
}

export interface ImportSubjectRow {
  name: string
}

export interface AppSettings {
  school_name: string
  locale: string
}

export interface ImportStudentRow {
  grade: number
  class_no: number
  number: number
  name: string
  username?: string
}

export interface ProblemListItem {
  id: number
  type: string
  title: string
  subject_id: number | null
  subject_name: string | null
  is_draft: boolean
  created_at: string
}

export interface ProblemChoice {
  id: number
  order_no: number
  content: string
  is_correct: boolean
}

export interface ProblemTestCase {
  id: number
  number: number
  input: string
  expected_output: string
  is_sample: boolean
  explanation: string
}

export interface ProblemDetail {
  id: number
  type: string
  title: string
  description: string
  comment: string
  subject_id: number | null
  subject_name: string | null
  is_draft: boolean
  created_at: string
  // short_answer
  answer?: string
  case_sensitive?: boolean
  // multiple_choice
  allow_multiple?: boolean
  choices?: ProblemChoice[]
  // code_submit
  input_format?: string
  output_format?: string
  constraints?: string
  time_limit_ms?: number
  memory_limit_mb?: number
  show_io_on_fail?: boolean
  test_cases?: ProblemTestCase[]
}

export interface ChoiceInput {
  content: string
  is_correct: boolean
}

export interface TestCaseInput {
  input: string
  expected_output: string
  is_sample: boolean
  explanation: string
}

export interface CreateProblemBody {
  type: string
  title: string
  description: string
  comment: string
  is_draft: boolean
  subject_id: number | null
  answer?: string
  case_sensitive?: boolean
  allow_multiple?: boolean
  choices?: ChoiceInput[]
  input_format?: string
  output_format?: string
  constraints?: string
  time_limit_ms?: number
  memory_limit_mb?: number
  show_io_on_fail?: boolean
  test_cases?: TestCaseInput[]
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
    meTeacher: () => request<TeacherLoginResponse>('GET', '/auth/me'),
    schoolName: () => request<SchoolNameResponse>('GET', '/auth/school-name'),
    loginStudent: (username: string, password: string) =>
      request<StudentLoginResponse>('POST', '/auth/login/student', { username, password }),
    logoutStudent: () => request<{ ok: boolean }>('POST', '/auth/logout/student'),
    meStudent: () => request<StudentLoginResponse>('GET', '/auth/student/me'),
    updateTeacherName: (name: string) =>
      request<{ ok: boolean }>('PUT', '/auth/me', { name }),
    changePasswordTeacher: (currentPassword: string, newPassword: string) =>
      request<{ ok: boolean }>('PUT', '/auth/me/password', {
        current_password: currentPassword,
        new_password: newPassword,
      }),
    changePasswordStudent: (currentPassword: string | null, newPassword: string) =>
      request<{ ok: boolean }>('POST', '/auth/student/change-password', {
        current_password: currentPassword,
        new_password: newPassword,
      }),
  },
  subjects: {
    list: () => request<Subject[]>('GET', '/subjects'),
  },
  classes: {
    list: () => request<ClassItem[]>('GET', '/classes'),
    get: (id: number) => request<ClassDetail>('GET', `/classes/${id}`),
    create: (name: string, subject_id: number) =>
      request<{ id: number }>('POST', '/classes', { name, subject_id }),
    update: (id: number, name: string, subject_id: number) =>
      request<{ ok: boolean }>('PUT', `/classes/${id}`, { name, subject_id }),
    delete: (id: number) => request<{ ok: boolean }>('DELETE', `/classes/${id}`),
  },
  students: {
    list: (classId: number) => request<StudentItem[]>('GET', `/classes/${classId}/students`),
    add: (classId: number, data: AddStudentBody) =>
      request<{ id: number }>('POST', `/classes/${classId}/students`, data),
    bulkAdd: (classId: number, data: AddStudentBody[]) =>
      request<BulkResult>('POST', `/classes/${classId}/students/bulk`, data),
    importStudents: (classId: number, data: ImportStudentRow[]) =>
      request<ImportResult>('POST', `/classes/${classId}/students/import`, data),
    delete: (id: number) => request<{ ok: boolean }>('DELETE', `/students/${id}`),
    resetPassword: (id: number) =>
      request<{ ok: boolean }>('POST', `/students/${id}/reset-password`),
  },
  admin: {
    listTeachers: () => request<AdminTeacher[]>('GET', '/admin/teachers'),
    createTeacher: (data: { username: string; name: string; password: string; role?: string }) =>
      request<{ id: number }>('POST', '/admin/teachers', data),
    updateTeacher: (id: number, data: { name?: string; role?: string; password?: string }) =>
      request<{ ok: boolean }>('PUT', `/admin/teachers/${id}`, data),
    deleteTeacher: (id: number) => request<{ ok: boolean }>('DELETE', `/admin/teachers/${id}`),
    createSubject: (name: string) =>
      request<{ id: number }>('POST', '/admin/subjects', { name }),
    deleteSubject: (id: number) => request<{ ok: boolean }>('DELETE', `/admin/subjects/${id}`),
    importTeachers: (data: ImportTeacherRow[]) =>
      request<ImportResult>('POST', '/admin/teachers/import', data),
    importSubjects: (data: ImportSubjectRow[]) =>
      request<ImportResult>('POST', '/admin/subjects/import', data),
    getAppSettings: () => request<AppSettings>('GET', '/admin/app-settings'),
    updateAppSettings: (data: AppSettings) =>
      request<{ ok: boolean }>('PUT', '/admin/app-settings', data),
  },
  settings: {
    setLocale: (locale: string) =>
      request<{ ok: boolean }>('PUT', '/settings/locale', { locale }),
  },
  problems: {
    list: (type?: string) =>
      request<ProblemListItem[]>('GET', `/problems${type ? `?type=${type}` : ''}`),
    get: (id: number) => request<ProblemDetail>('GET', `/problems/${id}`),
    create: (data: CreateProblemBody) => request<{ id: number }>('POST', '/problems', data),
    update: (id: number, data: CreateProblemBody) =>
      request<{ ok: boolean }>('PUT', `/problems/${id}`, data),
    delete: (id: number) => request<{ ok: boolean }>('DELETE', `/problems/${id}`),
  },
}
