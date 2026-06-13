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

// ─── 인증 ─────────────────────────────────────────────────────

export interface TeacherUser {
  id: number
  username: string
  name: string
  role: 'admin' | 'teacher'
}

export interface StudentUser {
  id: number
  student_number: string
  name: string
  division_id: number
  division_name: string
  password_reset_required: boolean
}

// ─── 대시보드 ──────────────────────────────────────────────────

export interface DashboardResponse {
  teacher_name: string
  teacher_role: string
  division_count: number
  divisions: Array<{
    id: number
    name: string
    student_count: number
    active_session: {
      id: number
      assessment_title: string
      status: string
      submission_count: number
      student_count: number
      time_limit_min: number | null
      start_at: string | null
    } | null
  }>
  stats: { problem_count: number; lesson_count: number; assessment_count: number }
  recent_logs: Array<{
    id: number
    action_type: string
    detail: string | null
    created_at: string
  }>
}

// ─── 분반/학생/교사 관리 ──────────────────────────────────

export interface DivisionRow {
  id: number
  name: string
  student_count: number
  teacher_count: number
  created_at: string
}

export interface TeacherBrief {
  id: number
  name: string
  username: string
  role: string
}

export interface StudentRow {
  id: number
  student_number: string
  name: string
  division_id: number
  password_reset_required: boolean
  created_at: string
}

export interface TeacherRow {
  id: number
  username: string
  name: string
  role: string
  division_count: number
  created_at: string
}

export interface BulkStudentItem {
  student_number: string
  name: string
  password: string
}

export interface BulkImportResult {
  inserted: number
  skipped: number
  errors: string[]
}

// ─── 수행평가 관리 ──────────────────────────────────────────

export interface AssessmentRow {
  id: number
  title: string
  description: string
  problem_count: number
  division_count: number
  is_locked: boolean
  created_at: string
}

export interface AssessmentProblemRow {
  id: number
  problem_id: number
  problem_type: number
  problem_title: string
  order_no: number
  score: number
}

export interface AssessmentDivisionRow {
  division_id: number
  division_name: string
  has_running_session: boolean
}

export interface AssessmentDetail extends AssessmentRow {
  problems: AssessmentProblemRow[]
  divisions: AssessmentDivisionRow[]
}

// ─── 차시 관리 ───────────────────────────────────────────────

export interface LessonRow {
  id: number
  title: string
  description: string
  order_no: number
  problem_count: number
  created_at: string
}

export interface LessonProblemRow {
  id: number
  problem_id: number
  problem_type: number
  problem_title: string
  order_no: number
}

export interface LessonRelease {
  division_id: number
  division_name: string
  is_released: boolean
  released_at: string | null
}

export interface LessonDetail extends LessonRow {
  problems: LessonProblemRow[]
  releases: LessonRelease[]
}

// ─── 문제 은행 ───────────────────────────────────────────────

export const PROBLEM_TYPE_LABELS = {
  1: '①실행결과맞히기',
  2: '②코드작성형',
  3: '③과제/보고서형',
  4: '④빈칸채우기',
} as const

export type ProblemType = 1 | 2 | 3 | 4

export interface ProblemListItem {
  id: number
  problem_type: number
  title: string
  is_structure_check: boolean
  created_at: string
}

export interface ProblemRow {
  id: number
  problem_type: number
  title: string
  description: string
  type_config: string
  is_structure_check: boolean
  created_at: string
}

export interface CreateProblemInput {
  problem_type: number
  title: string
  description?: string
  type_config?: string
  is_structure_check?: boolean
}

export interface UpdateProblemInput {
  title?: string
  description?: string
  type_config?: string
  is_structure_check?: boolean
}

// ─── 세션 관리 ───────────────────────────────────────────────

export interface SessionRow {
  id: number
  assessment_id: number
  assessment_title: string
  division_id: number
  division_name: string
  status: 'CREATED' | 'LOBBY' | 'RUNNING' | 'CLOSED'
  target_type: 'ALL' | 'INDIVIDUAL'
  time_limit_min: number | null
  start_at: string | null
  end_at: string | null
  is_paused: boolean
  is_result_released: boolean
  submission_count: number
  student_count: number
  created_at: string
}

export interface CreateSessionInput {
  assessment_id: number
  division_id: number
  target_type?: 'ALL' | 'INDIVIDUAL'
  time_limit_min?: number
  student_ids?: number[]
}

// ─── 학생 전용 ───────────────────────────────────────────────

