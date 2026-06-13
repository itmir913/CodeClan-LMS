import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api, type DivisionRow, type StudentRow, type TeacherBrief } from '@/api/client'

export const useDivisionStore = defineStore('division', () => {
  const divisions = ref<DivisionRow[]>([])
  const selectedDivisionId = ref<number | null>(null)
  const students = ref<StudentRow[]>([])
  const divisionTeachers = ref<TeacherBrief[]>([])

  const loading = ref(false)
  const error = ref<string | null>(null)

  function clearError() {
    error.value = null
  }

  async function fetchDivisions() {
    loading.value = true
    error.value = null
    try {
      divisions.value = await api.divisions.list()
    } catch (e) {
      error.value = e instanceof Error ? e.message : '분반 목록을 불러오지 못했습니다'
    } finally {
      loading.value = false
    }
  }

  async function createDivision(name: string) {
    const row = await api.divisions.create(name)
    divisions.value.push(row)
    return row
  }

  async function updateDivision(id: number, name: string) {
    await api.divisions.update(id, name)
    const div = divisions.value.find(d => d.id === id)
    if (div) div.name = name
  }

  async function deleteDivision(id: number) {
    await api.divisions.delete(id)
    divisions.value = divisions.value.filter(d => d.id !== id)
    if (selectedDivisionId.value === id) {
      selectedDivisionId.value = null
      students.value = []
    }
  }

  async function selectDivision(id: number) {
    selectedDivisionId.value = id
    students.value = []
    divisionTeachers.value = []
    loading.value = true
    error.value = null
    try {
      const [s, t] = await Promise.all([
        api.divisions.getStudents(id),
        api.divisions.getTeachers(id).catch(() => [] as TeacherBrief[]),
      ])
      students.value = s
      divisionTeachers.value = t
    } catch (e) {
      error.value = e instanceof Error ? e.message : '학생 목록을 불러오지 못했습니다'
    } finally {
      loading.value = false
    }
  }

  async function addStudent(divisionId: number, data: { student_number: string; name: string; password: string }) {
    const row = await api.divisions.addStudent(divisionId, data)
    if (selectedDivisionId.value === divisionId) {
      students.value.push(row)
      const div = divisions.value.find(d => d.id === divisionId)
      if (div) div.student_count += 1
    }
    return row
  }

  async function bulkImport(divisionId: number, items: Array<{ student_number: string; name: string; password: string }>) {
    const result = await api.divisions.bulkImport(divisionId, items)
    if (result.inserted > 0) {
      await selectDivision(divisionId)
      const div = divisions.value.find(d => d.id === divisionId)
      if (div) div.student_count += result.inserted
    }
    return result
  }

  async function deleteStudent(studentId: number) {
    await api.students.delete(studentId)
    students.value = students.value.filter(s => s.id !== studentId)
    if (selectedDivisionId.value !== null) {
      const div = divisions.value.find(d => d.id === selectedDivisionId.value)
      if (div && div.student_count > 0) div.student_count -= 1
    }
  }

  async function resetStudentPassword(studentId: number, newPassword: string) {
    await api.students.resetPassword(studentId, newPassword)
  }

  async function setDivisionTeachers(divisionId: number, teacherIds: number[]) {
    await api.divisions.setTeachers(divisionId, teacherIds)
    if (selectedDivisionId.value === divisionId) {
      divisionTeachers.value = await api.divisions.getTeachers(divisionId)
    }
    const div = divisions.value.find(d => d.id === divisionId)
    if (div) div.teacher_count = teacherIds.length
  }

  return {
    divisions,
    selectedDivisionId,
    students,
    divisionTeachers,
    loading,
    error,
    clearError,
    fetchDivisions,
    createDivision,
    updateDivision,
    deleteDivision,
    selectDivision,
    addStudent,
    bulkImport,
    deleteStudent,
    resetStudentPassword,
    setDivisionTeachers,
  }
})
