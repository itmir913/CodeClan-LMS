<template>
  <div class="page-layout">
    <AppSidebar />

    <!-- 채점 패널 (좌: 학생 목록, 우: 제출물) -->
    <div class="grading-split">
    <!-- 좌: 학생 목록 -->
    <aside class="sidebar">
      <div class="sidebar-header">
        <RouterLink :to="{ name: 'session-management' }" class="back-link">← 세션 관리</RouterLink>
        <h2 class="sidebar-title">채점</h2>
        <p class="sidebar-sub">세션 #{{ sessionId }}</p>
      </div>

      <div v-if="store.loading" class="list-loading">로딩 중...</div>
      <div v-else-if="store.error" class="list-error">{{ store.error }}</div>
      <div v-else-if="store.byStudent.length === 0" class="list-empty">제출 내역이 없습니다</div>
      <div v-else class="student-list">
        <div
          v-for="s in store.byStudent"
          :key="s.student_id"
          class="student-item"
          :class="{ selected: selectedStudentId === s.student_id }"
          @click="selectedStudentId = s.student_id"
        >
          <div class="student-item-name">{{ s.name }}</div>
          <div class="student-item-sub">{{ s.student_number }}</div>
          <div class="student-item-score">
            {{ totalScore(s.subs) }} / {{ totalMax(s.subs) }}점
          </div>
        </div>
      </div>

      <div class="sidebar-footer">
        <button class="btn btn-secondary" style="width:100%" @click="reload">
          새로고침
        </button>
      </div>
    </aside>

    <!-- 메인: 선택된 학생의 제출물 -->
    <main class="main-content">
      <div v-if="!selectedStudent" class="empty-state">
        <p>왼쪽에서 학생을 선택하세요</p>
      </div>

      <div v-else>
        <div class="detail-header">
          <h2 class="detail-title">{{ selectedStudent.name }}</h2>
          <p class="detail-sub">
            학번 {{ selectedStudent.student_number }} ·
            총점 {{ totalScore(selectedStudent.subs) }} / {{ totalMax(selectedStudent.subs) }}점
          </p>
        </div>

        <div class="submission-list">
          <div
            v-for="sub in selectedStudent.subs"
            :key="sub.id"
            class="sub-card"
          >
            <!-- 문제 헤더 -->
            <div class="sub-card-header">
              <div class="sub-meta">
                <span class="sub-order">{{ sub.problem_order + 1 }}번</span>
                <span class="sub-type">{{ typeLabel(sub.problem_type) }}</span>
                <span class="sub-title">{{ sub.problem_title }}</span>
              </div>
              <span class="sub-maxscore">{{ sub.max_score }}점</span>
            </div>

            <!-- 제출 내용 -->
            <div class="sub-content-area">
              <!-- 유형 ①②: 코드/텍스트 -->
              <template v-if="sub.problem_type === 1 || sub.problem_type === 2">
                <div class="sub-lang" v-if="sub.language">{{ sub.language }}</div>
                <pre class="sub-code">{{ sub.content }}</pre>
              </template>

              <!-- 유형 ③: 보고서 -->
              <template v-else-if="sub.problem_type === 3">
                <div class="sub-essay">{{ sub.content }}</div>
              </template>

              <!-- 유형 ④: 빈칸 -->
              <template v-else-if="sub.problem_type === 4">
                <div class="blank-answers">
                  <template v-for="(val, key) in parsedBlanks(sub.content)" :key="key">
                    <span class="blank-key">칸 {{ key }}:</span>
                    <code class="blank-val">{{ val }}</code>
                  </template>
                </div>
              </template>
            </div>

            <!-- 채점 영역 -->
            <div class="grade-area">
              <!-- 자동채점 결과 (유형①②) -->
              <template v-if="sub.problem_type === 1 || sub.problem_type === 2">
                <span
                  class="verdict-badge"
                  :class="verdictClass(sub.verdict)"
                >{{ verdictLabel(sub.verdict) }}</span>
                <span v-if="sub.score !== null" class="auto-score">{{ sub.score }}점</span>
              </template>

              <!-- 수동 채점 (유형②PENDING, ③④) -->
              <template v-if="needsManual(sub)">
                <div class="manual-grade">
                  <label class="grade-label">점수 입력</label>
                  <input
                    v-model.number="scoreInputs[sub.id]"
                    type="number"
                    min="0"
                    :max="sub.max_score"
                    class="score-input"
                    :placeholder="`0~${sub.max_score}`"
                  />
                  <button
                    class="btn btn-primary btn-sm"
                    :disabled="grading[sub.id]"
                    @click="doGrade(sub)"
                  >
                    {{ grading[sub.id] ? '저장 중...' : '저장' }}
                  </button>
                </div>
                <div v-if="gradeErrors[sub.id]" class="grade-error">{{ gradeErrors[sub.id] }}</div>
              </template>

              <!-- 채점 완료 -->
              <span
                v-else-if="sub.verdict === 'GRADED'"
                class="verdict-badge verdict-graded"
              >채점 완료 {{ sub.score }}점</span>
            </div>
          </div>
        </div>
      </div>
    </main>
    </div><!-- grading-split -->
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useGradingStore } from '@/stores/grading'
import type { SubmissionRow } from '@/api/client'
import AppSidebar from '@/components/AppSidebar.vue'

