import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '@/api/client'

export interface ActiveSession {
  id: number
  assessment_title: string
  status: 'RUNNING' | 'LOBBY'
  submission_count: number
  student_count: number
  time_limit_min: number | null
  start_at: string | null
}

export interface DivisionCard {
  id: number
  name: string
  student_count: number
  active_session: ActiveSession | null
}

export interface DashboardStats {
  problem_count: number
  lesson_count: number
  assessment_count: number
}

export interface RecentLog {
  id: number
  action_type: string
  detail: string | null
  created_at: string
}

export interface DashboardData {
  teacher_name: string
  teacher_role: string
  division_count: number
  divisions: DivisionCard[]
  stats: DashboardStats
  recent_logs: RecentLog[]
}

export const useDashboardStore = defineStore('dashboard', () => {
  const data = ref<DashboardData | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetch() {
    loading.value = true
    error.value = null
    try {
      data.value = await api.dashboard.get()
    } catch (e) {
      error.value = e instanceof Error ? e.message : '데이터를 불러오지 못했습니다'
    } finally {
      loading.value = false
    }
  }

  return { data, loading, error, fetch }
})
