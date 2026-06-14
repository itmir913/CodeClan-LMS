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
      path: '/teacher',
      name: 'teacher-home',
      component: () => import('@/views/TeacherHomeView.vue'),
      meta: { requiresAuth: 'teacher' },
    },
    {
      path: '/admin',
      name: 'admin-home',
      component: () => import('@/views/AdminView.vue'),
      meta: { requiresAuth: 'admin' },
    },
    {
      path: '/classes/:id',
      name: 'class-detail',
      component: () => import('@/views/ClassDetailView.vue'),
      meta: { requiresAuth: 'teacher' },
    },
    {
      path: '/student',
      name: 'student-home',
      component: () => import('@/views/StudentHomeView.vue'),
      meta: { requiresAuth: 'student' },
    },
    {
      path: '/student/change-password',
      name: 'student-change-password',
      component: () => import('@/views/StudentChangePasswordView.vue'),
      meta: { requiresAuth: 'student' },
    },
    {
      path: '/problem-bank',
      name: 'problem-bank',
      component: () => import('@/views/ProblemBankView.vue'),
      meta: { requiresAuth: 'teacher' },
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

  const requiredRole = to.meta.requiresAuth as string | undefined

  if (requiredRole === 'admin' || requiredRole === 'teacher') {
    try {
      const user = await api.auth.meTeacher()
      if (requiredRole === 'admin' && user.role !== 'admin') {
        // admin 전용 페이지 — teacher는 접근 불가
        await api.auth.logoutTeacher().catch(() => {})
        return { name: 'login' }
      } else if (requiredRole === 'teacher' && user.role !== 'teacher' && user.role !== 'admin') {
        // teacher 페이지 — admin도 허용
        await api.auth.logoutTeacher().catch(() => {})
        return { name: 'login' }
      }
    } catch {
      return { name: 'login' }
    }
  }

  if (requiredRole === 'student') {
    try {
      await api.auth.meStudent()
    } catch {
      return { name: 'login' }
    }
  }
})

export default router
