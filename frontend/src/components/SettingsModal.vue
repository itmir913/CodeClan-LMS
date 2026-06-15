<template>
  <Teleport to="body">
    <div
      v-if="modelValue"
      class="fixed inset-0 z-50 flex items-center justify-center p-4"
      style="background: var(--color-modal-overlay)"
      role="dialog"
      aria-modal="true"
    >
      <div
        class="w-full max-w-md rounded-2xl border flex flex-col"
        style="background: var(--color-bg-secondary); border-color: var(--color-border); box-shadow: var(--shadow-dropdown)"
        @keydown.esc="$emit('update:modelValue', false)"
        tabindex="-1"
        ref="panelRef"
      >
        <!-- Header -->
        <div class="flex items-center justify-between px-6 pt-5 pb-4 border-b"
             style="border-color: var(--color-border)">
          <h2 class="font-semibold" style="color: var(--color-text-primary)">
            {{ $t('auth.settingsTitle') }}
          </h2>
          <button
            class="w-8 h-8 p-0 rounded-lg flex items-center justify-center"
            style="border: 1px solid var(--color-border); color: var(--color-text-muted); background: transparent"
            @click="$emit('update:modelValue', false)"
            :aria-label="$t('common.cancel')"
          >
            <IconX :size="16" />
          </button>
        </div>

        <!-- Body -->
        <div class="px-6 py-5 flex flex-col gap-6 overflow-y-auto" style="max-height: 70vh">

          <!-- ── 이름 변경 (교사/admin 전용) ── -->
          <section v-if="isTeacher">
            <h3 class="font-medium mb-3" style="color: var(--color-text-muted)">
              {{ $t('auth.profileSection') }}
            </h3>

            <div v-if="nameError" class="rounded-lg border px-4 py-3 mb-3"
                 style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
                 role="alert">
              {{ nameError }}
            </div>
            <div v-if="nameSuccess" class="rounded-lg border px-4 py-3 mb-3"
                 style="background: var(--color-success-bg); border-color: var(--color-success); color: var(--color-success)"
                 role="status">
              {{ nameSuccess }}
            </div>

            <label class="block mb-1 font-medium" style="color: var(--color-text-primary)">
              {{ $t('auth.displayName') }}
            </label>
            <div class="flex items-center gap-2">
              <input
                v-model="nameInput"
                type="text"
                class="flex-1 rounded-lg"
                :placeholder="$t('auth.displayNamePlaceholder')"
                :disabled="isSavingName"
                @keydown.enter="onSaveName"
              />
              <button
                class="h-10 px-4 rounded-lg font-medium flex items-center gap-2 shrink-0"
                style="background: var(--color-accent); color: var(--color-accent-text); border: none"
                :disabled="isSavingName"
                @click="onSaveName"
              >
                <IconLoader2 v-if="isSavingName" :size="15" class="spin" />
                {{ isSavingName ? $t('common.saving') : $t('common.save') }}
              </button>
            </div>

            <hr class="mt-6" style="border-color: var(--color-border)" />

          </section>

          <!-- ── 비밀번호 변경 (공통) ── -->
          <section>
            <h3 class="font-medium mb-3" style="color: var(--color-text-muted)">
              {{ $t('auth.passwordSection') }}
            </h3>

            <div v-if="pwError" class="rounded-lg border px-4 py-3 mb-3"
                 style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
                 role="alert">
              {{ pwError }}
            </div>
            <div v-if="pwSuccess" class="rounded-lg border px-4 py-3 mb-3"
                 style="background: var(--color-success-bg); border-color: var(--color-success); color: var(--color-success)"
                 role="status">
              {{ pwSuccess }}
            </div>

            <div class="flex flex-col gap-3">
              <!-- 학생 최초 비밀번호 설정(현재 pw 없음)인 경우 숨김 -->
              <div v-if="isTeacher || !isFirstLogin">
                <label class="block mb-1 font-medium" style="color: var(--color-text-primary)">
                  {{ $t('auth.currentPassword') }}
                </label>
                <div class="relative">
                  <input
                    v-model="pwCurrent"
                    :type="showPwCurrent ? 'text' : 'password'"
                    class="w-full pr-12 rounded-lg password-input"
                    :disabled="isSavingPw"
                  />
                  <button
                    type="button"
                    class="absolute right-3 top-1/2 -translate-y-1/2 p-0.5 border-0 bg-transparent flex items-center justify-center cursor-pointer"
                    style="color: var(--color-text-muted)"
                    @click="showPwCurrent = !showPwCurrent"
                    :aria-label="$t('auth.togglePassword')"
                  >
                    <IconEye v-if="!showPwCurrent" :size="20" />
                    <IconEyeOff v-else :size="20" />
                  </button>
                </div>
              </div>
              <div>
                <label class="block mb-1 font-medium" style="color: var(--color-text-primary)">
                  {{ $t('auth.newPassword') }}
                </label>
                <div class="relative">
                  <input
                    v-model="pwNew"
                    :type="showPwNew ? 'text' : 'password'"
                    class="w-full pr-12 rounded-lg password-input"
                    :placeholder="$t('auth.newPasswordPlaceholder')"
                    :disabled="isSavingPw"
                  />
                  <button
                    type="button"
                    class="absolute right-3 top-1/2 -translate-y-1/2 p-0.5 border-0 bg-transparent flex items-center justify-center cursor-pointer"
                    style="color: var(--color-text-muted)"
                    @click="showPwNew = !showPwNew"
                    :aria-label="$t('auth.togglePassword')"
                  >
                    <IconEye v-if="!showPwNew" :size="20" />
                    <IconEyeOff v-else :size="20" />
                  </button>
                </div>
              </div>
              <div>
                <label class="block mb-1 font-medium" style="color: var(--color-text-primary)">
                  {{ $t('auth.confirmNewPassword') }}
                </label>
                <div class="relative">
                  <input
                    v-model="pwConfirm"
                    :type="showPwConfirm ? 'text' : 'password'"
                    class="w-full pr-12 rounded-lg password-input"
                    :placeholder="$t('auth.confirmNewPasswordPlaceholder')"
                    :disabled="isSavingPw"
                    @keydown.enter="onSavePw"
                  />
                  <button
                    type="button"
                    class="absolute right-3 top-1/2 -translate-y-1/2 p-0.5 border-0 bg-transparent flex items-center justify-center cursor-pointer"
                    style="color: var(--color-text-muted)"
                    @click="showPwConfirm = !showPwConfirm"
                    :aria-label="$t('auth.togglePassword')"
                  >
                    <IconEye v-if="!showPwConfirm" :size="20" />
                    <IconEyeOff v-else :size="20" />
                  </button>
                </div>
              </div>
              <button
                class="h-10 px-4 rounded-lg font-medium flex items-center justify-center gap-2 w-full"
                style="background: var(--color-accent); color: var(--color-accent-text); border: none"
                :disabled="isSavingPw"
                @click="onSavePw"
              >
                <IconLoader2 v-if="isSavingPw" :size="15" class="spin" />
                {{ isSavingPw ? $t('common.saving') : $t('auth.changePasswordSubmit') }}
              </button>
            </div>
          </section>

        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, computed, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { IconX, IconLoader2, IconEye, IconEyeOff } from '@tabler/icons-vue'
