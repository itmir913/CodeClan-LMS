import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api, type StudentItem, type AddStudentBody, type BulkResult, type ImportStudentRow } from '@/api/client'

export const useStudentStore = defineStore('student', () => {
  const students = ref<StudentItem[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const currentClassId = ref<number | null>(null)

  async function fetchStudents(classId: number) {
    loading.value = true
    error.value = null
    currentClassId.value = classId
    try {
      students.value = await api.students.list(classId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    } finally {
      loading.value = false
    }
  }

  async function addStudent(classId: number, data: AddStudentBody): Promise<void> {
    await api.students.add(classId, data)
    await fetchStudents(classId)
  }

  async function bulkAddStudents(classId: number, data: AddStudentBody[]): Promise<BulkResult> {
    const result = await api.students.bulkAdd(classId, data)
    await fetchStudents(classId)
    return result
  }

  async function deleteStudent(classId: number, studentId: number): Promise<void> {
    await api.students.delete(studentId)
    await fetchStudents(classId)
  }

  async function importStudents(classId: number, data: ImportStudentRow[]): Promise<void> {
    await api.students.importStudents(classId, data)
    await fetchStudents(classId)
  }

  async function resetStudentPassword(studentId: number): Promise<void> {
    await api.students.resetPassword(studentId)
    if (currentClassId.value !== null) {
      students.value = students.value.map((s) =>
        s.id === studentId ? { ...s, password_reset_required: true } : s,
      )
    }
  }

  return {
    students,
    loading,
    error,
    currentClassId,
    fetchStudents,
    addStudent,
    bulkAddStudents,
    importStudents,
    deleteStudent,
    resetStudentPassword,
  }
})
