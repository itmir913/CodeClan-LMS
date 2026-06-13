import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api, type TeacherUser, type StudentUser } from '@/api/client'

export interface SetupData {
  school_name: string
  admin_name: string
  admin_username: string
  admin_password: string
}

export const useAuthStore = defineStore('auth', () => {
  const teacher = ref<TeacherUser | null>(null)
  const student = ref<StudentUser | null>(null)
  const schoolName = ref<string>('')
  const loading = ref(false)
  const error = ref<string | null>(null)

  const isTeacherLoggedIn = computed(() => teacher.value !== null)
  const isStudentLoggedIn = computed(() => student.value !== null)
  const isAdmin = computed(() => teacher.value?.role === 'admin')

  function clearError() {
    error.value = null
  }

  async function fetchSchoolName() {
    try {
      const res = await api.auth.schoolName()
      schoolName.value = res.school_name
    } catch {
      // 학교 이름 조회 실패는 조용히 무시
    }
  }

  async function loginTeacher(username: string, password: string) {
    loading.value = true
    error.value = null
    try {
      const res = await api.auth.loginTeacher(username, password)
      teacher.value = res.user
      await fetchSchoolName()
    } catch (e) {
      error.value = e instanceof Error ? e.message : '로그인에 실패했습니다'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function fetchTeacherMe() {
    try {
      teacher.value = await api.auth.me()
      await fetchSchoolName()
    } catch {
      teacher.value = null
    }
  }

  async function logoutTeacher() {
    loading.value = true
    try {
      await api.auth.logout()
    } finally {
      teacher.value = null
      loading.value = false
    }
  }

  async function loginStudent(studentNumber: string, password: string) {
    loading.value = true
    error.value = null
    try {
      const res = await api.auth.loginStudent(studentNumber, password)
      student.value = res.user
      return res.user
    } catch (e) {
      error.value = e instanceof Error ? e.message : '로그인에 실패했습니다'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function fetchStudentMe() {
    try {
      student.value = await api.auth.studentMe()
    } catch {
      student.value = null
    }
  }

  async function logoutStudent() {
    loading.value = true
    try {
      await api.auth.logoutStudent()
    } finally {
      student.value = null
      loading.value = false
    }
  }

  async function changeStudentPassword(currentPassword: string, newPassword: string) {
    loading.value = true
    error.value = null
    try {
      await api.auth.studentChangePassword(currentPassword, newPassword)
      if (student.value) {
        student.value = { ...student.value, password_reset_required: false }
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '비밀번호 변경에 실패했습니다'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function completeSetup(data: SetupData) {
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
    teacher,
    student,
    schoolName,
    loading,
    error,
    isTeacherLoggedIn,
    isStudentLoggedIn,
    isAdmin,
    completeSetup,
    clearError,
    fetchSchoolName,
    loginTeacher,
    fetchTeacherMe,
    logoutTeacher,
    loginStudent,
    fetchStudentMe,
    logoutStudent,
    changeStudentPassword,
  }
})