export interface StudentLessonRow {
  id: number
  title: string
  description: string
  order_no: number
  problem_count: number
  released_at: string | null
}

export interface StudentAssessmentRow {
  id: number
  title: string
  description: string
  problem_count: number
  session_id: number | null
  session_status: 'CREATED' | 'LOBBY' | 'RUNNING' | 'CLOSED' | null
  is_result_released: boolean
}

export interface StudentActiveSession {
  id: number
  assessment_id: number
  assessment_title: string
  status: 'LOBBY' | 'RUNNING'
  time_limit_min: number | null
  start_at: string | null
  is_paused: boolean
  is_result_released: boolean
}

// ─── 제출 / 채점 ─────────────────────────────────────────────

export interface SessionProblemRow {
  ap_id: number
  order_no: number
  max_score: number
  problem_id: number
  problem_type: number
  title: string
  description: string
  type_config: string
  is_structure_check: boolean
  submission_id: number | null
  submitted_content: string | null
  submitted_language: string | null
  verdict: string | null
  submitted_score: number | null
}

export interface SubmissionResult {
  submission_id: number
  verdict: string | null
  score: number | null
}

export interface SubmissionRow {
  id: number
  student_id: number
  student_name: string
  student_number: string
  problem_id: number
  problem_type: number
  problem_title: string
  problem_order: number
  max_score: number
  content: string
  language: string | null
  verdict: string | null
  score: number | null
  submission_no: number
  created_at: string
}

// ─── API 객체 ─────────────────────────────────────────────────

