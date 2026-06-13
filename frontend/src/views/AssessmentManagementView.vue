<template>
  <div class="layout">
    <aside class="sidebar">
      <div class="sidebar-logo">
        <IconSchool :size="18" stroke-width="1.5" />
        <span>{{ auth.schoolName || 'CodeClan LMS' }}</span>
      </div>
      <nav class="sidebar-nav">
        <RouterLink :to="{ name: 'dashboard' }" class="nav-item">
          <IconLayoutDashboard :size="16" />대시보드
        </RouterLink>
        <div class="nav-group-label">준비</div>
        <RouterLink :to="{ name: 'problem-bank' }" class="nav-item"><IconDatabase :size="16" />문제 은행</RouterLink>
        <RouterLink :to="{ name: 'lesson-management' }" class="nav-item"><IconList :size="16" />차시 관리</RouterLink>
        <a href="#" class="nav-item active"><IconFileText :size="16" />수행평가</a>
        <div class="nav-group-label">관리</div>
        <RouterLink :to="{ name: 'division-management' }" class="nav-item"><IconUsers :size="16" />학생/반 관리</RouterLink>
      </nav>
      <div class="sidebar-footer">
        <div class="user-info" v-if="auth.teacher">
          <span class="user-name">{{ auth.teacher.name }}</span>
          <span class="user-role">{{ auth.teacher.role === 'admin' ? '관리자' : '교사' }}</span>
        </div>
        <button class="logout-btn" @click="logout">로그아웃</button>
      </div>
    </aside>

    <div class="main-split">
      <!-- 좌: 수행평가 목록 -->
      <div class="list-panel">
        <div class="panel-header">
          <span class="panel-title">수행평가</span>
          <button class="btn-icon" @click="openCreate" title="수행평가 추가">
            <IconPlus :size="15" />
          </button>
        </div>

        <div v-if="store.loading && store.assessments.length === 0" class="panel-loading">
          <div class="spinner"></div>
        </div>
        <div v-else-if="store.error" class="panel-error">{{ store.error }}</div>
        <div v-else-if="store.assessments.length === 0" class="panel-empty">수행평가가 없습니다</div>

        <button
          v-for="a in store.assessments"
          :key="a.id"
          class="assessment-item"
          :class="{ active: selectedId === a.id, locked: a.is_locked }"
          @click="selectAssessment(a.id)"
        >
          <div class="assessment-item-top">
            <span class="assessment-title">{{ a.title }}</span>
            <span v-if="a.is_locked" class="lock-badge"><IconLock :size="10" />운영중</span>
          </div>
          <div class="assessment-meta">문항 {{ a.problem_count }}개 · 분반 {{ a.division_count }}개</div>
          <div class="assessment-date">{{ formatDate(a.created_at) }}</div>
        </button>
      </div>

      <!-- 우: 상세 / 편집 -->
      <div class="detail-panel">
        <!-- 새 수행평가 -->
        <div v-if="mode === 'create'" class="editor-wrap">
          <div class="editor-header">
            <span class="editor-title">새 수행평가</span>
            <button class="btn-icon-sm" @click="mode = 'idle'"><IconX :size="14" /></button>
          </div>
          <div class="form-card">
            <div class="field">
              <label>제목 *</label>
              <input v-model="form.title" type="text" placeholder="수행평가 제목" @keyup.enter="submitCreate" />
            </div>
            <div class="field">
              <label>설명</label>
              <textarea v-model="form.description"></textarea>
            </div>
            <div v-if="saveError" class="save-error">{{ saveError }}</div>
            <div class="form-actions">
              <button @click="mode = 'idle'">취소</button>
              <button class="btn-primary" @click="submitCreate" :disabled="saving">
                {{ saving ? '추가 중...' : '추가' }}
              </button>
            </div>
          </div>
        </div>

        <!-- 수행평가 상세 -->
        <template v-else-if="mode === 'view' && store.currentAssessment">
          <div class="editor-header">
            <div class="editor-title">
              <span>{{ store.currentAssessment.title }}</span>
              <span v-if="store.currentAssessment.is_locked" class="lock-badge-large">
                <IconLock :size="13" />편집 잠금 (RUNNING 세션 존재)
              </span>
            </div>
            <div class="editor-header-actions">
              <button v-if="!store.currentAssessment.is_locked" class="btn-secondary" @click="startEdit">
                <IconPencil :size="13" />수정
              </button>
              <button v-if="!store.currentAssessment.is_locked" class="btn-icon-sm btn-danger" @click="confirmDelete">
                <IconTrash :size="13" />
              </button>
            </div>
          </div>

          <div class="detail-body">
            <!-- 배점 문항 -->
            <section class="detail-section">
              <div class="section-header">
                <span class="section-title">배점 문항</span>
                <div class="section-right">
                  <span class="total-score">합계 {{ totalScore }}점</span>
                  <button v-if="!store.currentAssessment.is_locked" class="btn-secondary" @click="openProblemPicker">
                    <IconPlus :size="13" />문항 배정 편집
                  </button>
                </div>
              </div>
              <div v-if="store.currentAssessment.problems.length === 0" class="empty-note">배정된 문항이 없습니다</div>
              <div v-else class="problem-list">
                <div v-for="(p, i) in store.currentAssessment.problems" :key="p.id" class="problem-row">
                  <span class="prob-order">{{ i + 1 }}</span>
                  <span class="type-badge" :class="`type-${p.problem_type}`">{{ typeShort(p.problem_type) }}</span>
                  <span class="prob-title">{{ p.problem_title }}</span>
                  <span class="prob-score">{{ p.score }}점</span>
                </div>
              </div>
            </section>

            <!-- 연결 분반 -->
            <section class="detail-section">
              <div class="section-header">
                <span class="section-title">연결 분반</span>
                <button class="btn-secondary" @click="openDivisionLink">
                  <IconPlus :size="13" />분반 연결
                </button>
              </div>
              <div v-if="store.currentAssessment.divisions.length === 0" class="empty-note">
                연결된 분반이 없습니다
              </div>
              <div v-else class="division-list">
                <div v-for="d in store.currentAssessment.divisions" :key="d.division_id" class="division-row">
                  <span class="division-name">{{ d.division_name }}</span>
                  <span v-if="d.has_running_session" class="running-badge"><IconAlertTriangle :size="11" />운영중</span>
                  <button
                    v-if="!d.has_running_session"
                    class="btn-icon-sm btn-danger"
                    title="연결 해제"
                    @click="unlinkDivision(d.division_id)"
                  >
                    <IconTrash :size="12" />
                  </button>
                </div>
              </div>
            </section>
          </div>
        </template>

        <div v-else class="detail-empty">
          <IconFileText :size="32" class="empty-icon" />
          <div>수행평가를 선택하거나 새로 추가하세요</div>
        </div>

        <!-- 문항 피커 모달 -->
        <div v-if="probPicker.open" class="modal-backdrop" @click.self="probPicker.open = false">
          <div class="modal modal-wide">
            <div class="modal-title">문항 배정 편집 (배점 입력)</div>
            <input v-model="probPicker.search" type="text" placeholder="문항 검색" />
            <div class="picker-list">
              <label
                v-for="p in filteredAllProblems"
                :key="p.id"
                class="picker-item"
                :class="{ checked: pickerItems.some(i => i.problem_id === p.id) }"
              >
                <input type="checkbox" :checked="pickerItems.some(i => i.problem_id === p.id)" @change="togglePicker(p.id)" />
                <span class="type-badge" :class="`type-${p.problem_type}`">{{ typeShort(p.problem_type) }}</span>
                <span class="flex-grow">{{ p.title }}</span>
                <input
                  v-if="pickerItems.some(i => i.problem_id === p.id)"
                  type="number"
                  min="0"
                  class="score-input"
                  :value="pickerItems.find(i => i.problem_id === p.id)?.score ?? 0"
                  @input="setScore(p.id, Number(($event.target as HTMLInputElement).value))"
                  @click.stop
                  placeholder="점"
                />
              </label>
            </div>
            <div class="modal-actions">
              <button @click="probPicker.open = false">취소</button>
              <button class="btn-primary" @click="submitProblems">저장</button>
            </div>
          </div>
        </div>

        <!-- 분반 연결 모달 -->
        <div v-if="divisionPicker.open" class="modal-backdrop" @click.self="divisionPicker.open = false">
          <div class="modal">
            <div class="modal-title">분반 연결</div>
            <p class="modal-hint">복사가 아닌 참조 연결입니다. 수정 시 모든 연결 분반에 즉시 반영됩니다.</p>
            <div class="picker-list">
              <label
                v-for="d in availableDivisions"
                :key="d.id"
                class="picker-item"
                :class="{ checked: store.currentAssessment?.divisions.some(x => x.division_id === d.id) }"
              >
                <input
                  type="checkbox"
                  :checked="store.currentAssessment?.divisions.some(x => x.division_id === d.id)"
                  @change="toggleDivision(d.id)"
                />
                {{ d.name }}
                <span class="div-count">학생 {{ d.student_count }}명</span>
              </label>
            </div>
            <div class="modal-actions">
              <button @click="divisionPicker.open = false">닫기</button>
            </div>
          </div>
        </div>

        <!-- 수정 모달 -->
        <div v-if="editModal.open" class="modal-backdrop" @click.self="editModal.open = false">
          <div class="modal">
            <div class="modal-title">수행평가 수정</div>
            <label>제목</label>
            <input v-model="form.title" type="text" />
            <label>설명</label>
            <textarea v-model="form.description"></textarea>
            <div v-if="saveError" class="save-error">{{ saveError }}</div>
            <div class="modal-actions">
              <button @click="editModal.open = false">취소</button>
              <button class="btn-primary" @click="submitUpdate" :disabled="saving">저장</button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useAssessmentStore } from '@/stores/assessment'
