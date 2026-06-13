<template>
  <div class="layout">
    <AppSidebar />

    <div class="main-split">
      <!-- 좌: 세션 목록 -->
      <div class="list-panel">
        <div class="panel-header">
          <span class="panel-title">시험 세션</span>
          <button class="btn-icon" title="새 세션 생성" @click="showCreateModal = true">
            <IconPlus :size="15" />
          </button>
        </div>

        <div class="filter-group">
          <select v-model="filterDivisionId" class="filter-select" @change="loadSessions">
            <option :value="null">전체 분반</option>
            <option v-for="d in divisions" :key="d.id" :value="d.id">{{ d.name }}</option>
          </select>
          <select v-model="filterStatus" class="filter-select" @change="loadSessions">
            <option value="">전체 상태</option>
            <option value="CREATED">CREATED</option>
            <option value="LOBBY">LOBBY</option>
            <option value="RUNNING">RUNNING</option>
            <option value="CLOSED">CLOSED</option>
          </select>
        </div>

        <div v-if="sessionStore.loading && sessionStore.sessions.length === 0" class="panel-loading">
          <div class="spinner" />
        </div>
        <div v-else-if="sessionStore.sessions.length === 0" class="panel-empty">세션이 없습니다</div>
        <div v-else class="session-list">
          <div
            v-for="s in sessionStore.sessions"
            :key="s.id"
            class="session-item"
            :class="{ 'session-item--active': selectedId === s.id }"
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
      </div>

      <!-- 우: 세션 상세 -->
      <div class="detail-panel">
        <div v-if="!selected" class="empty-state">
          <IconCalendarEvent :size="40" />
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

          <!-- 액션 버튼 -->
          <div class="section-card">
            <h3 class="section-title">세션 제어</h3>
            <div class="action-buttons">
              <button
                v-if="selected.status === 'CREATED'"
                class="btn-action btn-action--info"
                :disabled="transitioning"
                @click="doTransition('to_lobby')"
              >
                <IconDoor :size="15" /> 대기실 열기 (LOBBY)
              </button>
              <button
                v-if="selected.status === 'LOBBY'"
                class="btn-action btn-action--secondary"
                :disabled="transitioning"
                @click="doTransition('to_created')"
              >
                <IconArrowBack :size="15" /> 대기실 취소
              </button>
              <button
                v-if="selected.status === 'LOBBY'"
                class="btn-action btn-action--danger"
                :disabled="transitioning"
                @click="confirmStart"
              >
                <IconPlayerPlay :size="15" /> 시험 시작 (RUNNING)
              </button>
              <button
                v-if="selected.status === 'RUNNING'"
                class="btn-action btn-action--warning"
                :disabled="transitioning"
                @click="doPause"
              >
                <IconPlayerPause v-if="!selected.is_paused" :size="15" />
                <IconPlayerPlay v-else :size="15" />
                {{ selected.is_paused ? '재개' : '일시정지' }}
              </button>
              <button
                v-if="selected.status === 'RUNNING'"
                class="btn-action btn-action--danger"
                :disabled="transitioning"
                @click="confirmClose"
              >
                <IconPlayerStop :size="15" /> 시험 종료 (CLOSED)
              </button>
              <button
                v-if="selected.status === 'CLOSED'"
                class="btn-action"
                :class="selected.is_result_released ? 'btn-action--warning' : 'btn-action--info'"
                :disabled="transitioning"
                @click="doToggleRelease"
              >
                <IconEye v-if="!selected.is_result_released" :size="15" />
                <IconEyeOff v-else :size="15" />
                {{ selected.is_result_released ? '결과 비공개' : '결과 공개' }}
              </button>
              <RouterLink
                v-if="selected.status === 'RUNNING' || selected.status === 'CLOSED'"
                :to="{ name: 'session-grading', params: { id: selected.id } }"
                class="btn-action btn-action--secondary"
              >
                채점하기
              </RouterLink>
            </div>
          </div>

          <!-- 출결 위젯 -->
          <AttendanceWidget
            v-if="selected.status === 'LOBBY' || selected.status === 'RUNNING'"
            :session-id="selected.id"
            class="attendance-section"
          />

          <!-- 세션 정보 -->
          <div class="section-card">
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
      </div>
    </div>
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
          <label class="form-label">대상 *</label>
          <div class="target-options">
            <label class="target-option" :class="{ 'target-option--active': createForm.target_type === 'ALL' }">
              <input type="radio" v-model="createForm.target_type" value="ALL" />
              <div>
                <div class="target-option-title">분반 전체</div>
                <div class="target-option-desc">명부에 등록된 학생 전체가 참여합니다.</div>
              </div>
            </label>
            <label class="target-option" :class="{ 'target-option--active': createForm.target_type === 'INDIVIDUAL' }">
              <input type="radio" v-model="createForm.target_type" value="INDIVIDUAL" />
              <div>
                <div class="target-option-title">개별 학생 선택</div>
                <div class="target-option-desc">재시험·추가시험 등 특정 학생만 대상으로 합니다.</div>
              </div>
            </label>
          </div>

          <div v-if="createForm.target_type === 'INDIVIDUAL'" class="student-checklist">
            <div v-if="!createForm.division_id" class="checklist-empty">분반을 먼저 선택하세요</div>
            <template v-else>
              <div class="checklist-search">
                <IconSearch :size="14" />
                <input
                  v-model="studentSearch"
                  class="checklist-search-input"
                  placeholder="이름 또는 학번 검색"
                />
              </div>
              <div class="checklist-count">{{ createForm.selected_student_ids.length }}명 선택됨</div>
              <div v-if="filteredStudents.length === 0" class="checklist-empty">학생이 없습니다</div>
              <div v-else class="checklist-list">
                <label v-for="s in filteredStudents" :key="s.id" class="checklist-item">
                  <input type="checkbox" :value="s.id" v-model="createForm.selected_student_ids" />
                  <span class="checklist-num">{{ s.student_number }}</span>
                  <span class="checklist-name">{{ s.name }}</span>
                </label>
              </div>
            </template>
          </div>
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
        <p class="form-hint">세션은 <strong>CREATED</strong> 상태로 생성됩니다. 이후 LOBBY로 전환해 학생을 대기시킨 뒤 시작할 수 있습니다.</p>
      </div>
      <div class="modal-footer">
        <button @click="showCreateModal = false">취소</button>
        <button class="btn-primary-action" :disabled="creating" @click="doCreateSession">
          {{ creating ? '생성 중...' : '생성' }}
        </button>
      </div>
    </div>
  </div>

  <!-- 확인 모달 -->
  <div v-if="confirmModal.show" class="modal-backdrop" @click.self="confirmModal.show = false">
    <div class="modal modal-sm">
      <div class="modal-header">
        <h3>{{ confirmModal.title }}</h3>
      </div>
      <div class="modal-body">
        <p>{{ confirmModal.message }}</p>
      </div>
      <div class="modal-footer">
        <button @click="confirmModal.show = false">취소</button>
        <button class="btn-danger-action" @click="confirmModal.onConfirm">확인</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import {
  IconPlus, IconCalendarEvent,
  IconDoor, IconArrowBack, IconPlayerPlay, IconPlayerPause,
  IconPlayerStop, IconEye, IconEyeOff, IconSearch,
} from '@tabler/icons-vue'
import { useSessionStore } from '@/stores/session'
import { useDivisionStore } from '@/stores/division'
import { useAssessmentStore } from '@/stores/assessment'
import AttendanceWidget from '@/components/AttendanceWidget.vue'
import AppSidebar from '@/components/AppSidebar.vue'
import { api } from '@/api/client'
import type { SessionRow, StudentRow } from '@/api/client'

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
  target_type: 'ALL' as 'ALL' | 'INDIVIDUAL',
  selected_student_ids: [] as number[],
})
const divisionStudents = ref<StudentRow[]>([])
const studentSearch = ref('')

