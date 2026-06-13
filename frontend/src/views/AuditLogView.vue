<template>
  <div class="layout">
    <AppSidebar />

    <main class="main-content">
      <div class="page-header">
        <h1 class="page-title">감사 로그</h1>
        <div class="header-controls">
          <select v-model="filterType" class="filter-select" @change="load">
            <option value="">전체 유형</option>
            <option v-for="t in ACTION_TYPES" :key="t.value" :value="t.value">{{ t.label }}</option>
          </select>
          <button class="btn-refresh" @click="load">
            <IconRefresh :size="15" />
            새로고침
          </button>
        </div>
      </div>

      <div v-if="loading" class="center-state">
        <div class="spinner" />
        <span>불러오는 중...</span>
      </div>

      <div v-else-if="error" class="error-banner">{{ error }}</div>

      <div v-else-if="logs.length === 0" class="empty-state">
        기록된 감사 로그가 없습니다.
      </div>

      <template v-else>
        <div class="log-table">
          <div class="table-head">
            <span class="col-time">시각</span>
            <span class="col-actor">수행자</span>
            <span class="col-action">행동 유형</span>
            <span class="col-target">대상</span>
            <span class="col-detail">상세</span>
          </div>
          <div
            v-for="log in logs"
            :key="log.id"
            class="table-row"
          >
            <span class="col-time text-mono">{{ formatDate(log.created_at) }}</span>
            <span class="col-actor">{{ log.actor_name ?? '시스템' }}</span>
            <span class="col-action">
              <span class="action-badge" :class="actionClass(log.action_type)">
                {{ actionLabel(log.action_type) }}
              </span>
            </span>
            <span class="col-target text-secondary">
              {{ log.target_type ? `${log.target_type} #${log.target_id}` : '-' }}
            </span>
            <span class="col-detail text-secondary">{{ log.detail ?? '-' }}</span>
          </div>
        </div>

        <div class="pagination">
          <button :disabled="offset === 0" @click="prevPage">← 이전</button>
          <span class="page-info">{{ offset / PAGE_SIZE + 1 }}페이지</span>
          <button :disabled="logs.length < PAGE_SIZE" @click="nextPage">다음 →</button>
        </div>
      </template>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { IconRefresh } from '@tabler/icons-vue'
import AppSidebar from '@/components/AppSidebar.vue'
import { api, type AuditLogRow } from '@/api/client'

const PAGE_SIZE = 100

const ACTION_TYPES = [
  { value: 'grade', label: '채점' },
  { value: 'session_start', label: '세션 시작' },
  { value: 'session_close', label: '세션 종료' },
  { value: 'result_release', label: '결과 공개' },
  { value: 'password_reset', label: '비밀번호 재설정' },
  { value: 'lesson_release', label: '차시 공개' },
  { value: 'problem_create', label: '문제 등록' },
  { value: 'assessment_create', label: '수행평가 등록' },
]

const logs = ref<AuditLogRow[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const filterType = ref('')
const offset = ref(0)

async function load() {
  loading.value = true
  error.value = null
  try {
    logs.value = await api.audit.list({
      limit: PAGE_SIZE,
      offset: offset.value,
      action_type: filterType.value || undefined,
    })
  } catch (e) {
    error.value = e instanceof Error ? e.message : '감사 로그를 불러오지 못했습니다'
  } finally {
    loading.value = false
  }
}

function prevPage() {
  offset.value = Math.max(0, offset.value - PAGE_SIZE)
  load()
}

function nextPage() {
  offset.value += PAGE_SIZE
  load()
}

function formatDate(s: string): string {
  return s.replace('T', ' ').slice(0, 16)
}

function actionLabel(type: string): string {
  const map: Record<string, string> = {
    grade: '채점',
    session_start: '세션시작',
    session_close: '세션종료',
    result_release: '결과공개',
    password_reset: '비번재설정',
    lesson_release: '차시공개',
    problem_create: '문제등록',
    assessment_create: '수행평가등록',
    division_create: '분반등록',
    teacher_create: '교사등록',
    student_create: '학생등록',
    session_pause: '일시정지',
    session_resume: '재개',
  }
  return map[type] ?? type
}

function actionClass(type: string): string {
  if (type.includes('start') || type.includes('resume')) return 'badge-success'
  if (type.includes('close') || type.includes('pause')) return 'badge-warning'
  if (type === 'grade') return 'badge-info'
  if (type === 'result_release') return 'badge-success'
  if (type === 'password_reset') return 'badge-danger'
  return 'badge-neutral'
}

onMounted(load)
</script>

<style scoped>
.layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
  background: var(--color-background-secondary);
}

.main-content {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
  min-width: 0;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1.25rem;
}

.page-title {
  font-size: 18px;
  font-weight: 700;
  margin: 0;
}

.header-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.filter-select {
  padding: 6px 10px;
  border: 0.5px solid var(--color-border-primary);
  border-radius: var(--border-radius-md);
  background: var(--color-background-primary);
  color: var(--color-text-primary);
}

.btn-refresh {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 6px 12px;
  border-radius: var(--border-radius-md);
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-secondary);
  color: var(--color-text-secondary);
  cursor: pointer;
}

.btn-refresh:hover { background: var(--color-background-secondary); }

.center-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem;
  gap: 12px;
  color: var(--color-text-secondary);
}

.error-banner {
  background: var(--color-background-danger);
  color: var(--color-text-danger);
  border: 0.5px solid var(--color-border-danger);
  border-radius: var(--border-radius-md);
  padding: 10px 14px;
  margin-bottom: 1rem;
}

.empty-state {
  text-align: center;
  color: var(--color-text-tertiary);
  padding: 4rem;
}

/* ── 테이블 ── */
.log-table {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-lg);
  overflow: hidden;
  margin-bottom: 1rem;
}

.table-head {
  display: grid;
  grid-template-columns: 140px 100px 120px 160px 1fr;
  gap: 0;
  background: var(--color-background-secondary);
  border-bottom: 0.5px solid var(--color-border-secondary);
  padding: 8px 14px;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.table-row {
  display: grid;
  grid-template-columns: 140px 100px 120px 160px 1fr;
  gap: 0;
  padding: 9px 14px;
  border-bottom: 0.5px solid var(--color-border-tertiary);
  align-items: center;
}

.table-row:last-child { border-bottom: none; }
.table-row:hover { background: var(--color-background-secondary); }

.text-mono { font-family: monospace; }
.text-secondary { color: var(--color-text-secondary); }

.action-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: var(--border-radius-md);
  font-weight: 500;
}

.badge-success { background: var(--color-background-success); color: var(--color-text-success); }
.badge-warning  { background: var(--color-background-warning); color: var(--color-text-warning); }
.badge-info     { background: var(--color-background-info); color: var(--color-text-info); }
.badge-danger   { background: var(--color-background-danger); color: var(--color-text-danger); }
.badge-neutral  { background: var(--color-background-secondary); color: var(--color-text-secondary); }

/* ── 페이지네이션 ── */
.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.page-info { color: var(--color-text-secondary); }

/* ── 스피너 ── */
.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--color-border-secondary);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }
</style>
