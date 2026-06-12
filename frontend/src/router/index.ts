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
      meta: { requiresAuth: true },
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: '/login',
    },
  ],
})

// 전역 네비게이션 가드: 최초 부팅 체크 + 인증 체크
router.beforeEach(async (to) => {
  try {
    const { needs_setup } = await api.setup.status()

    if (needs_setup && to.name !== 'setup') {
      return { name: 'setup' }
    }

    if (!needs_setup && to.name === 'setup') {
      return { name: 'login' }
    }

    if (to.meta.requiresAuth) {
      await api.auth.me()
    }
  } catch {
    if (to.meta.requiresAuth) {
      return { name: 'login' }
    }
  }
})

export default router
