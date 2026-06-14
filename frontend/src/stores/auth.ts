import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api, type SetupRequest } from '@/api/client'

export const useAuthStore = defineStore('auth', () => {
  const schoolName = ref<string>('')
  const loading = ref(false)
  const error = ref<string | null>(null)

  function clearError() {
    error.value = null
  }

  async function completeSetup(data: SetupRequest) {
    loading.value = true
    error.value = null
    try {
      await api.setup.complete(data)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '설정 중 오류가 발생했습니다'
      throw e
    } finally {
      loading.value = false
    }
  }

  return {
    schoolName,
    loading,
    error,
    clearError,
    completeSetup,
  }
})