export const api = {
  dashboard: {
    get: () => request<DashboardResponse>('GET', '/dashboard'),
  },

  divisions: {
    list: () => request<DivisionRow[]>('GET', '/divisions'),
    create: (name: string) => request<DivisionRow>('POST', '/divisions', { name }),
    update: (id: number, name: string) =>
      request<{ ok: boolean }>('PUT', `/divisions/${id}`, { name }),
    delete: (id: number) => request<{ ok: boolean }>('DELETE', `/divisions/${id}`),
    getTeachers: (id: number) => request<TeacherBrief[]>('GET', `/divisions/${id}/teachers`),
    setTeachers: (id: number, teacher_ids: number[]) =>
      request<{ ok: boolean }>('PUT', `/divisions/${id}/teachers`, { teacher_ids }),
    getStudents: (id: number) => request<StudentRow[]>('GET', `/divisions/${id}/students`),
    addStudent: (id: number, data: BulkStudentItem) =>
      request<StudentRow>('POST', `/divisions/${id}/students`, data),
    bulkImport: (id: number, items: BulkStudentItem[]) =>
      request<BulkImportResult>('POST', `/divisions/${id}/students/bulk`, items),
  },

  students: {
    delete: (id: number) => request<{ ok: boolean }>('DELETE', `/students/${id}`),
    resetPassword: (id: number, new_password: string) =>
      request<{ ok: boolean }>('POST', `/students/${id}/reset-password`, { new_password }),
  },

  teachers: {
    list: () => request<TeacherRow[]>('GET', '/teachers'),
    create: (data: { username: string; name: string; password: string; role?: string }) =>
      request<TeacherRow>('POST', '/teachers', data),
    update: (id: number, data: { name?: string; role?: string; password?: string }) =>
      request<{ ok: boolean }>('PUT', `/teachers/${id}`, data),
    delete: (id: number) => request<{ ok: boolean }>('DELETE', `/teachers/${id}`),
  },

  assessments: {
    list: () => request<AssessmentRow[]>('GET', '/assessments'),
    get: (id: number) => request<AssessmentDetail>('GET', `/assessments/${id}`),
    create: (data: { title: string; description?: string }) =>
      request<AssessmentRow>('POST', '/assessments', data),
    update: (id: number, data: { title?: string; description?: string }) =>
      request<{ ok: boolean }>('PUT', `/assessments/${id}`, data),
    delete: (id: number) => request<{ ok: boolean }>('DELETE', `/assessments/${id}`),
    setProblems: (id: number, items: Array<{ problem_id: number; score: number }>) =>
      request<{ ok: boolean }>('PUT', `/assessments/${id}/problems`, items),
    linkDivision: (id: number, division_id: number) =>
      request<{ ok: boolean }>('POST', `/assessments/${id}/divisions`, { division_id }),
    unlinkDivision: (id: number, division_id: number) =>
      request<{ ok: boolean }>('DELETE', `/assessments/${id}/divisions/${division_id}`),
  },

  lessons: {
    list: () => request<LessonRow[]>('GET', '/lessons'),
    get: (id: number) => request<LessonDetail>('GET', `/lessons/${id}`),
    create: (data: { title: string; description?: string; order_no?: number }) =>
      request<LessonRow>('POST', '/lessons', data),
    update: (id: number, data: { title?: string; description?: string; order_no?: number }) =>
      request<{ ok: boolean }>('PUT', `/lessons/${id}`, data),
    delete: (id: number) => request<{ ok: boolean }>('DELETE', `/lessons/${id}`),
    setProblems: (id: number, problem_ids: number[]) =>
      request<{ ok: boolean }>('PUT', `/lessons/${id}/problems`, { problem_ids }),
    toggleRelease: (id: number, division_id: number, is_released: boolean) =>
      request<{ ok: boolean; released_at: string | null }>(
        'PUT', `/lessons/${id}/release`, { division_id, is_released }
      ),
  },

  problems: {
    list: (params?: { problem_type?: number; q?: string }) => {
      const qs = new URLSearchParams()
      if (params?.problem_type) qs.set('problem_type', String(params.problem_type))
      if (params?.q) qs.set('q', params.q)
      const query = qs.toString()
      return request<ProblemListItem[]>('GET', `/problems${query ? `?${query}` : ''}`)
    },
    get: (id: number) => request<ProblemRow>('GET', `/problems/${id}`),
    create: (data: CreateProblemInput) => request<ProblemRow>('POST', '/problems', data),
    update: (id: number, data: UpdateProblemInput) =>
      request<{ ok: boolean }>('PUT', `/problems/${id}`, data),
    delete: (id: number) => request<{ ok: boolean }>('DELETE', `/problems/${id}`),
  },

  sessions: {
    list: (params?: { division_id?: number; assessment_id?: number; status?: string }) => {
      const qs = new URLSearchParams()
      if (params?.division_id) qs.set('division_id', String(params.division_id))
      if (params?.assessment_id) qs.set('assessment_id', String(params.assessment_id))
      if (params?.status) qs.set('status', params.status)
      const query = qs.toString()
      return request<SessionRow[]>('GET', `/sessions${query ? `?${query}` : ''}`)
    },
    create: (data: CreateSessionInput) => request<SessionRow>('POST', '/sessions', data),
    transition: (id: number, action: string) =>
      request<{ ok: boolean; status: string }>('POST', `/sessions/${id}/transition`, { action }),
    pause: (id: number) =>
      request<{ ok: boolean; is_paused: boolean }>('POST', `/sessions/${id}/pause`),
    toggleResultRelease: (id: number) =>
      request<{ ok: boolean; is_result_released: boolean }>(
        'POST', `/sessions/${id}/result-release`
      ),
  },

  student: {
    lessons: () => request<StudentLessonRow[]>('GET', '/student/lessons'),
    assessments: () => request<StudentAssessmentRow[]>('GET', '/student/assessments'),
    activeSession: () => request<StudentActiveSession | null>('GET', '/student/active-session'),
    sessionProblems: () => request<SessionProblemRow[]>('GET', '/student/session-problems'),
    submit: (data: { problem_id: number; content: string; language?: string }) =>
      request<SubmissionResult>('POST', '/student/submit', data),
  },

  submissions: {
    forSession: (session_id: number) =>
      request<SubmissionRow[]>('GET', `/sessions/${session_id}/submissions`),
    grade: (id: number, score: number) =>
      request<{ ok: boolean }>('POST', `/submissions/${id}/grade`, { score }),
  },

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
      request<{ ok: boolean; user: TeacherUser }>(
        'POST', '/auth/login/teacher', { username, password },
      ),
    logout: () => request<{ ok: boolean }>('POST', '/auth/logout'),
    me: () => request<TeacherUser>('GET', '/auth/me'),
    schoolName: () => request<{ school_name: string }>('GET', '/auth/school-name'),

    loginStudent: (student_number: string, password: string) =>
      request<{ ok: boolean; user: StudentUser }>(
        'POST', '/auth/login/student', { student_number, password },
      ),
    logoutStudent: () => request<{ ok: boolean }>('POST', '/auth/logout/student'),
    studentMe: () => request<StudentUser>('GET', '/auth/student/me'),
    studentChangePassword: (current_password: string, new_password: string) =>
      request<{ ok: boolean }>(
        'POST', '/auth/student/change-password', { current_password, new_password },
      ),
  },
}
