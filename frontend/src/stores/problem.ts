import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  api,
  type ProblemListItem,
  type ProblemDetail,
  type CreateProblemBody,
} from '@/api/client'

export const useProblemStore = defineStore('problem', () => {
  const problems = ref<ProblemListItem[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchProblems(type?: string) {
    loading.value = true
    error.value = null
    try {
      problems.value = await api.problems.list(type)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    } finally {
      loading.value = false
    }
  }

  async function getProblem(id: number): Promise<ProblemDetail> {
    return await api.problems.get(id)
  }

  async function createProblem(data: CreateProblemBody): Promise<number> {
    const result = await api.problems.create(data)
    await fetchProblems()
    return result.id
  }

  async function updateProblem(id: number, data: CreateProblemBody): Promise<void> {
    await api.problems.update(id, data)
    await fetchProblems()
  }

  async function deleteProblem(id: number): Promise<void> {
    await api.problems.delete(id)
    await fetchProblems()
  }

  return {
    problems,
    loading,
    error,
    fetchProblems,
    getProblem,
    createProblem,
    updateProblem,
    deleteProblem,
  }
})
