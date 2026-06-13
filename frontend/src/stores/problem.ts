import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api, type ProblemListItem, type ProblemRow, type CreateProblemInput, type UpdateProblemInput } from '@/api/client'

export const useProblemStore = defineStore('problem', () => {
  const problems = ref<ProblemListItem[]>([])
  const currentProblem = ref<ProblemRow | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchProblems(params?: { problem_type?: number; q?: string }) {
    loading.value = true
    error.value = null
    try {
      problems.value = await api.problems.list(params)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '문제 목록을 불러오지 못했습니다'
    } finally {
      loading.value = false
    }
  }

  async function fetchProblem(id: number) {
    loading.value = true
    error.value = null
    try {
      currentProblem.value = await api.problems.get(id)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '문제를 불러오지 못했습니다'
      currentProblem.value = null
    } finally {
      loading.value = false
    }
  }

  async function createProblem(data: CreateProblemInput) {
    const row = await api.problems.create(data)
    problems.value.unshift({
      id: row.id,
      problem_type: row.problem_type,
      title: row.title,
      is_structure_check: row.is_structure_check,
      created_at: row.created_at,
    })
    return row
  }

  async function updateProblem(id: number, data: UpdateProblemInput) {
    await api.problems.update(id, data)
    const item = problems.value.find(p => p.id === id)
    if (item) {
      if (data.title) item.title = data.title
      if (data.is_structure_check !== undefined) item.is_structure_check = data.is_structure_check
    }
    if (currentProblem.value?.id === id) {
      if (data.title) currentProblem.value.title = data.title
      if (data.description !== undefined) currentProblem.value.description = data.description
      if (data.type_config !== undefined) currentProblem.value.type_config = data.type_config
      if (data.is_structure_check !== undefined) currentProblem.value.is_structure_check = data.is_structure_check
    }
  }

  async function deleteProblem(id: number) {
    await api.problems.delete(id)
    problems.value = problems.value.filter(p => p.id !== id)
    if (currentProblem.value?.id === id) currentProblem.value = null
  }

  return {
    problems,
    currentProblem,
    loading,
    error,
    fetchProblems,
    fetchProblem,
    createProblem,
    updateProblem,
    deleteProblem,
  }
})
