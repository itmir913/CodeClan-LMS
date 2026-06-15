import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api, type ClassItem, type ClassDetail, type Subject } from '@/api/client'

export const useClassStore = defineStore('class', () => {
  const classes = ref<ClassItem[]>([])
  const subjects = ref<Subject[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchClasses() {
    loading.value = true
    error.value = null
    try {
      classes.value = await api.classes.list()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    } finally {
      loading.value = false
    }
  }

  async function fetchSubjects() {
    subjects.value = await api.subjects.list()
  }

  async function createClass(name: string, subject_id: number) {
    await api.classes.create(name, subject_id)
    await fetchClasses()
  }

  async function updateClass(id: number, name: string, subject_id: number) {
    await api.classes.update(id, name, subject_id)
    await fetchClasses()
  }

  async function deleteClass(id: number) {
    await api.classes.delete(id)
    await fetchClasses()
  }

  async function fetchClassDetail(id: number): Promise<ClassDetail> {
    return api.classes.get(id)
  }

  return {
    classes,
    subjects,
    loading,
    error,
    fetchClasses,
    fetchSubjects,
    createClass,
    updateClass,
    deleteClass,
    fetchClassDetail,
  }
})
