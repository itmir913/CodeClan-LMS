import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api, type StudentLessonRow, type StudentAssessmentRow, type StudentActiveSession } from '@/api/client'

export const useStudentStore = defineStore('student', () => {
  const lessons = ref<StudentLessonRow[]>([])
  const assessments = ref<StudentAssessmentRow[]>([])
  const activeSession = ref<StudentActiveSession | null>(null)
  const loading = ref(false)
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
    loading,
    error,
    fetchLessons,
    fetchAssessments,
    fetchActiveSession,
    loadAll,
  }
})
