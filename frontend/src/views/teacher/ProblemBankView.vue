<template>
  <div class="min-h-screen flex flex-col" style="background: var(--color-bg-secondary)">

    <!-- ── 헤더 ── -->
    <header class="h-15 flex items-center justify-between px-7 flex-shrink-0"
            style="background: var(--color-bg-primary); border-bottom: 1px solid var(--color-border)">
      <div class="flex items-center gap-4">
        <button class="flex items-center gap-2 h-9 px-3 rounded-lg border"
                style="background: transparent; color: var(--color-text-muted); border-color: var(--color-border)"
                @click="goBack">
          <IconArrowLeft :size="15" />
          <span>{{ auth.teacher?.role === 'admin' ? $t('admin.adminBadge') : $t('problems.myClasses') }}</span>
        </button>
        <div class="w-px h-5" style="background: var(--color-border)"></div>
        <div class="flex items-center gap-2">
          <IconBooks :size="20" style="color: var(--color-accent)" />
          <span class="text-xl font-bold" style="color: var(--color-text-primary)">{{ $t('problems.title') }}</span>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <button
          class="w-9 h-9 p-0 rounded-lg flex items-center justify-center"
          style="border: 1px solid var(--color-border); color: var(--color-text-muted); background: transparent"
          @click="toggleTheme"
          :aria-label="$t('auth.toggleTheme')"
        >
          <IconMoon v-if="!isDark" :size="18" />
          <IconSun v-else :size="18" />
        </button>
        <button class="flex items-center gap-2 h-9 px-4 rounded-lg font-semibold"
                style="background: var(--color-accent); color: var(--color-accent-text); border: none"
                @click="openCreate">
          <IconPlus :size="16" />
          <span>{{ $t('problems.newProblem') }}</span>
        </button>
      </div>
    </header>

    <!-- ── 툴바 ── -->
    <div class="flex items-center gap-3 px-7 py-4 flex-wrap flex-shrink-0"
         style="background: var(--color-bg-primary); border-bottom: 1px solid var(--color-border)">
      <!-- 검색 -->
      <div class="relative flex-1" style="min-width: 180px; max-width: 340px">
        <IconSearch :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none"
                    style="color: var(--color-text-tertiary)" />
        <input
          v-model="searchQuery"
          type="text"
          class="w-full h-10 rounded-xl pl-10 pr-4 border"
          style="background: var(--color-bg-secondary); color: var(--color-text-primary); border-color: var(--color-border); font-size: 1rem"
          :placeholder="$t('problems.searchPlaceholder')"
        />
      </div>
      <!-- 유형 필터 -->
      <div class="flex items-center gap-2 flex-wrap">
        <button
          v-for="f in typeFilters"
          :key="f.value"
          class="h-8 px-4 rounded-full border font-medium transition-colors"
          :style="activeFilter === f.value
            ? 'background: var(--color-text-primary); color: var(--color-bg-primary); border-color: var(--color-text-primary)'
            : 'background: var(--color-bg-primary); color: var(--color-text-muted); border-color: var(--color-border)'"
          @click="activeFilter = f.value"
        >
          {{ f.label }}
        </button>
      </div>
      <!-- 과목 필터 -->
      <select
        v-model="selectedSubject"
        class="h-10 px-3 rounded-xl border text-base"
        style="background: var(--color-bg-secondary); color: var(--color-text-primary); border-color: var(--color-border)"
      >
        <option :value="null">{{ $t('problems.subjectAll') }}</option>
        <option v-for="s in classStore.subjects" :key="s.id" :value="s.id">{{ s.name }}</option>
      </select>
      <!-- 문제 수 -->
      <span class="ml-auto font-medium" style="color: var(--color-text-tertiary); white-space: nowrap">
        {{ $t('problems.problemCount', { count: filteredProblems.length }) }}
      </span>
    </div>

    <!-- ── 문제 목록 ── -->
    <main class="flex-1 px-7 py-5">

      <!-- 로딩 -->
      <div v-if="store.loading" class="flex items-center justify-center py-16">
        <IconLoader2 :size="32" class="spin" style="color: var(--color-accent)" />
      </div>

      <!-- 에러 -->
      <div v-else-if="store.error"
           class="flex items-center gap-3 p-4 rounded-xl mb-4"
           style="background: var(--color-danger-bg); color: var(--color-danger); border: 1px solid var(--color-danger-border)"
           role="alert">
        <IconAlertCircle :size="20" class="shrink-0" />
        <span>{{ $t(`errors.${store.error}`, $t('errors.ERR_UNKNOWN')) }}</span>
      </div>

      <!-- 빈 상태 -->
      <div v-else-if="store.problems.length === 0"
           class="flex flex-col items-center justify-center py-20 gap-3">
        <IconBooks :size="48" class="opacity-30" style="color: var(--color-text-muted)" />
        <p class="font-semibold" style="color: var(--color-text-muted)">{{ $t('problems.noProblemsFull') }}</p>
        <p style="color: var(--color-text-tertiary)">{{ $t('problems.noProblemHint') }}</p>
      </div>

      <!-- 검색 결과 없음 -->
      <div v-else-if="filteredProblems.length === 0"
           class="flex flex-col items-center justify-center py-20">
        <p style="color: var(--color-text-tertiary)">{{ $t('problems.noProblemsFiltered') }}</p>
      </div>

      <!-- 목록 -->
      <div v-else class="flex flex-col gap-2">
        <div
          v-for="p in filteredProblems"
          :key="p.id"
          class="flex items-center gap-4 px-5 py-4 rounded-xl border cursor-default transition-colors problem-row"
          style="background: var(--color-bg-primary); border-color: var(--color-border)"
        >
          <!-- 유형 뱃지 -->
          <span
            class="inline-flex items-center h-7 px-3 rounded-full font-semibold flex-shrink-0"
            :style="typeBadgeStyle(p.type)"
          >
            {{ typeLabel(p.type) }}
          </span>

          <!-- 제목 -->
          <span class="flex-1 font-semibold truncate" style="color: var(--color-text-primary)">
            {{ p.title }}
          </span>

          <!-- 임시저장 뱃지 -->
          <span v-if="p.is_draft"
                class="h-6 px-3 rounded-full font-medium flex-shrink-0 inline-flex items-center"
                style="background: var(--color-warning-bg); color: var(--color-warning)">
            {{ $t('problems.draft') }}
          </span>

          <!-- 과목 -->
          <span v-if="p.subject_name"
                class="flex-shrink-0" style="color: var(--color-text-tertiary)">
            {{ p.subject_name }}
          </span>

          <!-- 날짜 -->
          <span class="flex-shrink-0 font-mono" style="color: var(--color-text-tertiary)">
            {{ p.created_at.slice(0, 10) }}
          </span>

          <!-- 액션 -->
          <div class="flex gap-2 flex-shrink-0">
            <button
              class="h-8 px-3 rounded-lg border font-medium transition-colors"
              style="background: var(--color-bg-primary); color: var(--color-text-muted); border-color: var(--color-border)"
              @click="openEdit(p.id)"
            >
              {{ $t('problems.edit') }}
            </button>
            <button
              class="h-8 px-3 rounded-lg border font-medium transition-colors"
              style="background: var(--color-bg-primary); color: var(--color-danger); border-color: var(--color-danger-border)"
              @click="confirmDelete(p)"
            >
              {{ $t('problems.delete') }}
            </button>
          </div>
        </div>
      </div>
    </main>

    <!-- ══ 삭제 확인 모달 ══ -->
    <div v-if="deleteTarget"
         class="fixed inset-0 z-50 flex items-center justify-center p-4"
         style="background: var(--color-modal-overlay)">
      <div class="w-full max-w-md rounded-2xl p-7 flex flex-col gap-5"
           style="background: var(--color-bg-primary)">
        <div class="flex items-center gap-3">
          <IconAlertTriangle :size="22" style="color: var(--color-danger); flex-shrink: 0" />
          <h2 class="text-xl font-bold" style="color: var(--color-text-primary)">
            {{ $t('problems.deleteConfirmTitle') }}
          </h2>
        </div>
        <p style="color: var(--color-text-muted)">
          {{ $t('problems.deleteConfirm', { title: deleteTarget.title }) }}
        </p>
        <p style="color: var(--color-text-tertiary)">{{ $t('problems.deleteConfirmHint') }}</p>
        <div v-if="deleteError"
             class="flex items-center gap-2 p-3 rounded-lg"
             style="background: var(--color-danger-bg); color: var(--color-danger)"
             role="alert">
          <IconAlertCircle :size="16" class="shrink-0" />
          <span>{{ $t(`errors.${deleteError}`, $t('errors.ERR_UNKNOWN')) }}</span>
        </div>
        <div class="flex justify-end gap-3">
          <button
            :disabled="isDeleting"
            class="h-10 px-5 rounded-xl border font-medium"
            style="background: var(--color-bg-primary); color: var(--color-text-muted); border-color: var(--color-border)"
            @click="deleteTarget = null; deleteError = null"
          >
            {{ $t('problems.cancel') }}
          </button>
          <button
            :disabled="isDeleting"
            class="h-10 px-5 rounded-xl font-semibold"
            style="background: var(--color-danger); color: var(--color-accent-text); border: none"
            @click="doDelete"
          >
            <IconLoader2 v-if="isDeleting" :size="15" class="spin inline-block mr-1" />
            {{ isDeleting ? $t('problems.deleting') : $t('problems.delete') }}
          </button>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  IconArrowLeft, IconPlus, IconSearch, IconLoader2,
  IconAlertCircle, IconAlertTriangle, IconBooks, IconMoon, IconSun,
} from '@tabler/icons-vue'
import { useProblemStore } from '@/stores/problem'
import { useClassStore } from '@/stores/class'
import { useAuthStore } from '@/stores/auth'
import type { ProblemListItem } from '@/api/client'

