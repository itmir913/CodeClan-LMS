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
      // 역할 불일치 시 로그아웃 후 로그인 화면으로
      if (user.role !== requiredRole) {
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
