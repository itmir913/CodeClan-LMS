<template>
  <div class="min-h-screen flex flex-col" style="background: var(--color-bg-primary)">

    <!-- 상단 네비게이션 -->
    <header class="sticky top-0 z-10 h-16 border-b"
            style="background: var(--color-bg-secondary); border-color: var(--color-border)">
      <div class="max-w-4xl mx-auto h-full flex items-center justify-between px-4">

        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-lg flex items-center justify-center font-bold text-lg text-white"
               style="background: var(--color-accent)">C</div>
          <span class="font-semibold" style="color: var(--color-text-primary)">CodeClan LMS</span>
        </div>

        <div class="flex items-center gap-2">
          <button
            class="w-9 h-9 p-0 rounded-lg flex items-center justify-center cursor-pointer"
            style="border: 1px solid var(--color-border); color: var(--color-text-muted); background: transparent"
            @click="toggleTheme"
            :aria-label="$t('auth.toggleTheme')"
          >
            <IconMoon v-if="!isDark" :size="19" />
            <IconSun v-else :size="19" />
          </button>
          <LanguageSelector />
        </div>

      </div>
    </header>

    <!-- 메인 -->
    <main class="flex-1 flex items-center justify-center px-4 py-12">
      <div class="w-full max-w-md rounded-xl border p-8"
           style="background: var(--color-bg-secondary); border-color: var(--color-border); box-shadow: var(--shadow-card)">

        <!-- 학교 이름 -->
        <p class="text-center" style="color: var(--color-text-muted)">{{ schoolName }}</p>

        <!-- 탭 -->
        <div class="grid grid-cols-2 mt-6 border-b" style="border-color: var(--color-border)">
          <button
            v-for="tab in (['teacher', 'student'] as const)"
            :key="tab"
            class="w-full pb-3 px-0 -mb-px border-0 border-b-2 bg-transparent font-medium cursor-pointer transition-colors"
            :style="{
              borderBottomColor: activeTab === tab ? 'var(--color-accent)' : 'transparent',
              color: activeTab === tab ? 'var(--color-accent)' : 'var(--color-text-muted)',
              fontWeight: activeTab === tab ? '700' : '500',
            }"
            @click="switchTab(tab)"
          >
            {{ tab === 'teacher' ? $t('auth.teacher') : $t('auth.student') }}
          </button>
        </div>

        <!-- 폼 -->
        <form class="mt-6 flex flex-col gap-5" @submit.prevent="onSubmit" novalidate>

          <!-- 아이디/학번 -->
          <div class="flex flex-col gap-2">
            <label class="font-medium" style="color: var(--color-text-primary)">
              {{ activeTab === 'teacher' ? $t('auth.username') : $t('auth.studentId') }}
            </label>
            <input
              v-model="form.username"
              type="text"
              autocomplete="username"
              :placeholder="activeTab === 'teacher' ? $t('auth.username') : $t('auth.studentId')"
              :disabled="isSubmitting"
              class="h-12 w-full px-4 rounded-lg border outline-none transition-colors"
              style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)"
            />
          </div>

          <!-- 비밀번호 -->
          <div class="flex flex-col gap-2">
            <label class="font-medium" style="color: var(--color-text-primary)">
              {{ $t('auth.password') }}
            </label>
            <div class="relative">
              <input
                v-model="form.password"
                :type="showPassword ? 'text' : 'password'"
                autocomplete="current-password"
                :placeholder="$t('auth.password')"
                :disabled="isSubmitting"
                class="h-12 w-full pl-4 pr-12 rounded-lg border outline-none transition-colors"
                style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)"
              />
              <button
                type="button"
                class="absolute right-3 top-1/2 -translate-y-1/2 p-0.5 border-0 bg-transparent flex items-center justify-center cursor-pointer"
                style="color: var(--color-text-muted)"
                @click="showPassword = !showPassword"
                :aria-label="$t('auth.togglePassword')"
              >
                <IconEye v-if="!showPassword" :size="20" />
                <IconEyeOff v-else :size="20" />
              </button>
            </div>
          </div>

          <!-- 제출 버튼 -->
          <button
            type="submit"
            :disabled="isSubmitting"
            class="login-submit h-12 w-full flex items-center justify-center gap-2 rounded-lg font-semibold cursor-pointer transition-opacity"
            :class="isSubmitting ? 'opacity-60 cursor-not-allowed' : ''"
            style="background: var(--color-accent); color: var(--color-accent-text); border: 1px solid var(--color-accent)"
          >
            <IconLoader2 v-if="isSubmitting" :size="20" class="spin" />
            {{ isSubmitting ? $t('auth.signingIn') : $t('auth.signIn') }}
          </button>

          <!-- 오류 배너 -->
          <div
            v-if="errorMsg"
            class="flex items-center gap-2 rounded-lg border px-4 py-3"
            style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
            role="alert"
          >
            <IconAlertCircle :size="20" class="shrink-0" />
            <span>{{ errorMsg }}</span>
          </div>

        </form>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { IconMoon, IconSun, IconEye, IconEyeOff, IconLoader2, IconAlertCircle } from '@tabler/icons-vue'
import { useAuthStore } from '@/stores/auth'
import LanguageSelector from '@/components/LanguageSelector.vue'

const { t } = useI18n()
const router = useRouter()
const auth = useAuthStore()

const activeTab = ref<'teacher' | 'student'>('student')
const showPassword = ref(false)
const isDark = ref(document.documentElement.getAttribute('data-theme') === 'dark')
const schoolName = ref('')
const form = ref({ username: '', password: '' })
const isSubmitting = ref(false)
const errorMsg = ref<string | null>(null)

function toggleTheme() {
  isDark.value = !isDark.value
  document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
}

function switchTab(tab: 'teacher' | 'student') {
  activeTab.value = tab
  form.value = { username: '', password: '' }
  errorMsg.value = null
  showPassword.value = false
}

async function onSubmit() {
  if (isSubmitting.value) return
  errorMsg.value = null
  isSubmitting.value = true
  try {
    if (activeTab.value === 'teacher') {
      await auth.loginTeacher(form.value.username, form.value.password)
      router.push(auth.teacher?.role === 'admin' ? '/admin' : '/teacher')
    } else {
      await auth.loginStudent(form.value.username, form.value.password)
      router.push(auth.student?.password_reset_required ? '/student/change-password' : '/student')
    }
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    errorMsg.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isSubmitting.value = false
  }
}

onMounted(async () => {
  await auth.fetchSchoolName()
  schoolName.value = auth.schoolName
})
</script>

<style scoped>
/* hover 시 base layer background 변경을 막고 opacity만 적용 */
.login-submit:hover:not(:disabled) {
  opacity: 0.9;
  background: var(--color-accent);
}
</style>
