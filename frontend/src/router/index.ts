import { createRouter, createWebHistory } from 'vue-router'
import { api } from '@/api/client'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/setup',
      name: 'setup',
      component: () => import('@/views/SetupView.vue'),
    },
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/LoginView.vue'),
    },
    {
      path: '/dashboard',
      name: 'dashboard',
      component: () => import('@/views/DashboardView.vue'),
      meta: { requiresTeacherAuth: true },
    },
    {
      path: '/assessments',
      name: 'assessment-management',
      component: () => import('@/views/AssessmentManagementView.vue'),
      meta: { requiresTeacherAuth: true },
    },
    {
      path: '/lessons',
      name: 'lesson-management',
      component: () => import('@/views/LessonManagementView.vue'),
      meta: { requiresTeacherAuth: true },
    },
    {
      path: '/problems',
      name: 'problem-bank',
      component: () => import('@/views/ProblemBankView.vue'),
      meta: { requiresTeacherAuth: true },
    },
    {
      path: '/divisions',
      name: 'division-management',
      component: () => import('@/views/DivisionManagementView.vue'),
      meta: { requiresTeacherAuth: true },
    },
    {
      path: '/teachers',
      name: 'teacher-management',
      component: () => import('@/views/TeacherManagementView.vue'),
      meta: { requiresTeacherAuth: true },
    },
    {
      path: '/sessions',
      name: 'session-management',
      component: () => import('@/views/SessionManagementView.vue'),
      meta: { requiresTeacherAuth: true },
    },
    {
      path: '/sessions/:id/grading',
      name: 'session-grading',
      component: () => import('@/views/SessionGradingView.vue'),
      meta: { requiresTeacherAuth: true },
    },
    {
      path: '/lessons/:lessonId/live/:divisionId',
      name: 'lesson-live',
      component: () => import('@/views/LessonLiveView.vue'),
      meta: { requiresTeacherAuth: true },
    },
    {
      path: '/student/lessons/:id',
      name: 'student-lesson',
      component: () => import('@/views/StudentLessonView.vue'),
      meta: { requiresStudentAuth: true },
    },
    {
      path: '/student',
      name: 'student-home',
      component: () => import('@/views/StudentHomeView.vue'),
      meta: { requiresStudentAuth: true },
    },
    {
      path: '/student/change-password',
      name: 'student-change-password',
      component: () => import('@/views/StudentChangePasswordView.vue'),
      meta: { requiresStudentAuth: true },
    },
    {
      path: '/student/result/:sessionId',
      name: 'student-result',
      component: () => import('@/views/StudentResultView.vue'),
      meta: { requiresStudentAuth: true },
    },
    {
      path: '/audit-logs',
      name: 'audit-logs',
      component: () => import('@/views/AuditLogView.vue'),
      meta: { requiresTeacherAuth: true },
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: '/login',
    },
  ],
})

// 전역 네비게이션 가드: 초기 설정 체크 + 인증 체크
router.beforeEach(async (to) => {
  try {
    const { needs_setup } = await api.setup.status()

    if (needs_setup && to.name !== 'setup') {
      return { name: 'setup' }
    }

    if (!needs_setup && to.name === 'setup') {
      return { name: 'login' }
    }

    if (to.meta.requiresTeacherAuth) {
      await api.auth.me()
    }

    if (to.meta.requiresStudentAuth) {
      const studentUser = await api.auth.studentMe()
      if (studentUser.password_reset_required && to.name !== 'student-change-password') {
        return { name: 'student-change-password' }
      }
    }
  } catch {
    if (to.meta.requiresTeacherAuth || to.meta.requiresStudentAuth) {
      return { name: 'login' }
    }
  }
})

export default router
