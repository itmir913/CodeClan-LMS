<template>
  <div class="page-bg">
    <div class="card">
      <!-- 헤더 -->
      <div class="card-header">
        <svg class="icon-school" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path stroke-linecap="round" stroke-linejoin="round"
            d="M4.26 10.147a60.438 60.438 0 0 0-.491 6.347A48.627 48.627 0 0 1 12 20.904a48.627 48.627 0 0 1 8.232-4.41 60.46 60.46 0 0 0-.491-6.347m-15.482 0a50.57 50.57 0 0 0-2.658-.813A59.906 59.906 0 0 1 12 3.493a59.903 59.903 0 0 1 10.399 5.84c-.896.248-1.783.52-2.658.814m-15.482 0A50.717 50.717 0 0 1 12 13.489a50.702 50.702 0 0 1 3.741-3.342M6.75 15a.75.75 0 1 0 0-1.5.75.75 0 0 0 0 1.5Zm0 0v-3.675A55.378 55.378 0 0 1 12 8.443m-7.007 11.55A5.981 5.981 0 0 0 6.75 15.75v-1.5" />
        </svg>
        <div class="card-title">CodeClan LMS 초기 설정</div>
        <div class="card-subtitle">처음 실행 시 한 번만 설정합니다</div>
      </div>

      <form @submit.prevent="onSubmit">
        <!-- 학교 이름 -->
        <div class="field-group">
          <label for="school-name">학교 이름</label>
          <input
            id="school-name"
            v-model="form.schoolName"
            type="text"
            placeholder="예: 한국고등학교"
            autocomplete="off"
            :disabled="auth.loading"
          />
          <div class="field-hint">
            로그인 화면에 "환영합니다, OO고등학교 LMS입니다"로 표시됩니다
          </div>
        </div>

        <!-- 관리자 계정 섹션 -->
        <div class="section-divider">
          <div class="section-title">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" width="16" height="16">
              <path stroke-linecap="round" stroke-linejoin="round"
                d="M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z" />
            </svg>
            관리자 계정 생성
          </div>

          <div class="fields">
            <div class="field-group">
              <label for="admin-name">이름</label>
              <input
                id="admin-name"
                v-model="form.adminName"
                type="text"
                placeholder="예: 박교사"
                autocomplete="off"
                :disabled="auth.loading"
              />
            </div>

            <div class="field-group">
              <label for="admin-username">아이디</label>
              <input
                id="admin-username"
                v-model="form.adminUsername"
                type="text"
                placeholder="예: admin"
                autocomplete="off"
                :disabled="auth.loading"
              />
            </div>

            <div class="field-group">
              <label for="admin-password">비밀번호</label>
              <input
                id="admin-password"
                v-model="form.adminPassword"
                type="password"
                placeholder="8자 이상"
                autocomplete="new-password"
                :disabled="auth.loading"
              />
            </div>

            <div class="field-group">
              <label for="admin-password-confirm">비밀번호 확인</label>
              <input
                id="admin-password-confirm"
                v-model="form.adminPasswordConfirm"
                type="password"
                autocomplete="new-password"
                :disabled="auth.loading"
              />
            </div>
          </div>
        </div>

        <!-- 오류 메시지 -->
        <div v-if="displayError" class="error-msg">{{ displayError }}</div>

        <button type="submit" class="primary submit-btn" :disabled="auth.loading">
          <span v-if="auth.loading">설정 중...</span>
          <span v-else>✓ 설정 완료</span>
        </button>

        <div class="role-note">이 계정은 자동으로 "관리자" 권한으로 생성됩니다</div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

const form = reactive({
  schoolName: '',
  adminName: '',
  adminUsername: '',
  adminPassword: '',
  adminPasswordConfirm: '',
})

const localError = computed(() => {
  if (!form.schoolName.trim()) return null
  if (!form.adminName.trim()) return null
  if (!form.adminUsername.trim()) return null
  if (form.adminPassword.length > 0 && form.adminPassword.length < 8)
    return '비밀번호는 8자 이상이어야 합니다'
  if (form.adminPassword && form.adminPasswordConfirm && form.adminPassword !== form.adminPasswordConfirm)
    return '비밀번호가 일치하지 않습니다'
  return null
})

const displayError = computed(() => localError.value ?? auth.error)

async function onSubmit() {
  auth.clearError()

  if (!form.schoolName.trim()) { auth.error = '학교 이름을 입력해주세요'; return }
  if (!form.adminName.trim()) { auth.error = '관리자 이름을 입력해주세요'; return }
  if (!form.adminUsername.trim()) { auth.error = '아이디를 입력해주세요'; return }
  if (form.adminPassword.length < 8) { auth.error = '비밀번호는 8자 이상이어야 합니다'; return }
  if (form.adminPassword !== form.adminPasswordConfirm) { auth.error = '비밀번호가 일치하지 않습니다'; return }

  try {
    await auth.completeSetup({
      school_name: form.schoolName.trim(),
      admin_name: form.adminName.trim(),
      admin_username: form.adminUsername.trim(),
      admin_password: form.adminPassword,
    })
    router.replace({ name: 'login' })
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
  text-align: center;
  margin-bottom: 1.5rem;
}

.icon-school {
  width: 28px;
  height: 28px;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.card-title {
  font-weight: 500;
  font-size: 15px;
  margin-top: 8px;
}

.card-subtitle {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-top: 4px;
}

.field-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.field-hint {
  font-size: 11px;
  color: var(--color-text-tertiary);
  margin-top: 2px;
}

.section-divider {
  border-top: 0.5px solid var(--color-border-secondary);
  margin: 1rem 0;
  padding-top: 1rem;
}

.section-title {
  font-weight: 500;
  font-size: 13px;
  margin-bottom: 10px;
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--color-text-primary);
}

.fields {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.error-msg {
  background: var(--color-background-danger);
  color: var(--color-text-danger);
  font-size: 12px;
  padding: 8px 10px;
  border-radius: var(--border-radius-sm);
  margin-top: 0.75rem;
}

.submit-btn {
  width: 100%;
  margin-top: 1.25rem;
  padding: 10px;
  font-size: 13px;
}

.role-note {
  font-size: 11px;
  color: var(--color-text-tertiary);
  margin-top: 10px;
  text-align: center;
}
</style>
