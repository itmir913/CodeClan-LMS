import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  api,
  type StudentLessonRow,
  type StudentAssessmentRow,
  type StudentActiveSession,
  type SessionProblemRow,
} from '@/api/client'

export const useStudentStore = defineStore('student', () => {
  const lessons = ref<StudentLessonRow[]>([])
  const assessments = ref<StudentAssessmentRow[]>([])
  const activeSession = ref<StudentActiveSession | null>(null)
  const sessionProblems = ref<SessionProblemRow[]>([])
  const loading = ref(false)
  const examLoading = ref(false)
  const error = ref<string | null>(null)

  async function fetchLessons() {
    try {
      lessons.value = await api.student.lessons()
    } catch (e) {
      error.value = e instanceof Error ? e.message : '차시 목록을 불러오지 못했습니다'
    }
  }

  async function fetchAssessments() {
    try {
      assessments.value = await api.student.assessments()
    } catch (e) {
      error.value = e instanceof Error ? e.message : '수행평가 목록을 불러오지 못했습니다'
    }
  }

  async function fetchActiveSession() {
    try {
      activeSession.value = await api.student.activeSession()
    } catch {
      // 폴링 오류는 조용히 무시
    }
  }

  async function fetchSessionProblems() {
    examLoading.value = true
    try {
      sessionProblems.value = await api.student.sessionProblems()
    } catch (e) {
      error.value = e instanceof Error ? e.message : '문제 목록을 불러오지 못했습니다'
    } finally {
      examLoading.value = false
    }
  }

  async function submitAnswer(data: { problem_id: number; content: string; language?: string }) {
    const result = await api.student.submit(data)
    const idx = sessionProblems.value.findIndex(p => p.problem_id === data.problem_id)
    if (idx !== -1) {
      sessionProblems.value[idx] = {
        ...sessionProblems.value[idx],
        submission_id: result.submission_id,
        submitted_content: data.content,
        submitted_language: data.language ?? null,
        verdict: result.verdict,
        submitted_score: result.score,
      }
    }
    return result
  }

  async function loadAll() {
    loading.value = true
    error.value = null
    try {
      await Promise.all([fetchLessons(), fetchAssessments(), fetchActiveSession()])
    } finally {
      loading.value = false
    }
  }

  return {
    lessons,
    assessments,
    activeSession,
    sessionProblems,
    loading,
    examLoading,
    error,
    fetchLessons,
    fetchAssessments,
    fetchActiveSession,
    fetchSessionProblems,
    submitAnswer,
    loadAll,
  }
})
