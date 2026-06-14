<template>
  <div class="min-h-screen flex items-center justify-center px-4"
       style="background: var(--color-bg-primary)">
    <div class="w-full max-w-md rounded-xl border p-8"
         style="background: var(--color-bg-secondary);
                border-color: var(--color-border);
                box-shadow: var(--shadow-card)">

      <h1 class="text-xl font-semibold mb-2" style="color: var(--color-text-primary)">
        {{ $t('auth.changePassword') }}
      </h1>
      <p class="mb-6" style="color: var(--color-text-muted)">
        {{ $t('auth.changePasswordHint') }}
      </p>

      <!-- 오류 배너 -->
      <div v-if="errorMsg"
           class="mb-4 flex items-center gap-2 rounded-lg border px-4 py-3"
           style="background: var(--color-danger-bg);
                  border-color: var(--color-danger-border);
                  color: var(--color-danger)"
           role="alert">
        <IconAlertCircle :size="20" class="shrink-0" />
        <span>{{ errorMsg }}</span>
      </div>

      <form @submit.prevent="onSubmit" novalidate class="flex flex-col gap-5">

        <!-- 현재 비밀번호 (비밀번호가 설정된 경우만) -->
        <div v-if="!isFirstPasswordSet" class="flex flex-col gap-2">
          <label class="font-medium" style="color: var(--color-text-primary)">
            {{ $t('auth.currentPassword') }}
          </label>
          <div class="relative">
            <input
              v-model="form.currentPassword"
              :type="showCurrent ? 'text' : 'password'"
              autocomplete="current-password"
              :placeholder="$t('auth.currentPassword')"
              :disabled="isSubmitting"
              class="h-12 w-full rounded-lg border px-4 pr-12 outline-none transition-colors"
              style="background: var(--color-bg-primary);
                     border-color: var(--color-border);
                     color: var(--color-text-primary)"
            />
            <button type="button"
                    class="absolute right-3 top-1/2 -translate-y-1/2 flex items-center"
                    style="color: var(--color-text-muted)"
                    @click="showCurrent = !showCurrent"
                    :aria-label="$t('auth.togglePassword')">
              <IconEye v-if="!showCurrent" :size="20" />
              <IconEyeOff v-else :size="20" />
            </button>
          </div>
        </div>

        <!-- 새 비밀번호 -->
        <div class="flex flex-col gap-2">
          <label class="font-medium" style="color: var(--color-text-primary)">
            {{ $t('auth.newPassword') }}
          </label>
          <div class="relative">
            <input
              v-model="form.newPassword"
              :type="showNew ? 'text' : 'password'"
              autocomplete="new-password"
              :placeholder="$t('auth.newPasswordPlaceholder')"
              :disabled="isSubmitting"
              class="h-12 w-full rounded-lg border px-4 pr-12 outline-none transition-colors"
              style="background: var(--color-bg-primary);
                     border-color: var(--color-border);
                     color: var(--color-text-primary)"
            />
            <button type="button"
                    class="absolute right-3 top-1/2 -translate-y-1/2 flex items-center"
                    style="color: var(--color-text-muted)"
                    @click="showNew = !showNew"
                    :aria-label="$t('auth.togglePassword')">
              <IconEye v-if="!showNew" :size="20" />
              <IconEyeOff v-else :size="20" />
            </button>
          </div>
        </div>

        <!-- 새 비밀번호 확인 -->
        <div class="flex flex-col gap-2">
          <label class="font-medium" style="color: var(--color-text-primary)">
            {{ $t('auth.confirmNewPassword') }}
          </label>
          <div class="relative">
            <input
              v-model="form.confirmPassword"
              :type="showConfirm ? 'text' : 'password'"
              autocomplete="new-password"
              :placeholder="$t('auth.confirmNewPasswordPlaceholder')"
              :disabled="isSubmitting"
              class="h-12 w-full rounded-lg border px-4 pr-12 outline-none transition-colors"
              style="background: var(--color-bg-primary);
                     border-color: var(--color-border);
                     color: var(--color-text-primary)"
            />
            <button type="button"
                    class="absolute right-3 top-1/2 -translate-y-1/2 flex items-center"
                    style="color: var(--color-text-muted)"
                    @click="showConfirm = !showConfirm"
                    :aria-label="$t('auth.togglePassword')">
              <IconEye v-if="!showConfirm" :size="20" />
              <IconEyeOff v-else :size="20" />
            </button>
          </div>
        </div>

        <button
          type="submit"
          :disabled="isSubmitting"
          class="h-12 w-full flex items-center justify-center gap-2 rounded-lg font-semibold transition-opacity"
          style="background: var(--color-accent); color: var(--color-accent-text)"
          :class="isSubmitting ? 'opacity-60 cursor-not-allowed' : 'hover:opacity-90'"
        >
          <IconLoader2 v-if="isSubmitting" :size="20" class="spin" />
          {{ isSubmitting ? $t('auth.changingPassword') : $t('auth.changePasswordSubmit') }}
        </button>

      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { IconEye, IconEyeOff, IconLoader2, IconAlertCircle } from '@tabler/icons-vue'
import { useAuthStore } from '@/stores/auth'

const { t } = useI18n()
const router = useRouter()
const auth = useAuthStore()

const form = ref({ currentPassword: '', newPassword: '', confirmPassword: '' })
const isSubmitting = ref(false)
const errorMsg = ref<string | null>(null)
const showCurrent = ref(false)
const showNew = ref(false)
const showConfirm = ref(false)

// password_hash가 비어있는 최초 설정 상태 여부 (서버에서 판단)
const isFirstPasswordSet = computed(() => auth.student?.password_reset_required ?? false)

onMounted(async () => {
  // 학생 세션 없으면 로그인으로
  if (!auth.student) {
    try {
      await auth.fetchStudentMe()
    } catch {
      router.push('/login')
      return
    }
  }
  if (!auth.student?.password_reset_required) {
    router.push('/student')
  }
})

async function onSubmit() {
  if (isSubmitting.value) return                    // Layer 1: 중복 실행 방지
  errorMsg.value = null

  if (form.value.newPassword !== form.value.confirmPassword) {
    errorMsg.value = t('setup.passwordMismatch')
    return
  }

  isSubmitting.value = true                         // Layer 2: 즉시 잠금
  try {
    const current = isFirstPasswordSet.value ? null : form.value.currentPassword
    await auth.changePasswordStudent(current, form.value.newPassword)
    router.push('/student')
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    errorMsg.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isSubmitting.value = false                      // Layer 4: 항상 해제
  }
}
</script>