const router = useRouter()
const { t } = useI18n()
const store = useProblemStore()
const classStore = useClassStore()
const auth = useAuthStore()

const isDark = ref(document.documentElement.getAttribute('data-theme') === 'dark')

function toggleTheme() {
  isDark.value = !isDark.value
  document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
}

// ── 목록 상태 ────────────────────────────────────────────────────────────────

const searchQuery = ref('')
const activeFilter = ref('all')
const selectedSubject = ref<number | null>(null)

const typeFilters = computed(() => [
  { value: 'all', label: t('problems.all') },
  { value: 'short_answer', label: t('problems.type_short_answer') },
  { value: 'multiple_choice', label: t('problems.type_multiple_choice') },
  { value: 'code_submit', label: t('problems.type_code_submit') },
])

const filteredProblems = computed(() => {
  let list = store.problems
  if (activeFilter.value !== 'all') {
    list = list.filter((p) => p.type === activeFilter.value)
  }
  if (selectedSubject.value !== null) {
    list = list.filter((p) => p.subject_id === selectedSubject.value)
  }
  const q = searchQuery.value.trim().toLowerCase()
  if (q) {
    list = list.filter((p) => p.title.toLowerCase().includes(q))
  }
  return list
})

function typeLabel(slug: string): string {
  const map: Record<string, string> = {
    short_answer: t('problems.type_short_answer'),
    multiple_choice: t('problems.type_multiple_choice'),
    code_submit: t('problems.type_code_submit'),
  }
  return map[slug] ?? slug
}

