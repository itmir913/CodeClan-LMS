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
        <RouterLink :to="{ name: 'problem-bank' }" class="nav-item">
          <IconDatabase :size="16" />문제 은행
        </RouterLink>
        <a href="#" class="nav-item active"><IconList :size="16" />차시 관리</a>
        <a href="#" class="nav-item"><IconFileText :size="16" />수행평가</a>
        <div class="nav-group-label">관리</div>
        <RouterLink :to="{ name: 'division-management' }" class="nav-item">
          <IconUsers :size="16" />학생/반 관리
        </RouterLink>
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
      <!-- 좌: 차시 목록 -->
      <div class="list-panel">
        <div class="panel-header">
          <span class="panel-title">차시 목록</span>
          <button class="btn-icon" @click="openCreate" title="차시 추가">
            <IconPlus :size="15" />
          </button>
        </div>

        <div v-if="store.loading && store.lessons.length === 0" class="panel-loading">
          <div class="spinner"></div>
        </div>
        <div v-else-if="store.error" class="panel-error">{{ store.error }}</div>
        <div v-else-if="store.lessons.length === 0" class="panel-empty">차시가 없습니다</div>

        <button
          v-for="l in store.lessons"
          :key="l.id"
          class="lesson-item"
          :class="{ active: selectedId === l.id }"
          @click="selectLesson(l.id)"
        >
          <div class="lesson-order">{{ l.order_no }}차시</div>
          <div class="lesson-title">{{ l.title }}</div>
          <div class="lesson-meta">문항 {{ l.problem_count }}개</div>
        </button>
      </div>

      <!-- 우: 상세 / 편집 -->
      <div class="detail-panel">
        <!-- 새 차시 -->
        <div v-if="mode === 'create'" class="editor-wrap">
          <div class="editor-header">
            <span class="editor-title">새 차시</span>
            <button class="btn-icon-sm" @click="mode = 'idle'"><IconX :size="14" /></button>
          </div>
          <div class="form-card">
            <div class="field">
              <label>차시 번호</label>
              <input v-model.number="form.orderNo" type="number" min="1" placeholder="1" />
            </div>
            <div class="field">
              <label>제목 *</label>
              <input v-model="form.title" type="text" placeholder="차시 제목" @keyup.enter="submitCreate" />
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

        <!-- 차시 상세 -->
        <template v-else-if="mode === 'view' && store.currentLesson">
          <div class="editor-header">
            <div class="editor-title">
              <span class="order-chip">{{ store.currentLesson.order_no }}차시</span>
              <span v-if="!editingTitle">{{ store.currentLesson.title }}</span>
              <input v-else v-model="form.title" class="inline-title-input" @keyup.enter="submitUpdateTitle" @keyup.escape="editingTitle = false" />
            </div>
            <div class="editor-header-actions">
              <button class="btn-secondary" @click="startEditTitle"><IconPencil :size="13" />제목 수정</button>
              <button class="btn-icon-sm btn-danger" @click="confirmDelete"><IconTrash :size="13" /></button>
            </div>
          </div>

          <div class="detail-body">
            <!-- 문항 배정 -->
            <section class="detail-section">
              <div class="section-header">
                <span class="section-title">배정 문항</span>
                <button class="btn-secondary" @click="openProblemPicker">
                  <IconPlus :size="13" />문항 배정 편집
                </button>
              </div>
              <div v-if="store.currentLesson.problems.length === 0" class="empty-note">
                배정된 문항이 없습니다
              </div>
              <div v-else class="problem-list">
                <div v-for="(p, i) in store.currentLesson.problems" :key="p.id" class="problem-row">
                  <span class="prob-order">{{ i + 1 }}</span>
                  <span class="type-badge" :class="`type-${p.problem_type}`">{{ typeShort(p.problem_type) }}</span>
                  <span class="prob-title">{{ p.problem_title }}</span>
                </div>
              </div>
            </section>

            <!-- 분반별 공개 상태 -->
            <section class="detail-section">
              <div class="section-header">
                <span class="section-title">분반별 공개 상태</span>
              </div>
              <div v-if="store.currentLesson.releases.length === 0" class="empty-note">
                분반 정보가 없습니다
              </div>
              <div v-else class="release-list">
                <div v-for="rel in store.currentLesson.releases" :key="rel.division_id" class="release-row">
                  <span class="division-name">{{ rel.division_name }}</span>
                  <span class="release-date" v-if="rel.is_released && rel.released_at">
                    {{ formatDate(rel.released_at) }} 공개
                  </span>
                  <button
                    class="toggle-btn"
                    :class="{ released: rel.is_released }"
                    @click="toggleRelease(rel.division_id, !rel.is_released)"
                  >
                    {{ rel.is_released ? '공개 중 (클릭해 비공개)' : '비공개 (클릭해 공개)' }}
                  </button>
                </div>
              </div>
            </section>
          </div>
        </template>

        <!-- 빈 상태 -->
        <div v-else class="detail-empty">
          <IconList :size="32" class="empty-icon" />
          <div>차시를 선택하거나 새 차시를 추가하세요</div>
        </div>

        <!-- 문항 피커 모달 -->
        <div v-if="problemPicker.open" class="modal-backdrop" @click.self="problemPicker.open = false">
          <div class="modal modal-wide">
            <div class="modal-title">문항 배정 편집</div>
            <p class="modal-hint">체크박스를 선택하여 배정할 문항을 고르세요. 순서는 체크 순서를 따릅니다.</p>
            <input v-model="problemPicker.search" type="text" placeholder="문항 검색" />
            <div class="picker-list">
              <label
                v-for="p in filteredAllProblems"
                :key="p.id"
                class="picker-item"
                :class="{ checked: pickerSelected.includes(p.id) }"
              >
                <input
                  type="checkbox"
                  :value="p.id"
                  :checked="pickerSelected.includes(p.id)"
                  @change="togglePicker(p.id)"
                />
                <span class="type-badge" :class="`type-${p.problem_type}`">{{ typeShort(p.problem_type) }}</span>
                {{ p.title }}
              </label>
            </div>
            <div class="modal-actions">
              <button @click="problemPicker.open = false">취소</button>
              <button class="btn-primary" @click="submitProblems">저장</button>
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
import { useLessonStore } from '@/stores/lesson'
import { useProblemStore } from '@/stores/problem'
import {
  IconSchool, IconLayoutDashboard, IconDatabase, IconList, IconFileText,
  IconUsers, IconPlus, IconPencil, IconTrash, IconX,
} from '@tabler/icons-vue'

