<template>
  <div class="page-layout">
    <!-- 사이드바 -->
    <aside class="sidebar">
      <div class="sidebar-header">
        <RouterLink to="/dashboard" class="back-link">
          <IconArrowLeft :size="16" /> 대시보드
        </RouterLink>
        <h2 class="sidebar-title">세션 관리</h2>
      </div>

      <!-- 필터 -->
      <div class="filter-group">
        <label class="filter-label">분반 필터</label>
        <select v-model="filterDivisionId" class="filter-select" @change="loadSessions">
          <option :value="null">전체 분반</option>
          <option v-for="d in divisions" :key="d.id" :value="d.id">{{ d.name }}</option>
        </select>
        <label class="filter-label" style="margin-top: var(--space-2)">상태 필터</label>
        <select v-model="filterStatus" class="filter-select" @change="loadSessions">
          <option value="">전체</option>
          <option value="CREATED">CREATED</option>
          <option value="LOBBY">LOBBY</option>
          <option value="RUNNING">RUNNING</option>
          <option value="CLOSED">CLOSED</option>
        </select>
      </div>

      <!-- 세션 목록 -->
      <div class="session-list">
        <div v-if="sessionStore.loading" class="list-loading">로딩 중...</div>
        <div v-else-if="sessionStore.sessions.length === 0" class="list-empty">
          세션이 없습니다
        </div>
        <div
          v-for="s in sessionStore.sessions"
          :key="s.id"
          class="session-item"
          :class="{ selected: selectedId === s.id, [`status-${s.status.toLowerCase()}`]: true }"
          @click="selectSession(s)"
        >
          <div class="session-item-top">
            <span class="status-badge" :class="`badge-${s.status.toLowerCase()}`">{{ s.status }}</span>
            <span v-if="s.is_paused" class="pause-badge">일시정지</span>
          </div>
          <div class="session-item-title">{{ s.assessment_title }}</div>
          <div class="session-item-sub">{{ s.division_name }}</div>
        </div>
      </div>

      <!-- 새 세션 생성 버튼 -->
      <div class="sidebar-footer">
        <button class="btn btn-primary" style="width:100%" @click="showCreateModal = true">
          <IconPlus :size="16" /> 새 세션 생성
        </button>
      </div>
    </aside>

    <!-- 메인 콘텐츠 -->
    <main class="main-content">
      <div v-if="!selected" class="empty-state">
        <IconCalendarEvent :size="48" />
        <p>왼쪽에서 세션을 선택하거나 새 세션을 생성하세요</p>
      </div>

      <div v-else class="session-detail">
        <!-- 상태 헤더 -->
        <div class="detail-header">
          <div>
            <h2 class="detail-title">{{ selected.assessment_title }}</h2>
            <p class="detail-sub">{{ selected.division_name }} · {{ selected.target_type === 'ALL' ? '전체 학생' : '개별 지정' }}</p>
          </div>
          <div class="status-header-badges">
            <span class="status-badge-lg" :class="`badge-${selected.status.toLowerCase()}`">
              {{ STATUS_LABELS[selected.status] }}
            </span>
            <span v-if="selected.is_paused" class="pause-badge-lg">⏸ 일시정지</span>
          </div>
        </div>

        <!-- 에러 배너 -->
        <div v-if="actionError" class="error-banner">{{ actionError }}</div>

        <!-- 통계 카드 -->
        <div class="stat-cards">
          <div class="stat-card">
            <span class="stat-value">{{ selected.student_count }}</span>
            <span class="stat-label">대상 학생</span>
          </div>
          <div class="stat-card">
            <span class="stat-value">{{ selected.submission_count }}</span>
            <span class="stat-label">제출 수</span>
          </div>
          <div class="stat-card">
            <span class="stat-value">{{ selected.time_limit_min ?? '무제한' }}</span>
            <span class="stat-label">제한 시간(분)</span>
          </div>
          <div class="stat-card">
            <span class="stat-value">{{ selected.start_at ? formatTime(selected.start_at) : '-' }}</span>
            <span class="stat-label">시작 시각</span>
          </div>
        </div>

        <!-- 액션 버튼 영역 -->
        <div class="action-section">
          <h3 class="section-title">세션 제어</h3>
          <div class="action-buttons">
            <!-- CREATED → LOBBY -->
            <button
              v-if="selected.status === 'CREATED'"
              class="btn btn-primary"
              :disabled="transitioning"
              @click="doTransition('to_lobby')"
            >
              <IconDoor :size="16" /> 대기실 열기 (LOBBY)
            </button>

            <!-- LOBBY → CREATED (취소) -->
            <button
              v-if="selected.status === 'LOBBY'"
              class="btn btn-secondary"
              :disabled="transitioning"
              @click="doTransition('to_created')"
            >
              <IconArrowBack :size="16" /> 대기실 취소
            </button>

            <!-- LOBBY → RUNNING -->
            <button
              v-if="selected.status === 'LOBBY'"
              class="btn btn-danger"
              :disabled="transitioning"
              @click="confirmStart"
            >
              <IconPlayerPlay :size="16" /> 시험 시작 (RUNNING)
            </button>

            <!-- RUNNING → 일시정지/재개 -->
            <button
              v-if="selected.status === 'RUNNING'"
              class="btn btn-warning"
              :disabled="transitioning"
              @click="doPause"
            >
              <IconPlayerPause v-if="!selected.is_paused" :size="16" />
              <IconPlayerPlay v-else :size="16" />
              {{ selected.is_paused ? '재개' : '일시정지' }}
            </button>

            <!-- RUNNING → CLOSED -->
            <button
              v-if="selected.status === 'RUNNING'"
              class="btn btn-danger"
              :disabled="transitioning"
              @click="confirmClose"
            >
              <IconPlayerStop :size="16" /> 시험 종료 (CLOSED)
            </button>

            <!-- CLOSED → 결과 공개 토글 -->
            <button
              v-if="selected.status === 'CLOSED'"
              class="btn"
              :class="selected.is_result_released ? 'btn-warning' : 'btn-primary'"
              :disabled="transitioning"
              @click="doToggleRelease"
            >
              <IconEye v-if="!selected.is_result_released" :size="16" />
              <IconEyeOff v-else :size="16" />
              {{ selected.is_result_released ? '결과 비공개' : '결과 공개' }}
            </button>

            <!-- RUNNING/CLOSED → 채점 -->
            <RouterLink
              v-if="selected.status === 'RUNNING' || selected.status === 'CLOSED'"
              :to="{ name: 'session-grading', params: { id: selected.id } }"
              class="btn btn-secondary"
            >
              채점하기
            </RouterLink>
          </div>
        </div>

        <!-- 출결 위젯 (LOBBY / RUNNING) -->
        <AttendanceWidget
          v-if="selected.status === 'LOBBY' || selected.status === 'RUNNING'"
          :session-id="selected.id"
          class="attendance-section"
        />

        <!-- 세션 정보 -->
        <div class="info-section">
          <h3 class="section-title">세션 정보</h3>
          <table class="info-table">
            <tbody>
              <tr><td>세션 ID</td><td>#{{ selected.id }}</td></tr>
              <tr><td>수행평가</td><td>{{ selected.assessment_title }}</td></tr>
              <tr><td>분반</td><td>{{ selected.division_name }}</td></tr>
              <tr><td>대상</td><td>{{ selected.target_type === 'ALL' ? '전체 학생' : '개별 지정' }}</td></tr>
              <tr><td>생성일</td><td>{{ selected.created_at }}</td></tr>
              <tr v-if="selected.start_at"><td>시작</td><td>{{ selected.start_at }}</td></tr>
              <tr v-if="selected.end_at"><td>종료</td><td>{{ selected.end_at }}</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </main>
  </div>

  <!-- 세션 생성 모달 -->
  <div v-if="showCreateModal" class="modal-backdrop" @click.self="showCreateModal = false">
    <div class="modal">
      <div class="modal-header">
        <h3>새 세션 생성</h3>
        <button class="modal-close" @click="showCreateModal = false">×</button>
      </div>
      <div class="modal-body">
        <div v-if="createError" class="error-banner">{{ createError }}</div>
        <div class="form-group">
          <label class="form-label">수행평가 *</label>
          <select v-model="createForm.assessment_id" class="form-input">
            <option :value="0">선택하세요</option>
            <option v-for="a in assessments" :key="a.id" :value="a.id">{{ a.title }}</option>
          </select>
        </div>
        <div class="form-group">
          <label class="form-label">분반 *</label>
          <select v-model="createForm.division_id" class="form-input">
            <option :value="0">선택하세요</option>
            <option v-for="d in divisions" :key="d.id" :value="d.id">{{ d.name }}</option>
          </select>
        </div>
        <div class="form-group">
          <label class="form-label">제한 시간 (분, 비워두면 무제한)</label>
          <input
            v-model.number="createForm.time_limit_min"
            type="number"
            min="1"
            class="form-input"
            placeholder="예: 60"
          />
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" @click="showCreateModal = false">취소</button>
        <button class="btn btn-primary" :disabled="creating" @click="doCreateSession">
          {{ creating ? '생성 중...' : '생성' }}
        </button>
      </div>
    </div>
  </div>

  <!-- 확인 모달 (시작/종료 등 비가역 전환) -->
  <div v-if="confirmModal.show" class="modal-backdrop" @click.self="confirmModal.show = false">
    <div class="modal modal-sm">
      <div class="modal-header">
        <h3>{{ confirmModal.title }}</h3>
      </div>
      <div class="modal-body">
        <p>{{ confirmModal.message }}</p>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" @click="confirmModal.show = false">취소</button>
        <button class="btn btn-danger" @click="confirmModal.onConfirm">확인</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  IconArrowLeft, IconPlus, IconCalendarEvent,
  IconDoor, IconArrowBack, IconPlayerPlay, IconPlayerPause,
  IconPlayerStop, IconEye, IconEyeOff,
} from '@tabler/icons-vue'
import { useSessionStore } from '@/stores/session'
import { useDivisionStore } from '@/stores/division'
import { useAssessmentStore } from '@/stores/assessment'
import AttendanceWidget from '@/components/AttendanceWidget.vue'
import type { SessionRow } from '@/api/client'

