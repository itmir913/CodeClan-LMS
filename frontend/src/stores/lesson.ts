import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api, type LessonRow, type LessonDetail } from '@/api/client'

export const useLessonStore = defineStore('lesson', () => {
  const lessons = ref<LessonRow[]>([])
  const currentLesson = ref<LessonDetail | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchLessons() {
    loading.value = true
    error.value = null
    try {
      lessons.value = await api.lessons.list()
    } catch (e) {
      error.value = e instanceof Error ? e.message : '차시 목록을 불러오지 못했습니다'
    } finally {
      loading.value = false
    }
  }

  async function fetchLesson(id: number) {
    loading.value = true
    error.value = null
    try {
      currentLesson.value = await api.lessons.get(id)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '차시를 불러오지 못했습니다'
      currentLesson.value = null
    } finally {
      loading.value = false
    }
  }

  async function createLesson(data: { title: string; description?: string; order_no?: number }) {
    const row = await api.lessons.create(data)
    lessons.value.push(row)
    lessons.value.sort((a, b) => a.order_no - b.order_no || a.id - b.id)
    return row
  }

  async function updateLesson(id: number, data: { title?: string; description?: string; order_no?: number }) {
    await api.lessons.update(id, data)
    const item = lessons.value.find(l => l.id === id)
    if (item) {
      if (data.title) item.title = data.title
      if (data.description !== undefined) item.description = data.description
      if (data.order_no !== undefined) item.order_no = data.order_no
    }
    if (currentLesson.value?.id === id) {
      if (data.title) currentLesson.value.title = data.title
      if (data.description !== undefined) currentLesson.value.description = data.description
      if (data.order_no !== undefined) currentLesson.value.order_no = data.order_no
    }
  }

  async function deleteLesson(id: number) {
    await api.lessons.delete(id)
    lessons.value = lessons.value.filter(l => l.id !== id)
    if (currentLesson.value?.id === id) currentLesson.value = null
  }

  async function setProblems(id: number, problemIds: number[]) {
    await api.lessons.setProblems(id, problemIds)
    if (currentLesson.value?.id === id) {
      await fetchLesson(id)
    }
    const item = lessons.value.find(l => l.id === id)
    if (item) item.problem_count = problemIds.length
  }

  async function toggleRelease(lessonId: number, divisionId: number, isReleased: boolean) {
    const result = await api.lessons.toggleRelease(lessonId, divisionId, isReleased)
    if (currentLesson.value?.id === lessonId) {
      const rel = currentLesson.value.releases.find(r => r.division_id === divisionId)
      if (rel) {
        rel.is_released = isReleased
        rel.released_at = result.released_at
      }
    }
  }

  return {
    lessons, currentLesson, loading, error,
    fetchLessons, fetchLesson, createLesson, updateLesson,
    deleteLesson, setProblems, toggleRelease,
  }
})