import { useProblemStore } from '@/stores/problem'
import { useDivisionStore } from '@/stores/division'
import {
  IconSchool, IconLayoutDashboard, IconDatabase, IconList, IconFileText,
  IconUsers, IconPlus, IconPencil, IconTrash, IconX, IconLock, IconAlertTriangle,
} from '@tabler/icons-vue'

const router = useRouter()
const auth = useAuthStore()
const store = useAssessmentStore()
const problemStore = useProblemStore()
const divisionStore = useDivisionStore()

const selectedId = ref<number | null>(null)
const mode = ref<'idle' | 'create' | 'view'>('idle')
const saving = ref(false)
const saveError = ref<string | null>(null)

const form = reactive({ title: '', description: '' })

const probPicker = reactive({ open: false, search: '' })
const pickerItems = ref<Array<{ problem_id: number; score: number }>>([])

const divisionPicker = reactive({ open: false })
const editModal = reactive({ open: false })

const totalScore = computed(() => store.currentAssessment?.problems.reduce((s, p) => s + p.score, 0) ?? 0)

const filteredAllProblems = computed(() =>
  problemStore.problems.filter(p =>
    !probPicker.search || p.title.toLowerCase().includes(probPicker.search.toLowerCase())
  )
)

const availableDivisions = computed(() => divisionStore.divisions)