function typeBadgeStyle(slug: string): string {
  const styles: Record<string, string> = {
    short_answer: 'background: var(--color-type-short-bg); color: var(--color-type-short)',
    multiple_choice: 'background: var(--color-type-mcq-bg); color: var(--color-type-mcq)',
    code_submit: 'background: var(--color-type-coding-bg); color: var(--color-type-coding)',
  }
  return styles[slug] ?? 'background: var(--color-bg-secondary); color: var(--color-text-muted)'
}

// ── 탐색 ─────────────────────────────────────────────────────────────────────

function openCreate() {
  router.push({ name: 'problem-new' })
}

function openEdit(id: number) {
  router.push({ name: 'problem-edit', params: { id } })
}

function goBack() {
  router.push(auth.teacher?.role === 'admin' ? '/admin' : '/teacher')
}

// ── 삭제 ─────────────────────────────────────────────────────────────────────

const deleteTarget = ref<ProblemListItem | null>(null)
const deleteError = ref<string | null>(null)
const isDeleting = ref(false)

function confirmDelete(p: ProblemListItem) {
  deleteTarget.value = p
  deleteError.value = null
}

async function doDelete() {
  if (isDeleting.value) return
  if (!deleteTarget.value) return
  isDeleting.value = true
  deleteError.value = null
  try {
    await store.deleteProblem(deleteTarget.value.id)
    deleteTarget.value = null
  } catch (e) {
    deleteError.value = e instanceof Error ? e.message : 'ERR_UNKNOWN'
  } finally {
    isDeleting.value = false
  }
}

// ── ESC 키 ───────────────────────────────────────────────────────────────────

function handleKeydown(e: KeyboardEvent) {
  if (e.key !== 'Escape') return
  if (deleteTarget.value) {
    deleteTarget.value = null
    deleteError.value = null
  }
}

// ── 초기화 ───────────────────────────────────────────────────────────────────

onMounted(async () => {
  window.addEventListener('keydown', handleKeydown)
  await Promise.all([store.fetchProblems(), classStore.fetchSubjects()])
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.problem-row:hover {
  border-color: var(--color-border-strong) !important;
  background: var(--color-bg-tertiary) !important;
}
</style>