const router = useRouter()
const auth = useAuthStore()
const store = useLessonStore()
const problemStore = useProblemStore()

const selectedId = ref<number | null>(null)
const mode = ref<'idle' | 'create' | 'view'>('idle')
const saving = ref(false)
const saveError = ref<string | null>(null)
const editingTitle = ref(false)

const form = reactive({ title: '', description: '', orderNo: 1 })

const problemPicker = reactive({ open: false, search: '' })
const pickerSelected = ref<number[]>([])

const filteredAllProblems = computed(() =>
  problemStore.problems.filter(p =>
    !problemPicker.search || p.title.toLowerCase().includes(problemPicker.search.toLowerCase())
  )
)

function typeShort(t: number): string {
  return ({ 1: '①실행', 2: '②코드', 3: '③과제', 4: '④빈칸' } as Record<number, string>)[t] ?? `유형${t}`
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString('ko-KR', { month: 'short', day: 'numeric' })
}

function openCreate() {
  form.title = ''; form.description = ''
  form.orderNo = (store.lessons[store.lessons.length - 1]?.order_no ?? 0) + 1
  mode.value = 'create'
  saveError.value = null
}

async function submitCreate() {
  if (!form.title.trim()) { saveError.value = '차시 제목을 입력하세요'; return }
  saving.value = true; saveError.value = null
  try {
    const row = await store.createLesson({ title: form.title.trim(), description: form.description, order_no: form.orderNo })
    await selectLesson(row.id)
  } catch (e) {
    saveError.value = e instanceof Error ? e.message : '오류가 발생했습니다'
  } finally {
    saving.value = false
  }
}

async function selectLesson(id: number) {
  selectedId.value = id
  mode.value = 'view'
  editingTitle.value = false
  await store.fetchLesson(id)
}

function startEditTitle() {
  form.title = store.currentLesson?.title ?? ''
  editingTitle.value = true
}

async function submitUpdateTitle() {
  if (!form.title.trim()) { editingTitle.value = false; return }
  try {
    await store.updateLesson(selectedId.value!, { title: form.title.trim() })
  } catch (e) {
    alert(e instanceof Error ? e.message : '오류')
  } finally {
    editingTitle.value = false
  }
}

