<template>
  <div class="page-bg">
    <div class="card">
      <div class="card-header">
        <div class="welcome-msg" v-if="schoolName">
          환영합니다, {{ schoolName }} LMS입니다
        </div>
        <div class="welcome-msg" v-else>CodeClan LMS</div>
      </div>

      <!-- 탭: 교사 / 학생 -->
      <div class="tabs">
        <button
          class="tab"
          :class="{ active: tab === 'teacher' }"
          @click="tab = 'teacher'"
          type="button"
        >
          교사 로그인
        </button>
        <button
          class="tab"
          :class="{ active: tab === 'student' }"
          @click="tab = 'student'"
          type="button"
        >
          학생 로그인
        </button>
      </div>

      <!-- 교사 로그인 폼 -->
      <form v-if="tab === 'teacher'" @submit.prevent="teacherLogin" class="login-form">
        <div class="field-group">
          <label for="t-username">아이디</label>
          <input
            id="t-username"
            v-model="teacher.username"
            type="text"
            placeholder="아이디"
            autocomplete="username"
            :disabled="loading"
          />
        </div>
        <div class="field-group">
          <label for="t-password">비밀번호</label>
          <input
            id="t-password"
            v-model="teacher.password"
            type="password"
            placeholder="비밀번호"
            autocomplete="current-password"
            :disabled="loading"
          />
        </div>

        <div v-if="error" class="error-msg">{{ error }}</div>

        <button type="submit" class="primary submit-btn" :disabled="loading">
          {{ loading ? '로그인 중...' : '로그인' }}
        </button>
      </form>

      <!-- 학생 로그인 폼 -->
      <form v-else @submit.prevent="studentLogin" class="login-form">
        <div class="field-group">
          <label for="s-number">학번</label>
          <input
            id="s-number"
            v-model="student.studentNumber"
            type="text"
            placeholder="학번 (예: 10101)"
            autocomplete="off"
            :disabled="loading"
          />
        </div>
        <div class="field-group">
          <label for="s-birth">생년월일</label>
          <input
            id="s-birth"
            v-model="student.birthDate"
            type="text"
            placeholder="YYYYMMDD (예: 20090315)"
            autocomplete="bday"
            maxlength="8"
            :disabled="loading"
          />
        </div>

        <div v-if="error" class="error-msg">{{ error }}</div>

        <button type="submit" class="primary submit-btn" :disabled="loading">
          {{ loading ? '로그인 중...' : '입장' }}
        </button>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { api } from '@/api/client'

const router = useRouter()

const tab = ref<'teacher' | 'student'>('teacher')
const loading = ref(false)
const error = ref('')
const schoolName = ref('')

const teacher = reactive({ username: '', password: '' })
const student = reactive({ studentNumber: '', birthDate: '' })

onMounted(async () => {
  try {
    const res = await api.auth.schoolName()
    schoolName.value = res.school_name
  } catch {
    // 무시
  }
})

async function teacherLogin() {
  error.value = ''
  if (!teacher.username || !teacher.password) {
    error.value = '아이디와 비밀번호를 입력해주세요'
    return
  }
  loading.value = true
  try {
    await api.auth.loginTeacher(teacher.username, teacher.password)
    router.replace({ name: 'dashboard' })
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : '로그인 실패'
  } finally {
    loading.value = false
  }
}

async function studentLogin() {
  error.value = ''
  if (!student.studentNumber || !student.birthDate) {
    error.value = '학번과 생년월일을 입력해주세요'
    return
  }
  // TODO: 학생 로그인 API 구현 후 연결
  error.value = '학생 로그인은 준비 중입니다'
}
</script>

<style scoped>
.page-bg {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-background-secondary);
}

.card {
  width: 100%;
  max-width: 340px;
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-secondary);
  border-radius: var(--border-radius-lg);
  padding: 1.5rem;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
}

.card-header {
  text-align: center;
  margin-bottom: 1.25rem;
}

.welcome-msg {
  font-size: 15px;
  font-weight: 500;
  color: var(--color-text-primary);
}

.tabs {
  display: flex;
  gap: 0;
  border: 1px solid var(--color-border-primary);
  border-radius: var(--border-radius-sm);
  overflow: hidden;
  margin-bottom: 1.25rem;
}

.tab {
  flex: 1;
  padding: 8px;
  font-size: 12px;
  font-weight: 500;
  border: none;
  border-radius: 0;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}

.tab.active {
  background: var(--color-accent);
  color: #fff;
}

.tab:hover:not(.active) {
  background: var(--color-background-secondary);
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.field-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.error-msg {
  background: var(--color-background-danger);
  color: var(--color-text-danger);
  font-size: 12px;
  padding: 8px 10px;
  border-radius: var(--border-radius-sm);
}

.submit-btn {
  width: 100%;
  margin-top: 4px;
  padding: 10px;
  font-size: 13px;
}
</style>
