import { createRouter, createWebHistory } from 'vue-router'
import { api } from '@/api/client'
import { i18n } from '@/i18n'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/setup',
      name: 'setup',
      component: () => import('@/views/setup/SetupView.vue'),
    },
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/auth/LoginView.vue'),
    },
    {
      path: '/teacher',
      name: 'teacher-home',
      component: () => import('@/views/teacher/TeacherHomeView.vue'),
      meta: { requiresAuth: 'teacher' },
    },
    {
      path: '/admin',
      name: 'admin-home',
      component: () => import('@/views/admin/AdminView.vue'),
      meta: { requiresAuth: 'admin' },
    },
    {
      path: '/classes/:id',
      name: 'class-detail',
      component: () => import('@/views/teacher/ClassDetailView.vue'),
      meta: { requiresAuth: 'teacher' },
    },
    {
      path: '/student',
      name: 'student-home',
      component: () => import('@/views/student/StudentHomeView.vue'),
      meta: { requiresAuth: 'student' },
    },
    {
      path: '/student/change-password',
      name: 'student-change-password',
      component: () => import('@/views/student/StudentChangePasswordView.vue'),
      meta: { requiresAuth: 'student' },
    },
    {
      path: '/problem-bank',
      name: 'problem-bank',
      component: () => import('@/views/teacher/ProblemBankView.vue'),
      meta: { requiresAuth: 'teacher' },
    },
    {
      path: '/problem-bank/new',
      name: 'problem-new',
      component: () => import('@/views/teacher/ProblemFormView.vue'),
      meta: { requiresAuth: 'teacher' },
    },
    {
      path: '/problem-bank/:id/edit',
      name: 'problem-edit',
      component: () => import('@/views/teacher/ProblemFormView.vue'),
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
      const res = await api.auth.meTeacher()
      i18n.global.locale.value = res.locale as 'ko' | 'en'
      // admin이 /teacher 홈에 오면 /admin으로 (공유 페이지인 classes, problem-bank는 그대로 허용)
      if (to.name === 'teacher-home' && res.user.role === 'admin') {
        return { name: 'admin-home' }
      }
      // teacher(non-admin)가 admin 전용 페이지에 오면 /teacher로
      if (requiredRole === 'admin' && res.user.role !== 'admin') {
        return { name: 'teacher-home' }
      }
    } catch {
      // 교사 세션 없음 — 학생 세션이면 student 홈으로
      try {
        const res = await api.auth.meStudent()
        i18n.global.locale.value = res.locale as 'ko' | 'en'
        return { name: 'student-home' }
      } catch {
        return { name: 'login' }
      }
    }
  }

  if (requiredRole === 'student') {
    try {
      const res = await api.auth.meStudent()
      i18n.global.locale.value = res.locale as 'ko' | 'en'
    } catch {
      // 학생 세션 없음 — 교사/관리자 세션이면 해당 홈으로
      try {
        const res = await api.auth.meTeacher()
        i18n.global.locale.value = res.locale as 'ko' | 'en'
        return res.user.role === 'admin' ? { name: 'admin-home' } : { name: 'teacher-home' }
      } catch {
        return { name: 'login' }
      }
    }
  }
})

export default router