const route = useRoute()
const store = useGradingStore()

const sessionId = computed(() => Number(route.params.id))
const selectedStudentId = ref<number | null>(null)
const scoreInputs = reactive<Record<number, number>>({})
const grading = reactive<Record<number, boolean>>({})
const gradeErrors = reactive<Record<number, string>>({})

const selectedStudent = computed(() =>
  store.byStudent.find(s => s.student_id === selectedStudentId.value) ?? null
)

async function reload() {
  await store.fetchSubmissions(sessionId.value)
  if (store.byStudent.length > 0 && !selectedStudentId.value) {
    selectedStudentId.value = store.byStudent[0].student_id
  }
}

onMounted(reload)

function typeLabel(t: number) {
  return { 1: '①실행결과', 2: '②코드작성', 3: '③과제형', 4: '④빈칸채우기' }[t] ?? `유형${t}`
}

function verdictLabel(v: string | null) {
  if (!v) return '미제출'
  return { AC: '정답', WA: '오답', PENDING: '채점 대기', GRADED: '채점 완료' }[v] ?? v
}

function verdictClass(v: string | null) {
  if (!v) return 'verdict-none'
  return { AC: 'verdict-ac', WA: 'verdict-wa', PENDING: 'verdict-pending', GRADED: 'verdict-graded' }[v] ?? ''
}

function needsManual(sub: SubmissionRow) {
  if (sub.problem_type === 1 && sub.verdict && sub.verdict !== 'GRADED') return false
  if (sub.problem_type === 2 && sub.verdict === 'AC') return false
  return sub.verdict !== 'GRADED'
}

function parsedBlanks(content: string): Record<string, string> {
  try { return JSON.parse(content) as Record<string, string> }
  catch { return { '?': content } }
}

function totalScore(subs: SubmissionRow[]) {
  return subs.reduce((acc, s) => acc + (s.score ?? 0), 0)
}

function totalMax(subs: SubmissionRow[]) {
  return subs.reduce((acc, s) => acc + s.max_score, 0)
}

async function doGrade(sub: SubmissionRow) {
  const score = scoreInputs[sub.id]
  if (score === undefined || score === null) {
    gradeErrors[sub.id] = '점수를 입력하세요'
    return
  }
  if (score < 0 || score > sub.max_score) {
    gradeErrors[sub.id] = `0~${sub.max_score} 범위로 입력하세요`
    return
  }
  gradeErrors[sub.id] = ''
  grading[sub.id] = true
  try {
    await store.gradeSubmission(sub.id, score)
  } catch (e) {
    gradeErrors[sub.id] = e instanceof Error ? e.message : '채점에 실패했습니다'
  } finally {
    grading[sub.id] = false
  }
}
</script>

<style scoped>
.page-layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.grading-split {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-width: 0;
}

