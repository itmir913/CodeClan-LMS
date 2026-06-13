<template>
  <div class="page-bg">
    <div v-if="loading" class="loading">로딩 중...</div>
    <div v-else-if="error" class="error-banner">{{ error }}</div>
    <div v-else class="container">
      <header class="header">
        <span class="school-name">{{ auth.schoolName }}</span>
        <div class="user-info">
          <span>{{ auth.student?.name }}</span>
          <button class="logout-btn" @click="logout">로그아웃</button>
        </div>
      </header>
      <main class="main">
        <p class="placeholder">학생 학습 화면이 구현 예정입니다.</p>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()
const loading = ref(false)
const error = ref<string | null>(null)

onMounted(async () => {
  loading.value = true
  try {
    if (!auth.isStudentLoggedIn) {
      await auth.fetchStudentMe()
      if (!auth.isStudentLoggedIn) {
        router.replace({ name: 'login' })
        return
      }
    }
    if (auth.student?.password_reset_required) {
      router.replace({ name: 'student-change-password' })
    }
    await auth.fetchSchoolName()
  } catch (e) {
    error.value = e instanceof Error ? e.message : '오류가 발생했습니다'
  } finally {
    loading.value = false
  }
})

async function logout() {
  await auth.logoutStudent()
  router.replace({ name: 'login' })
}
</script>

<style scoped>
.page-bg {
  min-height: 100vh;
  background: var(--color-background-secondary);
}

.loading, .error-banner {
  padding: 2rem;
  text-align: center;
}

.error-banner {
  background: var(--color-background-danger);
  color: var(--color-text-danger);
}

.container {
  max-width: 900px;
  margin: 0 auto;
  padding: 1.5rem;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--color-border-secondary);
  margin-bottom: 1.5rem;
}

.school-name {
  font-weight: 600;
  font-size: 15px;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 13px;
}

.logout-btn {
  font-size: 12px;
  color: var(--color-text-secondary);
  border: none;
  background: none;
  cursor: pointer;
  padding: 4px 8px;
}

.logout-btn:hover {
  color: var(--color-text-primary);
}

.placeholder {
  color: var(--color-text-secondary);
  font-size: 14px;
}
</style>
