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
      path: '/:pathMatch(.*)*',
      redirect: '/login',
    },
  ],
})

router.beforeEach(async (to) => {
  try {
    const status = await api.setup.status()

    // 서버에 저장된 locale이 있으면 앱 전체 기본 언어로 적용
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
    if (to.name !== 'setup') {
      return { name: 'setup' }
    }
  }
})

export default router