const sessionStore = useSessionStore()
const divisionStore = useDivisionStore()
const assessmentStore = useAssessmentStore()

const STATUS_LABELS: Record<string, string> = {
  CREATED: '생성됨',
  LOBBY: '대기실',
  RUNNING: '진행 중',
  CLOSED: '종료됨',
}

const filterDivisionId = ref<number | null>(null)
const filterStatus = ref('')
const selectedId = ref<number | null>(null)
const actionError = ref<string | null>(null)
const transitioning = ref(false)

const selected = computed(() =>
  selectedId.value !== null
    ? sessionStore.sessions.find(s => s.id === selectedId.value) ?? null
    : null
)

const divisions = computed(() => divisionStore.divisions)
const assessments = computed(() => assessmentStore.assessments)

const showCreateModal = ref(false)
const creating = ref(false)
const createError = ref<string | null>(null)
const createForm = ref({
  assessment_id: 0,
  division_id: 0,
  time_limit_min: null as number | null,
})

const confirmModal = ref({
  show: false,
  title: '',
  message: '',
  onConfirm: () => {},
})

onMounted(async () => {
  await Promise.all([
    loadSessions(),
    divisionStore.fetchDivisions(),
    assessmentStore.fetchAssessments(),
  ])
})

async function loadSessions() {
  await sessionStore.fetchSessions({
    division_id: filterDivisionId.value ?? undefined,
    status: filterStatus.value || undefined,
  })
}

