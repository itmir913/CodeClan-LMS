import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api, type TeacherRow } from '@/api/client'

export const useTeacherStore = defineStore('teacher', () => {
  const teachers = ref<TeacherRow[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchTeachers() {
    loading.value = true
    error.value = null
    try {
      teachers.value = await api.teachers.list()
    } catch (e) {
      error.value = e instanceof Error ? e.message : '교사 목록을 불러오지 못했습니다'
    } finally {
      loading.value = false
    }
  }

  async function createTeacher(data: { username: string; name: string; password: string; role?: string }) {
    const row = await api.teachers.create(data)
    teachers.value.push(row)
    return row
  }

  async function updateTeacher(id: number, data: { name?: string; role?: string; password?: string }) {
    await api.teachers.update(id, data)
    const t = teachers.value.find(t => t.id === id)
    if (t) {
      if (data.name) t.name = data.name
      if (data.role) t.role = data.role
    }
  }

  async function deleteTeacher(id: number) {
    await api.teachers.delete(id)
    teachers.value = teachers.value.filter(t => t.id !== id)
  }

  return { teachers, loading, error, fetchTeachers, createTeacher, updateTeacher, deleteTeacher }
})
