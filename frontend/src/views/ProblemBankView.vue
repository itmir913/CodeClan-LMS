<template>
  <div class="layout">
    <AppSidebar />

    <div class="main-split">
      <!-- 좌: 문제 목록 -->
      <div class="list-panel">
        <div class="panel-header">
          <span class="panel-title">문제 은행</span>
          <button class="btn-icon" @click="openCreate" title="문제 추가">
            <IconPlus :size="15" />
          </button>
        </div>

        <!-- 필터 -->
        <div class="filter-bar">
          <input v-model="searchQuery" type="text" placeholder="제목 검색" class="search-input" @input="onSearch" />
          <select v-model="typeFilter" @change="onSearch">
            <option value="">전체 유형</option>
            <option v-for="(label, type) in PROBLEM_TYPE_LABELS" :key="type" :value="type">{{ label }}</option>
          </select>
        </div>

        <div v-if="store.loading && store.problems.length === 0" class="panel-loading">
          <div class="spinner"></div>
        </div>
        <div v-else-if="store.error" class="panel-error">{{ store.error }}</div>
        <div v-else-if="store.problems.length === 0" class="panel-empty">문제가 없습니다</div>

        <button
          v-for="p in store.problems"
          :key="p.id"
          class="problem-item"
          :class="{ active: selectedId === p.id }"
          @click="selectProblem(p.id)"
        >
          <div class="problem-item-top">
            <span class="type-badge" :class="`type-${p.problem_type}`">{{ typeShort(p.problem_type) }}</span>
            <span v-if="p.is_structure_check" class="struct-badge">⑤구조</span>
          </div>
          <div class="problem-item-title">{{ p.title }}</div>
          <div class="problem-item-date">{{ formatDate(p.created_at) }}</div>
        </button>
      </div>

      <!-- 우: 상세/편집 -->
      <div class="detail-panel">
        <!-- 새 문제 폼 -->
        <div v-if="mode === 'create'" class="editor-wrap">
          <div class="editor-header">
            <span class="editor-title">새 문제</span>
            <button class="btn-icon-sm" @click="mode = 'idle'" title="닫기"><IconX :size="14" /></button>
          </div>
          <ProblemEditor
            :initial="null"
            :saving="saving"
            :save-error="saveError"
            @save="handleCreate"
            @cancel="mode = 'idle'"
          />
        </div>

        <!-- 문제 상세 / 편집 -->
        <div v-else-if="mode === 'view' || mode === 'edit'" class="editor-wrap">
          <div v-if="store.loading" class="detail-loading">
            <div class="spinner"></div>불러오는 중...
          </div>
          <div v-else-if="store.currentProblem">
            <div class="editor-header">
              <span class="editor-title">
                <span class="type-badge" :class="`type-${store.currentProblem.problem_type}`">
                  {{ typeShort(store.currentProblem.problem_type) }}
                </span>
                {{ store.currentProblem.title }}
              </span>
              <div class="editor-header-actions">
                <button v-if="mode === 'view'" class="btn-secondary" @click="mode = 'edit'">
                  <IconPencil :size="13" />편집
                </button>
                <button class="btn-icon-sm btn-danger" @click="confirmDelete" title="삭제">
                  <IconTrash :size="13" />
                </button>
              </div>
            </div>

            <ProblemEditor
              v-if="mode === 'edit'"
              :initial="store.currentProblem"
              :saving="saving"
              :save-error="saveError"
              @save="handleUpdate"
              @cancel="mode = 'view'"
            />

            <!-- 읽기 전용 뷰 -->
            <div v-else class="problem-view">
              <div class="view-section" v-if="store.currentProblem.description">
                <div class="view-label">설명</div>
                <div class="view-content pre-wrap">{{ store.currentProblem.description }}</div>
              </div>
              <div class="view-section">
                <div class="view-label">type_config (JSON)</div>
                <pre class="json-view">{{ formatJson(store.currentProblem.type_config) }}</pre>
              </div>
              <div class="view-section" v-if="store.currentProblem.is_structure_check">
                <div class="view-label">구조검사</div>
                <span class="struct-badge-large">⑤구조검사 게이트 활성화</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 빈 상태 -->
        <div v-else class="detail-empty">
          <IconDatabase :size="32" class="empty-icon" />
          <div>문제를 선택하거나 새 문제를 추가하세요</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useProblemStore } from '@/stores/problem'
import { PROBLEM_TYPE_LABELS, type ProblemRow, type CreateProblemInput, type UpdateProblemInput } from '@/api/client'
import ProblemEditor from '@/components/ProblemEditor.vue'
import {
  IconDatabase, IconPlus, IconPencil, IconTrash, IconX,
} from '@tabler/icons-vue'
import AppSidebar from '@/components/AppSidebar.vue'

const router = useRouter()
const auth = useAuthStore()
const store = useProblemStore()

const selectedId = ref<number | null>(null)
const mode = ref<'idle' | 'create' | 'view' | 'edit'>('idle')
const saving = ref(false)
const saveError = ref<string | null>(null)
const searchQuery = ref('')
const typeFilter = ref<string>('')

