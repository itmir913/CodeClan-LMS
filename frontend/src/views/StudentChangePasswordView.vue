<template>
  <div class="page-bg">
    <div class="card">
      <div class="card-header">
        <div class="card-title">비밀번호 변경</div>
        <div class="card-subtitle">
          {{ isForced ? '최초 로그인입니다. 반드시 비밀번호를 변경해주세요.' : '새 비밀번호를 입력해주세요.' }}
        </div>
      </div>

      <form @submit.prevent="onSubmit" class="form">
        <div class="field-group" v-if="!isForced">
          <label for="current-pw">현재 비밀번호</label>
          <input
            id="current-pw"
            v-model="form.current"
            type="password"
            autocomplete="current-password"
            :disabled="auth.loading"
          />
        </div>
        <div class="field-group">
          <label for="new-pw">새 비밀번호</label>
          <input
            id="new-pw"
            v-model="form.newPw"
            type="password"
            placeholder="6자 이상"
            autocomplete="new-password"
            :disabled="auth.loading"
          />
        </div>
        <div class="field-group">
          <label for="new-pw-confirm">새 비밀번호 확인</label>
          <input
            id="new-pw-confirm"
            v-model="form.confirm"
            type="password"
            autocomplete="new-password"
            :disabled="auth.loading"
          />
        </div>

        <div v-if="auth.error" class="error-msg">{{ auth.error }}</div>

        <button type="submit" class="primary submit-btn" :disabled="auth.loading">
          {{ auth.loading ? '변경 중...' : '비밀번호 변경' }}
        </button>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

const form = reactive({ current: '', newPw: '', confirm: '' })

const isForced = computed(() => auth.student?.password_reset_required ?? false)

onMounted(async () => {
  if (!auth.isStudentLoggedIn) {
    await auth.fetchStudentMe()
    if (!auth.isStudentLoggedIn) {
      router.replace({ name: 'login' })
    }
  }
})

async function onSubmit() {
  auth.clearError()
  if (form.newPw.length < 6) { auth.error = '비밀번호는 6자 이상이어야 합니다'; return }
  if (form.newPw !== form.confirm) { auth.error = '비밀번호가 일치하지 않습니다'; return }

  try {
    await auth.changeStudentPassword(form.current, form.newPw)
    router.replace({ name: 'student-home' })
  } catch {
    // error already set by store
  }
}
</script>

<style scoped>
.page-bg {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-background-secondary);
  padding: 2rem;
}

.card {
  width: 100%;
  max-width: 360px;
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-secondary);
  border-radius: var(--border-radius-lg);
  padding: 1.5rem;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
}

.card-header {
  margin-bottom: 1.5rem;
}

.card-title {
  font-weight: 600;
  font-size: 16px;
  margin-bottom: 6px;
}

.card-subtitle {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.form {
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
  margin-top: 8px;
  padding: 10px;
  font-size: 13px;
}
</style>