async function confirmDelete() {
  if (!confirm(`"${store.currentLesson?.title}" 차시를 삭제하시겠습니까?`)) return
  try {
    await store.deleteLesson(selectedId.value!)
    selectedId.value = null; mode.value = 'idle'
  } catch (e) {
    alert(e instanceof Error ? e.message : '삭제 실패')
  }
}

function openProblemPicker() {
  pickerSelected.value = store.currentLesson?.problems.map(p => p.problem_id) ?? []
  problemPicker.search = ''
  problemPicker.open = true
}

function togglePicker(id: number) {
  const idx = pickerSelected.value.indexOf(id)
  if (idx >= 0) pickerSelected.value.splice(idx, 1)
  else pickerSelected.value.push(id)
}

async function submitProblems() {
  try {
    await store.setProblems(selectedId.value!, pickerSelected.value)
    problemPicker.open = false
  } catch (e) {
    alert(e instanceof Error ? e.message : '저장 실패')
  }
}

async function toggleRelease(divisionId: number, isReleased: boolean) {
  try {
    await store.toggleRelease(selectedId.value!, divisionId, isReleased)
  } catch (e) {
    alert(e instanceof Error ? e.message : '오류')
  }
}

async function logout() {
  await auth.logoutTeacher()
  router.replace({ name: 'login' })
}

