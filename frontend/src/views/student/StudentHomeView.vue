<template>
  <div class="min-h-screen" style="background: var(--color-bg-primary)">

    <!-- Top Nav -->
    <header class="sticky top-0 z-30 h-16 border-b"
            style="background: var(--color-bg-secondary); border-color: var(--color-border)">
      <div class="h-full max-w-full flex items-center justify-between px-6">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg flex items-center justify-center font-bold text-white shrink-0"
               style="background: var(--color-accent)">C</div>
          <span class="font-semibold" style="color: var(--color-text-primary)">CodeClan LMS</span>
        </div>
        <div class="flex items-center gap-2">
          <button
            class="w-9 h-9 p-0 rounded-lg flex items-center justify-center border bg-transparent"
            style="border-color: var(--color-border); color: var(--color-text-muted);"
            @click="showSettings = true"
            :aria-label="$t('common.settings')"
          >
            <IconSettings :size="18" />
          </button>
          <LanguageSelector />
          <button
            class="h-9 px-3 rounded-lg font-medium border bg-transparent"
            style="border-color: var(--color-border); color: var(--color-text-muted);"
            @click="onLogout"
            :disabled="isLoggingOut"
          >
            {{ $t('common.logout') }}
          </button>
        </div>
      </div>
    </header>

    <!-- Main -->
    <main class="max-w-4xl mx-auto px-4 sm:px-6 py-8">

      <!-- Student info card -->
      <div class="rounded-xl border p-6 mb-6"
           style="background: var(--color-bg-secondary); border-color: var(--color-border); box-shadow: var(--shadow-card)">
        <div class="flex items-center gap-4">
          <div class="w-14 h-14 rounded-full flex items-center justify-center font-bold text-xl"
               style="background: var(--color-accent); color: var(--color-accent-text)">
            {{ studentInitial }}
          </div>
          <div>
            <h1 class="font-bold text-xl" style="color: var(--color-text-primary)">
              {{ auth.student?.name }}
            </h1>
            <p style="color: var(--color-text-muted)">
              {{ $t('student.gradeClassInfo', { grade: auth.student?.grade, class_no: auth.student?.class_no, number: auth.student?.number }) }}
            </p>
          </div>
        </div>
      </div>

      <!-- Session area (stub — 단계 9에서 구현) -->
      <div class="rounded-xl border p-8 flex flex-col items-center justify-center gap-3"
           style="background: var(--color-bg-secondary); border-color: var(--color-border)">
        <IconClipboardOff :size="40" style="color: var(--color-text-tertiary)" />
        <p class="font-medium" style="color: var(--color-text-muted)">
          {{ $t('student.noActiveSession') }}
        </p>
      </div>

    </main>

    <SettingsModal v-model="showSettings" />

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { IconClipboardOff, IconSettings } from '@tabler/icons-vue'
import { useAuthStore } from '@/stores/auth'
import LanguageSelector from '@/components/LanguageSelector.vue'
import SettingsModal from '@/components/SettingsModal.vue'

const router = useRouter()
const auth = useAuthStore()
const isLoggingOut = ref(false)
const showSettings = ref(false)

const studentInitial = computed(() =>
  auth.student?.name ? auth.student.name.charAt(0) : '?'
)

async function onLogout() {
  if (isLoggingOut.value) return
  isLoggingOut.value = true
  try {
    await auth.logoutStudent()
    router.push('/login')
  } finally {
    isLoggingOut.value = false
  }
}

onMounted(async () => {
  if (!auth.student) {
    try { await auth.fetchStudentMe() } catch { router.push('/login') }
  }
})
</script>