function typeShort(t: number): string {
  return ({ 1: '①실행', 2: '②코드', 3: '③과제', 4: '④빈칸' } as Record<number, string>)[t] ?? `유형${t}`
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString('ko-KR', { month: 'short', day: 'numeric' })
}

function openCreate() {
  form.title = ''; form.description = ''
  mode.value = 'create'; saveError.value = null
}

async function submitCreate() {
  if (!form.title.trim()) { saveError.value = '제목을 입력하세요'; return }
  saving.value = true; saveError.value = null
  try {
    const row = await store.createAssessment({ title: form.title.trim(), description: form.description })
    await selectAssessment(row.id)
  } catch (e) {
    saveError.value = e instanceof Error ? e.message : '오류가 발생했습니다'
  } finally {
    saving.value = false
  }
}

async function selectAssessment(id: number) {
  selectedId.value = id; mode.value = 'view'; saveError.value = null
  await store.fetchAssessment(id)
}

function startEdit() {
  form.title = store.currentAssessment?.title ?? ''
  form.description = store.currentAssessment?.description ?? ''
  editModal.open = true; saveError.value = null
}

async function submitUpdate() {
  saving.value = true; saveError.value = null
  try {
    await store.updateAssessment(selectedId.value!, { title: form.title.trim(), description: form.description })
    editModal.open = false
  } catch (e) {
    saveError.value = e instanceof Error ? e.message : '오류가 발생했습니다'
  } finally {
    saving.value = false
  }
}