onMounted(async () => {
  if (!auth.isTeacherLoggedIn) {
    await auth.fetchTeacherMe()
    if (!auth.isTeacherLoggedIn) { router.replace({ name: 'login' }); return }
  }
  await Promise.all([store.fetchLessons(), problemStore.fetchProblems()])
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
  font-size: 10px;
  font-weight: 600;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 12px 16px 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px 7px 16px;
  font-size: 13px;
  color: var(--color-text-secondary);
  text-decoration: none;
  margin: 0 6px;
  border-radius: var(--border-radius-md);
  transition: background 0.1s;
}

.nav-item:hover { background: var(--color-background-secondary); color: var(--color-text-primary); }
.nav-item.active { background: var(--color-background-info); color: var(--color-text-info); font-weight: 500; }

.sidebar-footer {
  padding: 12px 16px;
  border-top: 1px solid var(--color-border-secondary);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.user-info { display: flex; align-items: center; gap: 8px; }
.user-name { font-size: 13px; font-weight: 500; }
.user-role { font-size: 10px; background: var(--color-background-info); color: var(--color-text-info); padding: 2px 6px; border-radius: 4px; }
.logout-btn { font-size: 12px; color: var(--color-text-secondary); padding: 4px 0; border: none; background: none; cursor: pointer; text-align: left; }

.main-split { flex: 1; display: flex; overflow: hidden; }

.list-panel {
  width: 220px;
  flex-shrink: 0;
  border-right: 1px solid var(--color-border-secondary);
  background: var(--color-background-primary);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 16px 10px;
  border-bottom: 1px solid var(--color-border-secondary);
}

.panel-title { font-weight: 600; font-size: 13px; }

.btn-icon {
  width: 26px;
  height: 26px;
  padding: 0;
  border: 1px solid var(--color-border-primary);
  border-radius: var(--border-radius-sm);
  background: var(--color-background-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.panel-loading, .panel-error, .panel-empty {
  padding: 16px;
  font-size: 12px;
  color: var(--color-text-tertiary);
  display: flex;
  align-items: center;
  gap: 6px;
}

.panel-error { color: var(--color-text-danger); }

.lesson-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 10px 14px;
  text-align: left;
  border: none;
  border-bottom: 1px solid var(--color-border-tertiary);
  background: none;
  cursor: pointer;
  transition: background 0.1s;
  width: 100%;
}

.lesson-item:hover { background: var(--color-background-secondary); }
.lesson-item.active { background: var(--color-background-info); }

.lesson-order { font-size: 10px; font-weight: 700; color: var(--color-text-info); }
.lesson-title { font-size: 13px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.lesson-meta { font-size: 11px; color: var(--color-text-tertiary); }

/* ── 상세 패널 ── */
.detail-panel {
  flex: 1;
  overflow-y: auto;
  background: var(--color-background-secondary);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.detail-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--color-text-tertiary);
  font-size: 13px;
}

.empty-icon { color: var(--color-text-tertiary); }

.editor-wrap { display: flex; flex-direction: column; gap: 12px; }

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.editor-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  font-size: 15px;
}

.order-chip {
  font-size: 11px;
  font-weight: 700;
  background: var(--color-background-info);
  color: var(--color-text-info);
  padding: 2px 8px;
  border-radius: 4px;
}

.inline-title-input {
  font-size: 15px;
  font-weight: 600;
  border: 1px solid var(--color-accent);
  border-radius: var(--border-radius-sm);
  padding: 2px 6px;
}

.editor-header-actions { display: flex; gap: 8px; }

.btn-secondary {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  padding: 5px 10px;
}

.btn-icon-sm {
  width: 24px;
  height: 24px;
  padding: 0;
  border: 1px solid var(--color-border-secondary);
  border-radius: var(--border-radius-sm);
  background: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
}

.btn-icon-sm.btn-danger { color: var(--color-text-danger); }
.btn-icon-sm.btn-danger:hover { background: var(--color-background-danger); }

.form-card {
  background: var(--color-background-primary);
  border: 1px solid var(--color-border-secondary);
  border-radius: var(--border-radius-lg);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.field { display: flex; flex-direction: column; gap: 5px; }
.field label { font-size: 12px; font-weight: 600; color: var(--color-text-secondary); }
.field textarea { width: 100%; min-height: 70px; padding: 6px 8px; border: 1px solid var(--color-border-primary); border-radius: var(--border-radius-sm); font-size: 13px; font-family: inherit; resize: vertical; }

.save-error {
  font-size: 12px;
  color: var(--color-text-danger);
  background: var(--color-background-danger);
  padding: 8px 10px;
  border-radius: var(--border-radius-sm);
}

.form-actions { display: flex; justify-content: flex-end; gap: 8px; }

.btn-primary {
  background: var(--color-accent);
  color: #fff;
  border-color: transparent;
  padding: 7px 14px;
  font-size: 13px;
}
.btn-primary:hover:not(:disabled) { background: var(--color-accent-hover); }

.detail-body { display: flex; flex-direction: column; gap: 14px; }

.detail-section {
  background: var(--color-background-primary);
  border: 1px solid var(--color-border-secondary);
  border-radius: var(--border-radius-lg);
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.section-header { display: flex; justify-content: space-between; align-items: center; }
.section-title { font-weight: 500; font-size: 13px; }

.empty-note { font-size: 12px; color: var(--color-text-tertiary); }

.problem-list { display: flex; flex-direction: column; gap: 6px; }
.problem-row { display: flex; align-items: center; gap: 8px; font-size: 13px; }
.prob-order { font-size: 11px; font-weight: 700; color: var(--color-text-tertiary); min-width: 16px; }
.prob-title { color: var(--color-text-primary); }

.type-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 1px 5px;
  border-radius: 3px;
  color: #fff;
  white-space: nowrap;
}

.type-1 { background: #7c3aed; }
.type-2 { background: #0369a1; }
.type-3 { background: #065f46; }
.type-4 { background: #92400e; }

.release-list { display: flex; flex-direction: column; gap: 8px; }
.release-row { display: flex; align-items: center; gap: 10px; font-size: 13px; }
.division-name { font-weight: 500; min-width: 100px; }
.release-date { font-size: 11px; color: var(--color-text-tertiary); }

.toggle-btn {
  font-size: 11px;
  padding: 3px 10px;
  border-radius: 20px;
  border: 1px solid var(--color-border-secondary);
  background: var(--color-background-secondary);
  color: var(--color-text-secondary);
  cursor: pointer;
  margin-left: auto;
  transition: all 0.1s;
}

.toggle-btn.released {
  background: var(--color-background-success);
  color: var(--color-text-success);
  border-color: var(--color-border-success);
}

/* ── 모달 ── */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  background: var(--color-background-primary);
  border-radius: var(--border-radius-lg);
  padding: 20px;
  width: 400px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  max-height: 80vh;
}

.modal-wide { width: 520px; }
.modal-title { font-weight: 600; font-size: 15px; }
.modal-hint { font-size: 12px; color: var(--color-text-secondary); margin: 0; }
.modal-actions { display: flex; justify-content: flex-end; gap: 8px; }

.picker-list {
  overflow-y: auto;
  max-height: 300px;
  border: 1px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md);
  display: flex;
  flex-direction: column;
}

.picker-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  font-size: 13px;
  border-bottom: 1px solid var(--color-border-tertiary);
  transition: background 0.1s;
}

.picker-item:last-child { border-bottom: none; }
.picker-item:hover { background: var(--color-background-secondary); }
.picker-item.checked { background: var(--color-background-info); }

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--color-border-primary);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }
</style>