const filteredStudents = computed(() => {
  const q = studentSearch.value.trim().toLowerCase()
  if (!q) return divisionStudents.value
  return divisionStudents.value.filter(s =>
    s.name.toLowerCase().includes(q) || s.student_number.includes(q)
  )
})

watch(() => createForm.value.division_id, async (id) => {
  createForm.value.selected_student_ids = []
  studentSearch.value = ''
  if (id) {
    divisionStudents.value = await api.divisions.getStudents(id).catch(() => [])
  } else {
    divisionStudents.value = []
  }
})

watch(() => createForm.value.target_type, () => {
  createForm.value.selected_student_ids = []
})

watch(showCreateModal, (open) => {
  if (!open) {
    createForm.value = {
      assessment_id: 0, division_id: 0, time_limit_min: null,
      target_type: 'ALL', selected_student_ids: [],
    }
    divisionStudents.value = []
    studentSearch.value = ''
    createError.value = null
  }
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
  if (createForm.value.target_type === 'INDIVIDUAL' && createForm.value.selected_student_ids.length === 0) {
    createError.value = '개별 학생을 1명 이상 선택하세요'
    return
  }

  creating.value = true
  try {
    const row = await sessionStore.createSession({
      assessment_id: createForm.value.assessment_id,
      division_id: createForm.value.division_id,
      time_limit_min: createForm.value.time_limit_min ?? undefined,
      target_type: createForm.value.target_type,
      student_ids: createForm.value.target_type === 'INDIVIDUAL'
        ? createForm.value.selected_student_ids
        : undefined,
    })
    showCreateModal.value = false
    selectedId.value = row.id
  } catch (e) {
    createError.value = e instanceof Error ? e.message : '세션 생성 실패'
  } finally {
    creating.value = false
  }
}
</script>

<style scoped>
/* ── 레이아웃 ── */
.layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
  background: var(--color-background-secondary);
}