function selectSession(s: SessionRow) {
  selectedId.value = s.id
  actionError.value = null
}

function formatTime(iso: string): string {
  return iso.replace('T', ' ').slice(0, 16)
}

async function doTransition(action: string) {
  if (!selected.value) return
  transitioning.value = true
  actionError.value = null
  try {
    await sessionStore.transition(selected.value.id, action)
    await loadSessions()
  } catch (e) {
    actionError.value = e instanceof Error ? e.message : '전환 실패'
  } finally {
    transitioning.value = false
  }
}

function confirmStart() {
  confirmModal.value = {
    show: true,
    title: '시험 시작',
    message: '시험을 시작하면 LOBBY → RUNNING으로 전환되며, 되돌릴 수 없습니다. 시작하시겠습니까?',
    onConfirm: async () => {
      confirmModal.value.show = false
      await doTransition('to_running')
    },
  }
}

function confirmClose() {
  confirmModal.value = {
    show: true,
    title: '시험 종료',
    message: '시험을 종료하면 RUNNING → CLOSED로 전환되며, 되돌릴 수 없습니다. 종료하시겠습니까?',
    onConfirm: async () => {
      confirmModal.value.show = false
      await doTransition('close')
    },
  }
}

async function doPause() {
  if (!selected.value) return
  transitioning.value = true
  actionError.value = null
  try {
    await sessionStore.pause(selected.value.id)
    await loadSessions()
  } catch (e) {
    actionError.value = e instanceof Error ? e.message : '일시정지 실패'
  } finally {
    transitioning.value = false
  }
}

