import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api, type SetupRequest, type TeacherUser, type StudentUser } from '@/api/client'
import { i18n } from '@/i18n'

export const useAuthStore = defineStore('auth', () => {
  const schoolName = ref<string>('')
  const teacher = ref<TeacherUser | null>(null)
  const student = ref<StudentUser | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  function clearError() {
    error.value = null
  }

  function applyLocale(locale: string) {
    if (locale) {
      i18n.global.locale.value = locale as 'ko' | 'en'
    }
  }

  async function fetchSchoolName() {
    try {
      const res = await api.auth.schoolName()
      schoolName.value = res.school_name
    } catch {
      // 로그인 화면 렌더링을 막지 않음
    }
  }

  async function completeSetup(data: SetupRequest) {
    loading.value = true
    error.value = null
    try {
      await api.setup.complete(data)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'ERR_UNKNOWN'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function loginTeacher(username: string, password: string) {
    const res = await api.auth.loginTeacher(username, password)
    teacher.value = res.user
    student.value = null
    applyLocale(res.locale)
  }

  async function logoutTeacher() {
    await api.auth.logoutTeacher()
    teacher.value = null
  }

  async function fetchTeacherMe() {
    teacher.value = await api.auth.meTeacher()
  }

  async function loginStudent(username: string, password: string) {
    const res = await api.auth.loginStudent(username, password)
    student.value = res.user
    teacher.value = null
    applyLocale(res.locale)
  }

  async function logoutStudent() {
    await api.auth.logoutStudent()
    student.value = null
  }

  async function fetchStudentMe() {
    student.value = await api.auth.meStudent()
  }

  async function changePasswordStudent(currentPassword: string | null, newPassword: string) {
    await api.auth.changePasswordStudent(currentPassword, newPassword)
    if (student.value) {
      student.value = { ...student.value, password_reset_required: false }
    }
  }

  return {
    schoolName,
    teacher,
    student,
    loading,
    error,
    clearError,
    fetchSchoolName,
    completeSetup,
    loginTeacher,
    logoutTeacher,
    fetchTeacherMe,
    loginStudent,
    logoutStudent,
    fetchStudentMe,
    changePasswordStudent,
  }
})