import { useAuthStore } from '@/stores/auth'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ (e: 'update:modelValue', val: boolean): void }>()

const { t } = useI18n()
const auth = useAuthStore()
const panelRef = ref<HTMLElement | null>(null)

const isTeacher = computed(() => auth.teacher !== null)
const isFirstLogin = computed(
  () => auth.student !== null && auth.student.password_reset_required,
)

// ── 이름 변경 ────────────────────────────────────────────────────────
const nameInput = ref('')
const nameError = ref<string | null>(null)
const nameSuccess = ref<string | null>(null)
const isSavingName = ref(false)

async function onSaveName() {
  if (isSavingName.value) return
  nameError.value = null
  nameSuccess.value = null
  const trimmed = nameInput.value.trim()
  if (!trimmed) {
    nameError.value = t('errors.ERR_NAME_REQUIRED')
    return
  }
  isSavingName.value = true
  try {
    await auth.updateTeacherName(trimmed)
    nameSuccess.value = t('auth.displayName') + ' ' + t('common.save') + ' ✓'
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    nameError.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isSavingName.value = false
  }
}

// ── 비밀번호 변경 ─────────────────────────────────────────────────────
const pwCurrent = ref('')
const pwNew = ref('')
const pwConfirm = ref('')
const pwError = ref<string | null>(null)
const pwSuccess = ref<string | null>(null)
const isSavingPw = ref(false)
const showPwCurrent = ref(false)
const showPwNew = ref(false)
const showPwConfirm = ref(false)

async function onSavePw() {
  if (isSavingPw.value) return
  pwError.value = null
  pwSuccess.value = null
  if (pwNew.value !== pwConfirm.value) {
    pwError.value = t('errors.ERR_PASSWORD_MISMATCH')
    return
  }
  isSavingPw.value = true
  try {
    if (isTeacher.value) {
      await auth.changePasswordTeacher(pwCurrent.value, pwNew.value)
    } else {
      await auth.changePasswordStudent(
        isFirstLogin.value ? null : pwCurrent.value,
        pwNew.value,
      )
    }
    pwSuccess.value = t('auth.changePasswordSubmit') + ' ✓'
    pwCurrent.value = ''
    pwNew.value = ''
    pwConfirm.value = ''
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    pwError.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isSavingPw.value = false
  }
}

// ── 모달 열릴 때 초기화 ────────────────────────────────────────────────
watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      nameInput.value = auth.teacher?.name ?? ''
      nameError.value = null
      nameSuccess.value = null
      pwCurrent.value = ''
      pwNew.value = ''
      pwConfirm.value = ''
      pwError.value = null
      pwSuccess.value = null
      showPwCurrent.value = false
      showPwNew.value = false
      showPwConfirm.value = false
      nextTick(() => panelRef.value?.focus())
    }
  },
)
</script>

<style scoped>
.password-input::-ms-reveal,
.password-input::-ms-clear {
  display: none;
}
.password-input::-webkit-contacts-auto-fill-button,
.password-input::-webkit-credentials-auto-fill-button {
  visibility: hidden;
}
</style>