async function doToggleRelease() {
  if (!selected.value) return
  transitioning.value = true
  actionError.value = null
  try {
    await sessionStore.toggleResultRelease(selected.value.id)
    await loadSessions()
  } catch (e) {
    actionError.value = e instanceof Error ? e.message : '결과 공개 전환 실패'
  } finally {
    transitioning.value = false
  }
}

async function doCreateSession() {
  createError.value = null
  if (!createForm.value.assessment_id) { createError.value = '수행평가를 선택하세요'; return }
  if (!createForm.value.division_id) { createError.value = '분반을 선택하세요'; return }

  creating.value = true
  try {
    const row = await sessionStore.createSession({
      assessment_id: createForm.value.assessment_id,
      division_id: createForm.value.division_id,
      time_limit_min: createForm.value.time_limit_min ?? undefined,
    })
    showCreateModal.value = false
    createForm.value = { assessment_id: 0, division_id: 0, time_limit_min: null }
    selectedId.value = row.id
  } catch (e) {
    createError.value = e instanceof Error ? e.message : '세션 생성 실패'
  } finally {
    creating.value = false
  }
}
</script>

<style scoped>
.page-layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
  background: var(--color-background);
}

.sidebar {
  width: 280px;
  flex-shrink: 0;
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  background: var(--color-surface);
}

.sidebar-header {
  padding: var(--space-4);
  border-bottom: 1px solid var(--color-border);
}

.back-link {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  color: var(--color-text-muted);
  text-decoration: none;
  font-size: 0.875rem;
  margin-bottom: var(--space-2);
}

.back-link:hover { color: var(--color-text); }

.sidebar-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.filter-group {
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.filter-label {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.filter-select {
  width: 100%;
  padding: var(--space-1) var(--space-2);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
  background: var(--color-background);
  color: var(--color-text);
}

.session-list {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-2);
}

.list-loading,
.list-empty {
  padding: var(--space-4);
  text-align: center;
  color: var(--color-text-muted);
  font-size: 0.875rem;
}

.session-item {
  padding: var(--space-3);
  border-radius: var(--radius);
  cursor: pointer;
  border: 1px solid transparent;
  margin-bottom: var(--space-1);
  transition: background 0.15s;
}

.session-item:hover { background: var(--color-background-hover); }
.session-item.selected {
  border-color: var(--color-primary);
  background: var(--color-background-info);
}

.session-item-top {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  margin-bottom: var(--space-1);
}

.session-item-title {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text);
}

