import { createRouter, createWebHistory } from 'vue-router'
import { api } from '@/api/client'
import { i18n } from '@/i18n'

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
      path: '/:pathMatch(.*)*',
      redirect: '/login',
    },
  ],
})

router.beforeEach(async (to) => {
  // setup 체크는 항상 먼저
  try {
    const status = await api.setup.status()

    if (status.locale) {
      i18n.global.locale.value = status.locale as 'ko' | 'en'
    }

    if (status.needs_setup && to.name !== 'setup') {
      return { name: 'setup' }
    }

    if (!status.needs_setup && to.name === 'setup') {
      return { name: 'login' }
    }
  } catch {
    if (to.name !== 'setup') return { name: 'setup' }
  }

  // 교사 인증 필요 라우트
  if (to.meta.requiresTeacherAuth) {
    try {
      await api.auth.meTeacher()
    } catch {
      return { name: 'login' }
    }
  }

  // 학생 인증 필요 라우트
  if (to.meta.requiresStudentAuth) {
    try {
      await api.auth.meStudent()
    } catch {
      return { name: 'login' }
    }
  }
})

export default router