function typeShort(t: number): string {
  return ({ 1: '①실행', 2: '②코드', 3: '③과제', 4: '④빈칸' } as Record<number, string>)[t] ?? `유형${t}`
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString('ko-KR', { month: 'short', day: 'numeric' })
}

function formatJson(s: string): string {
  try { return JSON.stringify(JSON.parse(s), null, 2) } catch { return s }
}

let searchTimer: ReturnType<typeof setTimeout> | null = null
function onSearch() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    store.fetchProblems({
      q: searchQuery.value || undefined,
      problem_type: typeFilter.value ? Number(typeFilter.value) : undefined,
    })
  }, 300)
}

async function selectProblem(id: number) {
  selectedId.value = id
  mode.value = 'view'
  saveError.value = null
  await store.fetchProblem(id)
}

function openCreate() {
  selectedId.value = null
  mode.value = 'create'
  saveError.value = null
}

async function handleCreate(data: CreateProblemInput) {
  saving.value = true
  saveError.value = null
  try {
    const row = await store.createProblem(data)
    selectedId.value = row.id
    await store.fetchProblem(row.id)
    mode.value = 'view'
  } catch (e) {
    saveError.value = e instanceof Error ? e.message : '저장 실패'
  } finally {
    saving.value = false
  }
}

async function handleUpdate(data: UpdateProblemInput) {
  saving.value = true
  saveError.value = null
  try {
    await store.updateProblem(selectedId.value!, data)
    await store.fetchProblem(selectedId.value!)
    mode.value = 'view'
  } catch (e) {
    saveError.value = e instanceof Error ? e.message : '저장 실패'
  } finally {
    saving.value = false
  }
}

async function confirmDelete() {
  if (!store.currentProblem) return
  if (!confirm(`"${store.currentProblem.title}" 문제를 삭제하시겠습니까?`)) return
  try {
    await store.deleteProblem(store.currentProblem.id)
    selectedId.value = null
    mode.value = 'idle'
  } catch (e) {
    alert(e instanceof Error ? e.message : '삭제 실패')
  }
}


onMounted(async () => {
  if (!auth.isTeacherLoggedIn) {
    await auth.fetchTeacherMe()
    if (!auth.isTeacherLoggedIn) { router.replace({ name: 'login' }); return }
  }
  await store.fetchProblems()
})
</script>

<style scoped>
.layout { display: flex; height: 100vh; overflow: hidden; }

/* ── 2단 레이아웃 ── */
.main-split { flex: 1; display: flex; overflow: hidden; }

.list-panel {
  width: 240px;
  flex-shrink: 0;
  border-right: 1px solid var(--color-border-secondary);
  background: var(--color-background-primary);
  display: flex;
  flex-direction: column;
  overflow: hidden;
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

.filter-bar {
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  border-bottom: 1px solid var(--color-border-secondary);
}

.search-input {
  font-size: 12px;
  padding: 5px 8px;
}

.filter-bar select {
  font-size: 12px;
  padding: 4px 6px;
  border: 1px solid var(--color-border-primary);
  border-radius: var(--border-radius-sm);
  background: var(--color-background-primary);
  color: var(--color-text-primary);
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

.problem-item {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 9px 12px;
  text-align: left;
  border: none;
  border-bottom: 1px solid var(--color-border-tertiary);
  background: none;
  cursor: pointer;
  transition: background 0.1s;
  width: 100%;
  overflow: hidden;
}

.problem-item:hover { background: var(--color-background-secondary); }
.problem-item.active { background: var(--color-background-info); }

.problem-item-top { display: flex; align-items: center; gap: 6px; }

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

.struct-badge {
  font-size: 10px;
  font-weight: 600;
  color: var(--color-text-warning);
  background: var(--color-background-warning);
  padding: 1px 5px;
  border-radius: 3px;
}

.problem-item-title {
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.problem-item-date { font-size: 10px; color: var(--color-text-tertiary); }

/* ── 상세 패널 ── */
.detail-panel {
  flex: 1;
  overflow-y: auto;
  background: var(--color-background-secondary);
}

.detail-empty {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--color-text-tertiary);
  font-size: 13px;
}

.empty-icon { color: var(--color-text-tertiary); }

.editor-wrap {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

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

.detail-loading {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--color-text-secondary);
  padding: 24px;
}

/* ── 읽기 뷰 ── */
.problem-view {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.view-section { display: flex; flex-direction: column; gap: 6px; }
.view-label { font-size: 11px; font-weight: 600; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: 0.04em; }
.view-content { font-size: 13px; color: var(--color-text-primary); }
.pre-wrap { white-space: pre-wrap; }

.json-view {
  font-family: 'Courier New', monospace;
  font-size: 12px;
  background: var(--color-background-secondary);
  border: 1px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md);
  padding: 10px 12px;
  margin: 0;
  overflow-x: auto;
  max-height: 300px;
  overflow-y: auto;
}

.struct-badge-large {
  display: inline-flex;
  align-items: center;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-warning);
  background: var(--color-background-warning);
  padding: 4px 10px;
  border-radius: var(--border-radius-sm);
}

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