.session-item-sub {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.status-badge {
  display: inline-block;
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
}

.badge-created { background: var(--color-background-hover); color: var(--color-text-muted); }
.badge-lobby { background: var(--color-background-info); color: var(--color-primary); }
.badge-running { background: var(--color-background-success); color: var(--color-text-success); }
.badge-closed { background: var(--color-background-hover); color: var(--color-text-muted); }

.pause-badge {
  font-size: 0.7rem;
  background: var(--color-background-warning);
  color: var(--color-text-warning);
  padding: 1px 6px;
  border-radius: var(--radius-sm);
}

.sidebar-footer {
  padding: var(--space-3) var(--space-4);
  border-top: 1px solid var(--color-border);
}

.main-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-6);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: var(--space-3);
  color: var(--color-text-muted);
}

.session-detail { max-width: 720px; }

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--space-6);
}

.detail-title {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-text);
  margin: 0 0 var(--space-1);
}

.detail-sub {
  color: var(--color-text-muted);
  font-size: 0.875rem;
  margin: 0;
}

.status-header-badges {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: var(--space-1);
}

.status-badge-lg {
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius);
  font-size: 0.875rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.pause-badge-lg {
  font-size: 0.8rem;
  background: var(--color-background-warning);
  color: var(--color-text-warning);
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
}

.error-banner {
  background: var(--color-background-danger, #fef2f2);
  color: var(--color-danger, #dc2626);
  border: 1px solid var(--color-border-danger);
  border-radius: var(--radius);
  padding: var(--space-3);
  margin-bottom: var(--space-4);
  font-size: 0.875rem;
}

.stat-cards {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--space-3);
  margin-bottom: var(--space-6);
}

.stat-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  padding: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.stat-value {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-text);
}

.stat-label {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.action-section,
.info-section {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  padding: var(--space-4);
  margin-bottom: var(--space-4);
}

.attendance-section {
  margin-bottom: var(--space-4);
}

.section-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0 0 var(--space-3);
}

.action-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.info-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}

.info-table td {
  padding: var(--space-2) var(--space-3);
  border-bottom: 1px solid var(--color-border);
}

.info-table td:first-child {
  color: var(--color-text-muted);
  width: 120px;
}

/* 버튼 */
.btn {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-2) var(--space-4);
  border: 1px solid transparent;
  border-radius: var(--radius);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s;
}

.btn:disabled { opacity: 0.5; cursor: not-allowed; }

.btn-primary {
  background: var(--color-primary);
  color: white;
}

.btn-secondary {
  background: var(--color-surface);
  border-color: var(--color-border);
  color: var(--color-text);
}

.btn-danger {
  background: var(--color-danger, #dc2626);
  color: white;
}

.btn-warning {
  background: var(--color-background-warning);
  color: var(--color-text-warning);
  border-color: var(--color-border-warning, #fde68a);
}

/* 모달 */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  background: var(--color-surface);
  border-radius: var(--radius-lg, var(--radius));
  width: 480px;
  max-width: 95vw;
  box-shadow: 0 8px 32px rgba(0,0,0,0.15);
}

.modal-sm { width: 360px; }

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-4);
  border-bottom: 1px solid var(--color-border);
}

.modal-header h3 { margin: 0; font-size: 1rem; }

.modal-close {
  background: none;
  border: none;
  font-size: 1.25rem;
  cursor: pointer;
  color: var(--color-text-muted);
  line-height: 1;
}

.modal-body { padding: var(--space-4); }

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-4);
  border-top: 1px solid var(--color-border);
}

.form-group {
  margin-bottom: var(--space-3);
}

.form-label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  margin-bottom: var(--space-1);
  color: var(--color-text);
}

.form-input {
  width: 100%;
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  font-size: 0.875rem;
  background: var(--color-background);
  color: var(--color-text);
  box-sizing: border-box;
}
</style>
