<template>
  <div class="min-h-screen flex items-center justify-center p-6"
       style="background: var(--color-bg-primary)">

    <!-- 우상단 언어 선택 -->
    <div class="fixed top-4 right-4">
      <LanguageSelector />
    </div>

    <div class="w-full max-w-md rounded-xl p-8"
         style="background: var(--color-bg-secondary); box-shadow: var(--shadow-card)">

      <!-- 헤더 -->
      <div class="flex flex-col items-center mb-8">
        <div class="w-12 h-12 rounded-full flex items-center justify-center mb-4"
             style="background: var(--color-info-bg)">
          <IconSchool :size="28" style="color: var(--color-accent)" />
        </div>
        <h1 class="text-xl font-semibold mb-1"
            style="color: var(--color-text-primary)">
          {{ $t('setup.title') }}
        </h1>
        <p style="color: var(--color-text-muted)">
          {{ $t('setup.subtitle') }}
        </p>
      </div>

      <!-- 오류 배너 -->
      <div v-if="errorMsg" class="mb-6 rounded-lg px-4 py-3 flex items-start gap-3"
           style="background: var(--color-danger-bg); border: 1px solid var(--color-danger-border)">
        <IconAlertCircle :size="20" class="mt-0.5 shrink-0" style="color: var(--color-danger)" />
        <span style="color: var(--color-danger)">{{ errorMsg }}</span>
      </div>

      <form @submit.prevent="onSubmit" novalidate>

        <!-- 학교 이름 -->
        <div class="mb-5">
          <label class="block font-medium mb-1.5"
                 style="color: var(--color-text-primary)">
            {{ $t('setup.schoolName') }}
          </label>
          <input
            v-model="form.schoolName"
            type="text"
            :placeholder="$t('setup.schoolNamePlaceholder')"
            autocomplete="off"
            :disabled="submitting"
          />
          <p class="mt-1" style="color: var(--color-text-muted); font-size: 0.875rem">
            {{ $t('setup.schoolNameHint') }}
          </p>
        </div>

        <!-- 구분선 -->
        <div class="flex items-center gap-3 my-6">
          <div class="flex-1 h-px" style="background: var(--color-border)" />
          <span class="flex items-center gap-1.5 font-medium"
                style="color: var(--color-text-muted)">
            <IconUser :size="16" />
            {{ $t('setup.adminSection') }}
          </span>
          <div class="flex-1 h-px" style="background: var(--color-border)" />
        </div>

        <!-- 관리자 이름 -->
        <div class="mb-4">
          <label class="block font-medium mb-1.5"
                 style="color: var(--color-text-primary)">
            {{ $t('setup.adminName') }}
          </label>
          <input
            v-model="form.adminName"
            type="text"
            :placeholder="$t('setup.adminNamePlaceholder')"
            autocomplete="off"
            :disabled="submitting"
          />
        </div>

        <!-- 관리자 아이디 -->
        <div class="mb-4">
          <label class="block font-medium mb-1.5"
                 style="color: var(--color-text-primary)">
            {{ $t('setup.adminUsername') }}
          </label>
          <input
            v-model="form.adminUsername"
            type="text"
            :placeholder="$t('setup.adminUsernamePlaceholder')"
            autocomplete="off"
            :disabled="submitting"
          />
        </div>

        <!-- 비밀번호 -->
        <div class="mb-4">
          <label class="block font-medium mb-1.5"
                 style="color: var(--color-text-primary)">
            {{ $t('setup.adminPassword') }}
          </label>
          <input
            v-model="form.adminPassword"
            type="password"
            :placeholder="$t('setup.adminPasswordPlaceholder')"
            autocomplete="new-password"
            :disabled="submitting"
          />
        </div>

        <!-- 비밀번호 확인 -->
        <div class="mb-8">
          <label class="block font-medium mb-1.5"
                 style="color: var(--color-text-primary)">
            {{ $t('setup.adminPasswordConfirm') }}
          </label>
          <input
            v-model="form.adminPasswordConfirm"
            type="password"
            :placeholder="$t('setup.adminPasswordConfirmPlaceholder')"
            autocomplete="new-password"
            :disabled="submitting"
          />
        </div>

        <!-- 제출 버튼 -->
        <button
          type="submit"
          class="w-full font-semibold"
          style="background: var(--color-accent); color: var(--color-accent-text); border-color: var(--color-accent);"
          :disabled="submitting"
        >
          <IconLoader2 v-if="submitting" :size="18" class="spin mr-2" />
          {{ submitting ? $t('setup.submitting') : $t('setup.submit') }}
        </button>

      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { IconSchool, IconUser, IconAlertCircle, IconLoader2 } from '@tabler/icons-vue'
import { useAuthStore } from '@/stores/auth'
import LanguageSelector from '@/components/LanguageSelector.vue'

const { t } = useI18n()
const router = useRouter()
const auth = useAuthStore()

const form = ref({
  schoolName: '',
  adminName: '',
  adminUsername: '',
  adminPassword: '',
  adminPasswordConfirm: '',
})

const submitting = ref(false)
const errorMsg = ref<string | null>(null)

async function onSubmit() {
  errorMsg.value = null

  if (form.value.adminPassword !== form.value.adminPasswordConfirm) {
    errorMsg.value = t('setup.passwordMismatch')
    return
  }

  submitting.value = true
  try {
    await auth.completeSetup({
      school_name: form.value.schoolName,
      admin_name: form.value.adminName,
      admin_username: form.value.adminUsername,
      admin_password: form.value.adminPassword,
    })
    router.push('/login')
  } catch (e) {
    errorMsg.value = e instanceof Error ? e.message : t('common.error')
  } finally {
    submitting.value = false
  }
}
</script>
