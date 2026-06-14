<template>
  <div class="min-h-screen" style="background: var(--color-bg-primary)">

    <!-- Top Nav -->
    <header class="sticky top-0 z-30 h-16 border-b"
            style="background: var(--color-bg-secondary); border-color: var(--color-border)">
      <div class="h-full max-w-4xl mx-auto flex items-center justify-between px-4 sm:px-6">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg flex items-center justify-center font-bold text-white shrink-0"
               style="background: var(--color-accent)">C</div>
          <span class="font-semibold" style="color: var(--color-text-primary)">CodeClan LMS</span>
        </div>
        <div class="flex items-center gap-2">
          <LanguageSelector />
          <button
            class="h-9 px-3 rounded-lg font-medium"
            style="border: 1px solid var(--color-border); color: var(--color-text-muted); background: transparent"
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
              {{ auth.student?.grade }}학년 {{ auth.student?.class_no }}반 {{ auth.student?.number }}번
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { IconClipboardOff } from '@tabler/icons-vue'
import { useAuthStore } from '@/stores/auth'
import LanguageSelector from '@/components/LanguageSelector.vue'

const router = useRouter()
const auth = useAuthStore()
const isLoggingOut = ref(false)

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
