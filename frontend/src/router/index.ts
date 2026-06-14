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
      path: '/:pathMatch(.*)*',
      redirect: '/login',
    },
  ],
})

router.beforeEach(async (to) => {
  try {
    const { needs_setup } = await api.setup.status()

    if (needs_setup && to.name !== 'setup') {
      return { name: 'setup' }
    }

    if (!needs_setup && to.name === 'setup') {
      return { name: 'login' }
    }
  } catch {
    if (to.name !== 'setup') {
      return { name: 'setup' }
    }
  }
})

export default router