async function confirmDelete() {
  if (!confirm(`"${store.currentAssessment?.title}" 수행평가를 삭제하시겠습니까?`)) return
  try {
    await store.deleteAssessment(selectedId.value!)
    selectedId.value = null; mode.value = 'idle'
  } catch (e) { alert(e instanceof Error ? e.message : '삭제 실패') }
}

function openProblemPicker() {
  pickerItems.value = store.currentAssessment?.problems.map(p => ({ problem_id: p.problem_id, score: p.score })) ?? []
  probPicker.search = ''; probPicker.open = true
}

function togglePicker(id: number) {
  const idx = pickerItems.value.findIndex(i => i.problem_id === id)
  if (idx >= 0) pickerItems.value.splice(idx, 1)
  else pickerItems.value.push({ problem_id: id, score: 10 })
}

function setScore(problemId: number, score: number) {
  const item = pickerItems.value.find(i => i.problem_id === problemId)
  if (item) item.score = score
}

async function submitProblems() {
  try {
    await store.setProblems(selectedId.value!, pickerItems.value)
    probPicker.open = false
  } catch (e) { alert(e instanceof Error ? e.message : '저장 실패') }
}

function openDivisionLink() { divisionPicker.open = true }

async function toggleDivision(divisionId: number) {
  const linked = store.currentAssessment?.divisions.some(d => d.division_id === divisionId)
  try {
    if (linked) await store.unlinkDivision(selectedId.value!, divisionId)
    else await store.linkDivision(selectedId.value!, divisionId)
  } catch (e) { alert(e instanceof Error ? e.message : '오류') }
}

async function unlinkDivision(divisionId: number) {
  if (!confirm('분반 연결을 해제하시겠습니까?')) return
  try { await store.unlinkDivision(selectedId.value!, divisionId) }
  catch (e) { alert(e instanceof Error ? e.message : '오류') }
}

async function logout() {
  await auth.logoutTeacher(); router.replace({ name: 'login' })
}

onMounted(async () => {
  if (!auth.isTeacherLoggedIn) {
    await auth.fetchTeacherMe()
    if (!auth.isTeacherLoggedIn) { router.replace({ name: 'login' }); return }
  }
  await Promise.all([
    store.fetchAssessments(),
    problemStore.fetchProblems(),
    divisionStore.fetchDivisions(),
  ])
})
</script>

<style scoped>
.layout { display: flex; height: 100vh; overflow: hidden; }

.sidebar {
  width: 200px;
  flex-shrink: 0;
  background: var(--color-background-primary);
  border-right: 1px solid var(--color-border-secondary);
  display: flex;
  flex-direction: column;
}

.sidebar-logo {
  padding: 14px 16px;
  font-weight: 700;
  font-size: 14px;
  border-bottom: 1px solid var(--color-border-secondary);
  display: flex;
  align-items: center;
  gap: 8px;
}

.sidebar-nav { flex: 1; padding: 8px 0; }
.nav-group-label {
  font-size: 10px; font-weight: 600; color: var(--color-text-tertiary);
  text-transform: uppercase; letter-spacing: 0.05em; padding: 12px 16px 4px;
}

.nav-item {
  display: flex; align-items: center; gap: 8px; padding: 7px 10px 7px 16px;
  font-size: 13px; color: var(--color-text-secondary); text-decoration: none;
  margin: 0 6px; border-radius: var(--border-radius-md); transition: background 0.1s;
}
.nav-item:hover { background: var(--color-background-secondary); color: var(--color-text-primary); }
.nav-item.active { background: var(--color-background-info); color: var(--color-text-info); font-weight: 500; }