.main-split {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-width: 0;
}

/* ── 목록 패널 ── */
.list-panel {
  width: 260px;
  flex-shrink: 0;
  border-right: 0.5px solid var(--color-border-secondary);
  background: var(--color-background-primary);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 14px 10px;
  border-bottom: 0.5px solid var(--color-border-secondary);
}

.panel-title {
  font-weight: 600;
}

.btn-icon {
  width: 28px;
  height: 28px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--border-radius-md);
  border: 0.5px solid var(--color-border-secondary);
  background: none;
  color: var(--color-text-secondary);
  cursor: pointer;
}

.btn-icon:hover { background: var(--color-background-secondary); }

.filter-group {
  padding: 8px 10px;
  border-bottom: 0.5px solid var(--color-border-secondary);
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.filter-select {
  width: 100%;
  padding: 5px 8px;
  border: 0.5px solid var(--color-border-primary);
  border-radius: var(--border-radius-sm);
  background: var(--color-background-primary);
  color: var(--color-text-primary);
}

.panel-loading {
  display: flex;
  justify-content: center;
  padding: 2rem;
}

.panel-empty {
  padding: 1rem;
  text-align: center;
  color: var(--color-text-tertiary);
}

.session-list {
  flex: 1;
  overflow-y: auto;
  padding: 6px;
}

.session-item {
  padding: 10px;
  border-radius: var(--border-radius-md);
  cursor: pointer;
  border: 0.5px solid transparent;
  margin-bottom: 3px;
  transition: background 0.1s;
}

.session-item:hover { background: var(--color-background-secondary); }

.session-item--active {
  border-color: var(--color-border-info);
  background: var(--color-background-info);
}

.session-item-top {
  display: flex;
  align-items: center;
  gap: 5px;
  margin-bottom: 4px;
}

.session-item-title {
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.session-item-sub {
  color: var(--color-text-secondary);
}

.status-badge {
  display: inline-block;
  padding: 1px 6px;
  border-radius: var(--border-radius-sm);
  font-weight: 600;
}

.badge-created { background: var(--color-background-secondary); color: var(--color-text-secondary); }
.badge-lobby   { background: var(--color-background-info); color: var(--color-text-info); }
.badge-running { background: var(--color-background-success); color: var(--color-text-success); }
.badge-closed  { background: var(--color-background-secondary); color: var(--color-text-tertiary); }

.pause-badge {
  background: var(--color-background-warning);
  color: var(--color-text-warning);
  padding: 1px 6px;
  border-radius: var(--border-radius-sm);
}

/* ── 상세 패널 ── */
.detail-panel {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
  min-width: 0;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 300px;
  gap: 12px;
  color: var(--color-text-tertiary);
}

.session-detail { max-width: 720px; }

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 1.25rem;
}

.detail-title {
  font-size: 18px;
  font-weight: 700;
  margin: 0 0 4px;
}

.detail-sub {
  color: var(--color-text-secondary);
  margin: 0;
}

.status-header-badges {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
}

.status-badge-lg {
  padding: 4px 12px;
  border-radius: var(--border-radius-md);
  font-weight: 700;
}

.pause-badge-lg {
  background: var(--color-background-warning);
  color: var(--color-text-warning);
  padding: 4px 10px;
  border-radius: var(--border-radius-sm);
}

.error-banner {
  background: var(--color-background-danger);
  color: var(--color-text-danger);
  border: 0.5px solid var(--color-border-danger);
  border-radius: var(--border-radius-md);
  padding: 10px 14px;
  margin-bottom: 1rem;
}

.stat-cards {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
  margin-bottom: 1rem;
}

.stat-card {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-md);
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
}

