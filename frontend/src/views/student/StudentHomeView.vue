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
    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 pb-16">

      <!-- Student info card -->
      <div class="rounded-xl border p-5 mb-6 flex items-center gap-4"
           style="background: var(--color-bg-secondary); border-color: var(--color-border); box-shadow: var(--shadow-card)">
        <div class="w-12 h-12 rounded-full flex items-center justify-center font-bold text-xl shrink-0"
             style="background: var(--color-accent); color: var(--color-accent-text)">
          {{ studentInitial }}
        </div>
        <div>
          <p class="font-bold" style="color: var(--color-text-primary)">
            {{ auth.student?.name }}
          </p>
          <p style="color: var(--color-text-muted)">
            {{ $t('student.gradeClassInfo', { grade: auth.student?.grade, class_no: auth.student?.class_no, number: auth.student?.number }) }}
          </p>
        </div>
      </div>

      <!-- Section header -->
      <div class="flex items-center justify-between mb-5">
        <h2 class="font-semibold tracking-widest uppercase"
            style="color: var(--color-text-muted)">
          {{ $t('student.myClasses') }}
          <span v-if="!loading">({{ classes.length }})</span>
        </h2>
      </div>

      <!-- Loading -->
      <div v-if="loading"
           class="flex items-center justify-center py-24 gap-3"
           style="color: var(--color-text-muted)">
        <IconLoader2 :size="22" class="spin" />
        <span>{{ $t('common.loading') }}</span>
      </div>

      <!-- Error -->
      <div v-else-if="error"
           class="flex items-center gap-3 rounded-xl border px-5 py-4"
           style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
           role="alert">
        <IconAlertCircle :size="20" class="shrink-0" />
        <span>{{ $t(`errors.${error}`, $t('errors.ERR_UNKNOWN')) }}</span>
        <button
          class="ml-auto h-8 px-3 rounded-lg font-medium border bg-transparent"
          style="border-color: var(--color-danger-border); color: var(--color-danger)"
          @click="fetchClasses"
        >{{ $t('common.retry') }}</button>
      </div>

      <!-- Empty state -->
      <div v-else-if="classes.length === 0"
           class="flex flex-col items-center justify-center py-24 gap-3">
        <IconSchool :size="48" style="color: var(--color-text-tertiary)" />
        <p class="font-medium" style="color: var(--color-text-muted)">{{ $t('student.noClasses') }}</p>
        <p style="color: var(--color-text-tertiary)">{{ $t('student.noClassesHint') }}</p>
      </div>

      <!-- Card grid -->
      <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        <article
          v-for="cls in classes"
          :key="cls.id"
          class="class-card rounded-xl border flex flex-col overflow-hidden cursor-pointer"
          style="background: var(--color-bg-secondary); border-color: var(--color-border); box-shadow: var(--shadow-card)"
          @click="router.push(`/student/classes/${cls.id}`)"
        >
          <span class="h-1 block shrink-0" :style="{ background: cardAccentColor(cls.id) }"></span>

          <div class="flex flex-col flex-1 p-5">
            <h3 class="font-bold" style="color: var(--color-text-primary)">{{ cls.name }}</h3>
            <p class="mt-1" style="color: var(--color-text-muted)">{{ cls.subject_name }}</p>

            <div class="mt-4 pt-3 border-t flex items-center gap-2"
                 style="border-color: var(--color-border)">
              <template v-if="cls.has_active_session">
                <span class="inline-flex items-center gap-2 rounded-full px-3 py-1 font-semibold session-live-badge">
                  <span class="w-2 h-2 rounded-full shrink-0" style="background: currentColor"></span>
                  {{ $t('classes.sessionLive') }}
                </span>
              </template>
              <template v-else>
                <IconUser :size="16" style="color: var(--color-text-tertiary)" />
                <span style="color: var(--color-text-muted)">{{ cls.teacher_name }}</span>
              </template>
            </div>
          </div>
        </article>
      </div>

    </main>

    <SettingsModal v-model="showSettings" />

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  IconSettings, IconLoader2, IconAlertCircle, IconSchool, IconUser,
} from '@tabler/icons-vue'
import { useAuthStore } from '@/stores/auth'
import { api } from '@/api/client'
import type { ClassItem } from '@/api/client'
import LanguageSelector from '@/components/LanguageSelector.vue'
import SettingsModal from '@/components/SettingsModal.vue'

const router = useRouter()
const auth = useAuthStore()

const isLoggingOut = ref(false)
const showSettings = ref(false)
const loading = ref(false)
const error = ref<string | null>(null)
const classes = ref<ClassItem[]>([])

const studentInitial = computed(() =>
  auth.student?.name ? auth.student.name.charAt(0) : '?'
)

const ACCENT_VARS = [
  'var(--color-card-accent-0)',
  'var(--color-card-accent-1)',
  'var(--color-card-accent-2)',
  'var(--color-card-accent-3)',
  'var(--color-card-accent-4)',
  'var(--color-card-accent-5)',
]

function cardAccentColor(id: number) {
  return ACCENT_VARS[id % ACCENT_VARS.length]
}

async function fetchClasses() {
  loading.value = true
  error.value = null
  try {
    classes.value = await api.student.myClasses()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'ERR_UNKNOWN'
  } finally {
    loading.value = false
  }
}

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
    try { await auth.fetchStudentMe() } catch { router.push('/login'); return }
  }
  await fetchClasses()
})
</script>

<style scoped>
.class-card {
  transition: transform 0.15s ease, box-shadow 0.15s ease, border-color 0.15s ease;
}
.class-card:hover {
  box-shadow: 0 14px 28px -16px rgba(15, 23, 42, 0.35);
  border-color: var(--color-border-strong);
}
[data-theme="dark"] .class-card:hover {
  box-shadow: 0 14px 28px -16px rgba(2, 6, 23, 0.6);
}
.session-live-badge {
  color: var(--color-danger);
  background: var(--color-danger-bg);
  border: 1px solid var(--color-danger-border);
}
</style>