/* ── 사이드바 ──────────────────────────────────────── */
.sidebar {
  width: 220px;
  flex-shrink: 0;
  background: var(--color-background-primary);
  border-right: 0.5px solid var(--color-border-secondary);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sidebar-header {
  padding: 1rem 1rem 0.75rem;
  border-bottom: 0.5px solid var(--color-border-secondary);
}

.back-link {
  font-size: 12px;
  color: var(--color-text-secondary);
  text-decoration: none;
  display: block;
  margin-bottom: 8px;
}

.back-link:hover { color: var(--color-text-primary); }

.sidebar-title {
  font-size: 15px;
  font-weight: 600;
  margin: 0 0 2px;
}

.sidebar-sub {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin: 0;
}

.list-loading, .list-empty, .list-error {
  padding: 1rem;
  font-size: 12px;
  color: var(--color-text-tertiary);
  text-align: center;
}

.list-error { color: var(--color-text-danger); }

.student-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.student-item {
  padding: 10px 8px;
  border-radius: var(--border-radius-md);
  cursor: pointer;
  border: 0.5px solid transparent;
  margin-bottom: 4px;
}

.student-item:hover {
  background: var(--color-background-secondary);
}

.student-item.selected {
  background: var(--color-background-info);
  border-color: var(--color-border-info);
}

.student-item-name {
  font-size: 13px;
  font-weight: 500;
}

.student-item-sub {
  font-size: 11px;
  color: var(--color-text-tertiary);
  margin-top: 1px;
}

.student-item-score {
  font-size: 12px;
  color: var(--color-text-info);
  margin-top: 3px;
}

.sidebar-footer {
  padding: 12px;
  border-top: 0.5px solid var(--color-border-secondary);
}

/* ── 메인 ──────────────────────────────────────────── */
.main-content {
  flex: 1;
  background: var(--color-background-secondary);
  overflow-y: auto;
  padding: 1.5rem;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  font-size: 13px;
  color: var(--color-text-tertiary);
}

.detail-header {
  margin-bottom: 1.25rem;
}

.detail-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 4px;
}

.detail-sub {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin: 0;
}

.submission-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

/* ── 제출 카드 ─────────────────────────────────────── */
.sub-card {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-lg);
  padding: 1rem 1.25rem;
}

.sub-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.sub-meta {
  display: flex;
  align-items: center;
  gap: 6px;
}

.sub-order {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.sub-type {
  font-size: 11px;
  background: var(--color-background-info);
  color: var(--color-text-info);
  padding: 1px 7px;
  border-radius: var(--border-radius-md);
}

.sub-title {
  font-size: 13px;
  font-weight: 500;
}

.sub-maxscore {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.sub-content-area {
  margin-bottom: 10px;
}

.sub-lang {
  font-size: 11px;
  color: var(--color-text-tertiary);
  margin-bottom: 4px;
}

.sub-code {
  background: var(--color-background-secondary);
  border-radius: var(--border-radius-md);
  padding: 10px;
  font-family: 'Consolas', 'SF Mono', monospace;
  font-size: 12px;
  line-height: 1.6;
  overflow-x: auto;
  margin: 0;
  white-space: pre-wrap;
  max-height: 200px;
  overflow-y: auto;
}

.sub-essay {
  font-size: 13px;
  line-height: 1.7;
  color: var(--color-text-primary);
  white-space: pre-wrap;
  max-height: 150px;
  overflow-y: auto;
  padding: 8px;
  background: var(--color-background-secondary);
  border-radius: var(--border-radius-md);
}

.blank-answers {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  font-size: 13px;
}

.blank-key {
  color: var(--color-text-secondary);
}

.blank-val {
  font-family: monospace;
  background: var(--color-background-info);
  color: var(--color-text-info);
  padding: 1px 6px;
  border-radius: 4px;
}

/* ── 채점 영역 ─────────────────────────────────────── */
.grade-area {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  padding-top: 10px;
  border-top: 0.5px solid var(--color-border-tertiary);
}

.verdict-badge {
  font-size: 11px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: var(--border-radius-md);
}

.verdict-ac      { background: var(--color-background-success); color: var(--color-text-success); }
.verdict-wa      { background: var(--color-background-danger); color: var(--color-text-danger); }
.verdict-pending { background: var(--color-background-warning); color: var(--color-text-warning); }
.verdict-graded  { background: var(--color-background-info); color: var(--color-text-info); }
.verdict-none    { background: var(--color-background-secondary); color: var(--color-text-tertiary); }

.auto-score {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-info);
}

.manual-grade {
  display: flex;
  align-items: center;
  gap: 6px;
}

.grade-label {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.score-input {
  width: 70px;
  font-size: 13px;
  padding: 4px 8px;
  border: 0.5px solid var(--color-border-primary);
  border-radius: var(--border-radius-md);
}

.grade-error {
  width: 100%;
  font-size: 12px;
  color: var(--color-text-danger);
  margin-top: 4px;
}

/* ── 공통 버튼 ─────────────────────────────────────── */
.btn {
  font-size: 13px;
  padding: 7px 14px;
  border-radius: var(--border-radius-md);
  cursor: pointer;
  border: 0.5px solid var(--color-border-secondary);
  background: var(--color-background-primary);
}

.btn-primary {
  background: var(--color-background-info);
  color: var(--color-text-info);
  border-color: var(--color-border-info);
}

.btn-secondary {
  color: var(--color-text-secondary);
}

.btn-sm {
  font-size: 12px;
  padding: 4px 10px;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