.sidebar-footer { padding: 12px 16px; border-top: 1px solid var(--color-border-secondary); display: flex; flex-direction: column; gap: 8px; }
.user-info { display: flex; align-items: center; gap: 8px; }
.user-name { font-size: 13px; font-weight: 500; }
.user-role { font-size: 10px; background: var(--color-background-info); color: var(--color-text-info); padding: 2px 6px; border-radius: 4px; }
.logout-btn { font-size: 12px; color: var(--color-text-secondary); padding: 4px 0; border: none; background: none; cursor: pointer; text-align: left; }

.main-split { flex: 1; display: flex; overflow: hidden; }

.list-panel {
  width: 230px; flex-shrink: 0;
  border-right: 1px solid var(--color-border-secondary);
  background: var(--color-background-primary);
  display: flex; flex-direction: column; overflow-y: auto;
}

.panel-header { display: flex; justify-content: space-between; align-items: center; padding: 14px 16px 10px; border-bottom: 1px solid var(--color-border-secondary); }
.panel-title { font-weight: 600; font-size: 13px; }

.btn-icon { width: 26px; height: 26px; padding: 0; border: 1px solid var(--color-border-primary); border-radius: var(--border-radius-sm); background: var(--color-background-secondary); cursor: pointer; display: flex; align-items: center; justify-content: center; }

.panel-loading, .panel-error, .panel-empty { padding: 16px; font-size: 12px; color: var(--color-text-tertiary); display: flex; align-items: center; gap: 6px; }
.panel-error { color: var(--color-text-danger); }

.assessment-item {
  display: flex; flex-direction: column; gap: 3px; padding: 10px 14px;
  text-align: left; border: none; border-bottom: 1px solid var(--color-border-tertiary);
  background: none; cursor: pointer; transition: background 0.1s; width: 100%;
}
.assessment-item:hover { background: var(--color-background-secondary); }
.assessment-item.active { background: var(--color-background-info); }
.assessment-item.locked { border-left: 3px solid var(--color-text-danger); }

.assessment-item-top { display: flex; align-items: center; justify-content: space-between; gap: 6px; }
.assessment-title { font-size: 13px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

.lock-badge {
  display: inline-flex; align-items: center; gap: 3px;
  font-size: 10px; font-weight: 600;
  color: var(--color-text-danger); background: var(--color-background-danger);
  padding: 2px 5px; border-radius: 3px; flex-shrink: 0;
}

.lock-badge-large {
  display: inline-flex; align-items: center; gap: 5px;
  font-size: 12px; font-weight: 600;
  color: var(--color-text-danger); background: var(--color-background-danger);
  padding: 3px 8px; border-radius: var(--border-radius-sm);
}

.assessment-meta { font-size: 11px; color: var(--color-text-secondary); }
.assessment-date { font-size: 10px; color: var(--color-text-tertiary); }

/* ── 상세 패널 ── */
.detail-panel { flex: 1; overflow-y: auto; background: var(--color-background-secondary); padding: 16px; display: flex; flex-direction: column; gap: 12px; }

.detail-empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: var(--color-text-tertiary); font-size: 13px; }
.empty-icon { color: var(--color-text-tertiary); }

.editor-wrap { display: flex; flex-direction: column; gap: 12px; }
.editor-header { display: flex; justify-content: space-between; align-items: center; }
.editor-title { display: flex; align-items: center; gap: 8px; font-weight: 600; font-size: 15px; }
.editor-header-actions { display: flex; gap: 8px; }

.form-card { background: var(--color-background-primary); border: 1px solid var(--color-border-secondary); border-radius: var(--border-radius-lg); padding: 16px; display: flex; flex-direction: column; gap: 12px; }
.field { display: flex; flex-direction: column; gap: 5px; }
.field label { font-size: 12px; font-weight: 600; color: var(--color-text-secondary); }
.field textarea { width: 100%; min-height: 70px; padding: 6px 8px; border: 1px solid var(--color-border-primary); border-radius: var(--border-radius-sm); font-size: 13px; font-family: inherit; resize: vertical; }

.save-error { font-size: 12px; color: var(--color-text-danger); background: var(--color-background-danger); padding: 8px 10px; border-radius: var(--border-radius-sm); }
.form-actions { display: flex; justify-content: flex-end; gap: 8px; }

