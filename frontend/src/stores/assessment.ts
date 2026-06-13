import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api, type AssessmentRow, type AssessmentDetail } from '@/api/client'

export const useAssessmentStore = defineStore('assessment', () => {
  const assessments = ref<AssessmentRow[]>([])
  const currentAssessment = ref<AssessmentDetail | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchAssessments() {
    loading.value = true
    error.value = null
    try {
      assessments.value = await api.assessments.list()
    } catch (e) {
      error.value = e instanceof Error ? e.message : '수행평가 목록을 불러오지 못했습니다'
    } finally {
      loading.value = false
    }
  }

  async function fetchAssessment(id: number) {
    loading.value = true
    error.value = null
    try {
      currentAssessment.value = await api.assessments.get(id)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '수행평가를 불러오지 못했습니다'
      currentAssessment.value = null
    } finally {
      loading.value = false
    }
  }

  async function createAssessment(data: { title: string; description?: string }) {
    const row = await api.assessments.create(data)
    assessments.value.unshift(row)
    return row
  }

  async function updateAssessment(id: number, data: { title?: string; description?: string }) {
    await api.assessments.update(id, data)
    const item = assessments.value.find(a => a.id === id)
    if (item) {
      if (data.title) item.title = data.title
      if (data.description !== undefined) item.description = data.description
    }
    if (currentAssessment.value?.id === id) {
      if (data.title) currentAssessment.value.title = data.title
      if (data.description !== undefined) currentAssessment.value.description = data.description
    }
  }

  async function deleteAssessment(id: number) {
    await api.assessments.delete(id)
    assessments.value = assessments.value.filter(a => a.id !== id)
    if (currentAssessment.value?.id === id) currentAssessment.value = null
  }

  async function setProblems(id: number, items: Array<{ problem_id: number; score: number }>) {
    await api.assessments.setProblems(id, items)
    if (currentAssessment.value?.id === id) await fetchAssessment(id)
    const item = assessments.value.find(a => a.id === id)
    if (item) item.problem_count = items.length
  }

  async function linkDivision(id: number, divisionId: number) {
    await api.assessments.linkDivision(id, divisionId)
    if (currentAssessment.value?.id === id) await fetchAssessment(id)
    const item = assessments.value.find(a => a.id === id)
    if (item) item.division_count += 1
  }

  async function unlinkDivision(id: number, divisionId: number) {
    await api.assessments.unlinkDivision(id, divisionId)
    if (currentAssessment.value?.id === id) {
      currentAssessment.value.divisions = currentAssessment.value.divisions.filter(d => d.division_id !== divisionId)
    }
    const item = assessments.value.find(a => a.id === id)
    if (item && item.division_count > 0) item.division_count -= 1
  }

  return {
    assessments, currentAssessment, loading, error,
    fetchAssessments, fetchAssessment, createAssessment, updateAssessment,
    deleteAssessment, setProblems, linkDivision, unlinkDivision,
  }
})
