import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api, type AdminTeacher, type Subject } from '@/api/client'

export const useAdminStore = defineStore('admin', () => {
  const teachers = ref<AdminTeacher[]>([])
  const subjects = ref<Subject[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchTeachers() {
    loading.value = true
    error.value = null
    try {
      teachers.value = await api.admin.listTeachers()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    } finally {
      loading.value = false
    }
  }

  async function fetchSubjects() {
    subjects.value = await api.subjects.list()
  }

  async function createTeacher(data: {
    username: string
    name: string
    password: string
    role?: string
  }) {
    await api.admin.createTeacher(data)
    await fetchTeachers()
  }

  async function updateTeacher(
    id: number,
    data: { name?: string; role?: string; password?: string },
  ) {
    await api.admin.updateTeacher(id, data)
    await fetchTeachers()
  }

  async function deleteTeacher(id: number) {
    await api.admin.deleteTeacher(id)
    await fetchTeachers()
  }

  async function createSubject(name: string) {
    await api.admin.createSubject(name)
    await fetchSubjects()
  }

  async function deleteSubject(id: number) {
    await api.admin.deleteSubject(id)
    await fetchSubjects()
  }

  return {
    teachers,
    subjects,
    loading,
    error,
    fetchTeachers,
    fetchSubjects,
    createTeacher,
    updateTeacher,
    deleteTeacher,
    createSubject,
    deleteSubject,
  }
})