.stat-label {
  color: var(--color-text-secondary);
}

.section-card {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-md);
  padding: 1rem 1.25rem;
  margin-bottom: 1rem;
}

.section-title {
  font-weight: 600;
  margin: 0 0 10px;
  color: var(--color-text-secondary);
}

.attendance-section { margin-bottom: 1rem; }

.action-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.btn-action {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 7px 14px;
  border-radius: var(--border-radius-md);
  font-weight: 500;
  cursor: pointer;
  border: 0.5px solid transparent;
  text-decoration: none;
  transition: opacity 0.1s;
}

.btn-action:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-action--info     { background: var(--color-background-info); color: var(--color-text-info); border-color: var(--color-border-info); }
.btn-action--danger   { background: var(--color-background-danger); color: var(--color-text-danger); border-color: var(--color-border-danger); }
.btn-action--warning  { background: var(--color-background-warning); color: var(--color-text-warning); }
.btn-action--secondary { background: var(--color-background-primary); color: var(--color-text-secondary); border-color: var(--color-border-secondary); }

.info-table {
  width: 100%;
  border-collapse: collapse;
}

.info-table td {
  padding: 7px 12px;
  border-bottom: 0.5px solid var(--color-border-tertiary);
}

.info-table td:first-child {
  color: var(--color-text-secondary);
  width: 100px;
}

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

/* ── 모달 ── */
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
  background: var(--color-background-primary);
  border-radius: var(--border-radius-lg);
  width: 480px;
  max-width: 95vw;
  box-shadow: 0 8px 32px rgba(0,0,0,0.15);
}

.modal-sm { width: 360px; }

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 16px;
  border-bottom: 0.5px solid var(--color-border-secondary);
}

.modal-header h3 { margin: 0; font-size: 15px; }

.modal-close {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: var(--color-text-secondary);
  line-height: 1;
  padding: 0;
}

.modal-body { padding: 16px; }

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 0.5px solid var(--color-border-secondary);
}

.form-group { margin-bottom: 12px; }

.form-label {
  display: block;
  font-weight: 500;
  margin-bottom: 5px;
  color: var(--color-text-primary);
}

.form-input {
  width: 100%;
  padding: 7px 10px;
  border: 0.5px solid var(--color-border-primary);
  border-radius: var(--border-radius-md);
  background: var(--color-background-primary);
  color: var(--color-text-primary);
  box-sizing: border-box;
}

.form-hint {
  color: var(--color-text-secondary);
  margin-top: 4px;
}

.btn-primary-action {
  background: var(--color-background-info);
  color: var(--color-text-info);
  border-color: var(--color-border-info);
  font-weight: 500;
  padding: 7px 16px;
}

.btn-danger-action {
  background: var(--color-background-danger);
  color: var(--color-text-danger);
  border-color: var(--color-border-danger);
  font-weight: 500;
  padding: 7px 16px;
}

/* ── 대상 선택 ── */
.target-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.target-option {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px;
  border: 0.5px solid var(--color-border-primary);
  border-radius: var(--border-radius-md);
  cursor: pointer;
  transition: border-color 0.1s, background 0.1s;
}

.target-option--active {
  border-color: var(--color-border-info);
  background: var(--color-background-info);
}

.target-option input[type="radio"] { flex-shrink: 0; margin-top: 2px; }

.target-option-title {
  font-weight: 500;
}

.target-option-desc {
  color: var(--color-text-secondary);
  margin-top: 2px;
}

.student-checklist {
  margin-top: 8px;
  border: 0.5px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md);
  padding: 8px;
}

.checklist-search {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 8px;
  border: 0.5px solid var(--color-border-primary);
  border-radius: var(--border-radius-sm);
  margin-bottom: 8px;
  color: var(--color-text-secondary);
}

.checklist-search-input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  color: var(--color-text-primary);
}

.checklist-count {
  color: var(--color-text-info);
  font-weight: 500;
  margin-bottom: 4px;
}

.checklist-empty {
  color: var(--color-text-tertiary);
  text-align: center;
  padding: 10px 0;
}

.checklist-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-height: 180px;
  overflow-y: auto;
}

.checklist-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
  border-radius: var(--border-radius-sm);
  cursor: pointer;
}

.checklist-item:hover { background: var(--color-background-secondary); }

.checklist-num { color: var(--color-text-secondary); min-width: 60px; }
.checklist-name { color: var(--color-text-primary); }
</style>
