import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api, type SessionRow, type CreateSessionInput } from '@/api/client'

export const useSessionStore = defineStore('session', () => {
  const sessions = ref<SessionRow[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchSessions(params?: { division_id?: number; assessment_id?: number; status?: string }) {
    loading.value = true
    error.value = null
    try {
      sessions.value = await api.sessions.list(params)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '세션 목록을 불러오지 못했습니다'
    } finally {
      loading.value = false
    }
  }

  async function createSession(data: CreateSessionInput): Promise<SessionRow> {
    const row = await api.sessions.create(data)
    sessions.value.unshift(row)
    return row
  }

  async function transition(id: number, action: string) {
    const res = await api.sessions.transition(id, action)
    const s = sessions.value.find(s => s.id === id)
    if (s) s.status = res.status as SessionRow['status']
    return res
  }

  async function pause(id: number) {
    const res = await api.sessions.pause(id)
    const s = sessions.value.find(s => s.id === id)
    if (s) s.is_paused = res.is_paused
    return res
  }

  async function toggleResultRelease(id: number) {
    const res = await api.sessions.toggleResultRelease(id)
    const s = sessions.value.find(s => s.id === id)
    if (s) s.is_result_released = res.is_result_released
    return res
  }

  return {
    sessions, loading, error,
    fetchSessions, createSession, transition, pause, toggleResultRelease,
  }
})