.btn-primary { background: var(--color-accent); color: #fff; border-color: transparent; padding: 7px 14px; font-size: 13px; }
.btn-primary:hover:not(:disabled) { background: var(--color-accent-hover); }

.btn-secondary { display: inline-flex; align-items: center; gap: 4px; font-size: 12px; padding: 5px 10px; }

.btn-icon-sm { width: 24px; height: 24px; padding: 0; border: 1px solid var(--color-border-secondary); border-radius: var(--border-radius-sm); background: none; cursor: pointer; display: flex; align-items: center; justify-content: center; color: var(--color-text-secondary); }
.btn-icon-sm.btn-danger { color: var(--color-text-danger); }
.btn-icon-sm.btn-danger:hover { background: var(--color-background-danger); }

.detail-body { display: flex; flex-direction: column; gap: 14px; }
.detail-section { background: var(--color-background-primary); border: 1px solid var(--color-border-secondary); border-radius: var(--border-radius-lg); padding: 14px 16px; display: flex; flex-direction: column; gap: 10px; }
.section-header { display: flex; justify-content: space-between; align-items: center; }
.section-title { font-weight: 500; font-size: 13px; }
.section-right { display: flex; align-items: center; gap: 10px; }
.total-score { font-size: 12px; font-weight: 600; color: var(--color-text-info); }

.empty-note { font-size: 12px; color: var(--color-text-tertiary); }

.problem-list { display: flex; flex-direction: column; gap: 6px; }
.problem-row { display: flex; align-items: center; gap: 8px; font-size: 13px; }
.prob-order { font-size: 11px; font-weight: 700; color: var(--color-text-tertiary); min-width: 16px; }
.prob-title { flex: 1; color: var(--color-text-primary); }
.prob-score { font-size: 12px; font-weight: 600; color: var(--color-text-info); }

.type-badge { font-size: 10px; font-weight: 600; padding: 1px 5px; border-radius: 3px; color: #fff; white-space: nowrap; }
.type-1 { background: #7c3aed; }
.type-2 { background: #0369a1; }
.type-3 { background: #065f46; }
.type-4 { background: #92400e; }

.division-list { display: flex; flex-direction: column; gap: 6px; }
.division-row { display: flex; align-items: center; gap: 10px; font-size: 13px; }
.division-name { font-weight: 500; flex: 1; }

.running-badge { display: inline-flex; align-items: center; gap: 4px; font-size: 11px; font-weight: 600; color: var(--color-text-danger); background: var(--color-background-danger); padding: 2px 7px; border-radius: 3px; }

/* ── 모달 ── */
.modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.35); display: flex; align-items: center; justify-content: center; z-index: 100; }
.modal { background: var(--color-background-primary); border-radius: var(--border-radius-lg); padding: 20px; width: 380px; display: flex; flex-direction: column; gap: 10px; box-shadow: 0 8px 32px rgba(0,0,0,0.15); max-height: 80vh; }
.modal-wide { width: 560px; }
.modal-title { font-weight: 600; font-size: 15px; }
.modal-hint { font-size: 12px; color: var(--color-text-secondary); margin: 0; }
.modal-actions { display: flex; justify-content: flex-end; gap: 8px; }

.picker-list { overflow-y: auto; max-height: 320px; border: 1px solid var(--color-border-secondary); border-radius: var(--border-radius-md); display: flex; flex-direction: column; }
.picker-item { display: flex; align-items: center; gap: 8px; padding: 8px 12px; cursor: pointer; font-size: 13px; border-bottom: 1px solid var(--color-border-tertiary); transition: background 0.1s; }
.picker-item:last-child { border-bottom: none; }
.picker-item:hover { background: var(--color-background-secondary); }
.picker-item.checked { background: var(--color-background-info); }
.flex-grow { flex: 1; }

.score-input { width: 54px; padding: 3px 6px; border: 1px solid var(--color-border-primary); border-radius: var(--border-radius-sm); font-size: 12px; text-align: right; }

.div-count { font-size: 11px; color: var(--color-text-tertiary); margin-left: auto; }

.spinner { width: 14px; height: 14px; border: 2px solid var(--color-border-primary); border-top-color: var(--color-accent); border-radius: 50%; animation: spin 0.7s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>
