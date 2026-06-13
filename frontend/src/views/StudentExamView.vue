<template>
  <div class="exam-root">
    <!-- 시험 헤더 -->
    <div class="exam-header" :class="{ 'exam-header--paused': session.is_paused }">
      <div class="exam-header-left">
        <span class="exam-icon">⚠</span>
        <div>
          <div class="exam-title">{{ session.assessment_title }}</div>
          <div class="exam-sub">{{ studentName }} · 시험 진행 중</div>
        </div>
      </div>
      <div class="exam-header-right">
        <span v-if="session.is_paused" class="pause-tag">일시정지</span>
        <span v-else-if="timeRemaining !== null" class="timer" :class="{ 'timer--low': timeRemaining < 300 }">
          남은 시간 {{ formatTime(timeRemaining) }}
        </span>
        <button class="btn-ghost" @click="$emit('logout')">로그아웃</button>
      </div>
    </div>

    <!-- 로딩 -->
    <div v-if="store.examLoading" class="center-loading">
      <div class="spinner" />
    </div>

    <!-- 에러 -->
    <div v-else-if="store.error" class="error-banner">{{ store.error }}</div>

    <!-- 문제 없음 -->
    <div v-else-if="store.sessionProblems.length === 0" class="center-loading">
      <p>등록된 문제가 없습니다.</p>
    </div>

    <!-- 문제 목록 -->
    <div v-else class="problem-list">
      <div
        v-for="(p, idx) in store.sessionProblems"
        :key="p.problem_id"
        class="problem-card"
        :id="`problem-${p.problem_id}`"
      >
        <!-- 문제 헤더 -->
        <div class="problem-card-header">
          <div class="problem-meta">
            <span class="problem-num">{{ idx + 1 }}번</span>
            <span class="type-badge" :class="`type-${p.problem_type}`">
              {{ typeLabel(p.problem_type) }}
            </span>
            <span v-if="p.is_structure_check" class="sc-badge">⑤구조검사</span>
          </div>
          <span class="problem-score">{{ p.max_score }}점</span>
        </div>

        <h3 class="problem-title">{{ p.title }}</h3>
        <p v-if="p.description" class="problem-desc">{{ p.description }}</p>

        <!-- 유형 ①: 실행결과맞히기 -->
        <template v-if="p.problem_type === 1">
          <pre class="code-block">{{ parsedConfig(p).code ?? '' }}</pre>
          <label class="input-label">예상 실행 결과</label>
          <textarea
            v-model="drafts[p.problem_id]"
            class="answer-textarea mono"
            rows="5"
            placeholder="실행 결과를 그대로 입력하세요"
          />
        </template>

        <!-- 유형 ②: 코드작성형 -->
        <template v-else-if="p.problem_type === 2">
          <div class="type2-meta">
            <span class="meta-chip">시간 {{ parsedConfig(p).time_limit_sec ?? 2 }}s</span>
            <span class="meta-chip">메모리 {{ parsedConfig(p).memory_limit_mb ?? 256 }}MB</span>
          </div>
          <div class="lang-row">
            <select v-model="languages[p.problem_id]" class="lang-select">
              <option
                v-for="lang in (parsedConfig(p).languages ?? ['python'])"
                :key="lang"
                :value="lang"
              >{{ lang }}</option>
            </select>
          </div>
          <textarea
            v-model="drafts[p.problem_id]"
            class="answer-textarea mono"
            rows="10"
            placeholder="코드를 작성하세요"
          />
          <p class="pending-note">※ 자동채점은 현재 준비 중입니다 (제출 후 교사 채점)</p>
        </template>

        <!-- 유형 ③: 과제/보고서형 -->
        <template v-else-if="p.problem_type === 3">
          <textarea
            v-model="drafts[p.problem_id]"
            class="answer-textarea"
            rows="8"
            placeholder="보고서 내용을 작성하세요"
          />
        </template>

        <!-- 유형 ④: 빈칸채우기 -->
        <template v-else-if="p.problem_type === 4">
          <div class="blank-code-wrapper">
            <template v-for="(seg, si) in codeSegments(p)" :key="si">
              <span v-if="seg.type === 'text'" class="code-segment">{{ seg.value }}</span>
              <input
                v-else
                :value="blankValues(p.problem_id)[seg.id]"
                class="blank-input"
                :placeholder="seg.placeholder"
                @input="onBlankInput(p.problem_id, seg.id, ($event.target as HTMLInputElement).value)"
              />
            </template>
          </div>
          <p class="blank-hint">강조된 칸만 수정할 수 있습니다.</p>
        </template>

        <!-- 제출 상태 + 버튼 -->
        <div class="submit-row">
          <div class="verdict-area">
            <span v-if="p.verdict === 'AC'" class="verdict verdict--ac">✓ 정답</span>
            <span v-else-if="p.verdict === 'WA'" class="verdict verdict--wa">✗ 오답</span>
            <span v-else-if="p.verdict === 'PENDING'" class="verdict verdict--pending">채점 대기중</span>
            <span v-else-if="p.verdict === 'GRADED'" class="verdict verdict--graded">
              채점 완료 {{ p.submitted_score }}점
            </span>
            <span v-else-if="p.submission_id" class="verdict verdict--submitted">제출됨</span>
          </div>
          <button
            class="btn-submit"
            :disabled="submitting[p.problem_id]"
            @click="doSubmit(p)"
          >
            {{ submitting[p.problem_id] ? '제출 중...' : (p.submission_id ? '재제출' : '제출') }}
          </button>
        </div>

        <div v-if="submitErrors[p.problem_id]" class="submit-error">
          {{ submitErrors[p.problem_id] }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { useStudentStore } from '@/stores/student'
import type { SessionProblemRow, StudentActiveSession } from '@/api/client'

const props = defineProps<{
  session: StudentActiveSession
  studentName: string
}>()

defineEmits<{ (e: 'logout'): void }>()

const store = useStudentStore()
const drafts = reactive<Record<number, string>>({})
const languages = reactive<Record<number, string>>({})
const blanks = reactive<Record<number, Record<string, string>>>({})
const submitting = reactive<Record<number, boolean>>({})
const submitErrors = reactive<Record<number, string>>({})
let timerHandle: ReturnType<typeof setInterval> | null = null

// ── 타이머 ──────────────────────────────────────────────────

const nowSec = ref(Math.floor(Date.now() / 1000))

const timeRemaining = computed<number | null>(() => {
  if (!props.session.time_limit_min || !props.session.start_at) return null
  const startSec = Math.floor(new Date(props.session.start_at).getTime() / 1000)
  const limitSec = props.session.time_limit_min * 60
  return Math.max(0, startSec + limitSec - nowSec.value)
})

function formatTime(secs: number) {
  const m = Math.floor(secs / 60).toString().padStart(2, '0')
  const s = (secs % 60).toString().padStart(2, '0')
  return `${m}:${s}`
}

// ── 유형별 헬퍼 ─────────────────────────────────────────────

type ParsedConfig = Record<string, unknown>

function parsedConfig(p: SessionProblemRow): ParsedConfig {
  try { return JSON.parse(p.type_config) as ParsedConfig }
  catch { return {} }
}

function typeLabel(t: number) {
  return { 1: '①실행결과', 2: '②코드작성', 3: '③과제형', 4: '④빈칸채우기' }[t] ?? `유형${t}`
}

// ④ 빈칸: [[N]] 패턴으로 코드 분리
type Seg = { type: 'text'; value: string } | { type: 'blank'; id: string; placeholder: string }

function codeSegments(p: SessionProblemRow): Seg[] {
  const cfg = parsedConfig(p)
  const code = (cfg.base_code as string) ?? ''
  const markers = (cfg.markers as Array<{ id: number; placeholder?: string }>) ?? []
  const placeholderMap: Record<string, string> = {}
  for (const m of markers) placeholderMap[String(m.id)] = m.placeholder ?? '?'

  const parts = code.split(/(\[\[\d+\]\])/)
  return parts.map(part => {
    const match = part.match(/^\[\[(\d+)\]\]$/)
    if (match) {
      const id = match[1]
      return { type: 'blank' as const, id, placeholder: placeholderMap[id] ?? '?' }
    }
    return { type: 'text' as const, value: part }
  })
}

function blankValues(problemId: number): Record<string, string> {
  return blanks[problemId] ?? {}
}

function onBlankInput(problemId: number, blankId: string, value: string) {
  if (!blanks[problemId]) blanks[problemId] = {}
  blanks[problemId][blankId] = value
  drafts[problemId] = JSON.stringify(blanks[problemId])
}

// ── 초기화 ──────────────────────────────────────────────────

function initDrafts() {
  for (const p of store.sessionProblems) {
    if (p.submitted_content !== null) {
      drafts[p.problem_id] = p.submitted_content
      if (p.problem_type === 4) {
        try { blanks[p.problem_id] = JSON.parse(p.submitted_content) } catch { /* noop */ }
      }
    } else {
      drafts[p.problem_id] = ''
    }
    if (p.submitted_language) languages[p.problem_id] = p.submitted_language
    else {
      const cfg = parsedConfig(p)
      const langs = cfg.languages as string[] | undefined
      languages[p.problem_id] = langs?.[0] ?? 'python'
    }
  }
}

onMounted(async () => {
  await store.fetchSessionProblems()
  initDrafts()
  timerHandle = setInterval(() => { nowSec.value = Math.floor(Date.now() / 1000) }, 1000)
})

onUnmounted(() => {
  if (timerHandle) clearInterval(timerHandle)
})

// ── 제출 ────────────────────────────────────────────────────

async function doSubmit(p: SessionProblemRow) {
  const content = drafts[p.problem_id] ?? ''
  if (!content.trim()) {
    submitErrors[p.problem_id] = '답을 입력해주세요'
    return
  }
  submitErrors[p.problem_id] = ''
  submitting[p.problem_id] = true
  try {
    await store.submitAnswer({
      problem_id: p.problem_id,
      content,
      language: p.problem_type === 2 ? (languages[p.problem_id] ?? 'python') : undefined,
    })
  } catch (e) {
    submitErrors[p.problem_id] = e instanceof Error ? e.message : '제출에 실패했습니다'
  } finally {
    submitting[p.problem_id] = false
  }
}
</script>

<style scoped>
.exam-root {
  min-height: 100vh;
  background: var(--color-background-secondary);
  display: flex;
  flex-direction: column;
}

/* ── 헤더 ──────────────────────────────────────────────── */
.exam-header {
  background: var(--color-background-danger);
  border-bottom: 1px solid var(--color-border-danger);
  padding: 10px 1.5rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  position: sticky;
  top: 0;
  z-index: 10;
}

.exam-header--paused {
  background: var(--color-background-warning);
  border-color: var(--color-border-secondary);
}

.exam-header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.exam-icon {
  font-size: 18px;
}

.exam-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--color-text-danger);
}

.exam-header--paused .exam-title {
  color: var(--color-text-warning);
}

.exam-sub {
  font-size: 11px;
  color: var(--color-text-danger);
  opacity: 0.8;
}

.exam-header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.timer {
  font-size: 13px;
  font-variant-numeric: tabular-nums;
  color: var(--color-text-danger);
  font-weight: 500;
}

.timer--low {
  font-weight: 700;
}

.pause-tag {
  font-size: 12px;
  color: var(--color-text-warning);
  background: var(--color-background-primary);
  padding: 2px 8px;
  border-radius: var(--border-radius-md);
}

.btn-ghost {
  font-size: 12px;
  color: var(--color-text-secondary);
  border: none;
  background: none;
  cursor: pointer;
  padding: 4px 8px;
}

/* ── 로딩/에러 ─────────────────────────────────────────── */
.center-loading {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 13px;
  color: var(--color-text-secondary);
  padding: 4rem;
}

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--color-border-secondary);
  border-top-color: var(--color-text-info);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

.error-banner {
  background: var(--color-background-danger);
  color: var(--color-text-danger);
  font-size: 13px;
  padding: 10px 1.5rem;
}

/* ── 문제 목록 ─────────────────────────────────────────── */
.problem-list {
  max-width: 760px;
  margin: 0 auto;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  width: 100%;
}

.problem-card {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-lg);
  padding: 1.25rem;
}

.problem-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.problem-meta {
  display: flex;
  align-items: center;
  gap: 6px;
}

.problem-num {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.type-badge {
  font-size: 11px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: var(--border-radius-md);
  background: var(--color-background-info);
  color: var(--color-text-info);
}

.sc-badge {
  font-size: 11px;
  color: var(--color-text-warning);
  background: var(--color-background-warning);
  padding: 2px 6px;
  border-radius: var(--border-radius-md);
}

.problem-score {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.problem-title {
  font-size: 14px;
  font-weight: 600;
  margin: 0 0 6px;
}

.problem-desc {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin: 0 0 12px;
  line-height: 1.6;
}

/* ── 입력 영역 ─────────────────────────────────────────── */
.code-block {
  background: var(--color-background-secondary);
  border-radius: var(--border-radius-md);
  padding: 12px;
  font-family: 'Consolas', 'SF Mono', monospace;
  font-size: 13px;
  line-height: 1.6;
  margin: 0 0 12px;
  overflow-x: auto;
  white-space: pre;
}

.input-label {
  display: block;
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-bottom: 6px;
}

.answer-textarea {
  width: 100%;
  border: 0.5px solid var(--color-border-primary);
  border-radius: var(--border-radius-md);
  padding: 8px 10px;
  font-size: 13px;
  line-height: 1.6;
  resize: vertical;
  box-sizing: border-box;
}

.answer-textarea.mono {
  font-family: 'Consolas', 'SF Mono', monospace;
}

.type2-meta {
  display: flex;
  gap: 6px;
  margin-bottom: 8px;
}

.meta-chip {
  font-size: 11px;
  background: var(--color-background-secondary);
  color: var(--color-text-secondary);
  padding: 2px 8px;
  border-radius: var(--border-radius-md);
}

.lang-row {
  margin-bottom: 8px;
}

.lang-select {
  font-size: 12px;
  padding: 4px 8px;
}

.pending-note {
  font-size: 11px;
  color: var(--color-text-tertiary);
  margin: 6px 0 0;
}

/* ── 유형 ④ 빈칸 ───────────────────────────────────────── */
.blank-code-wrapper {
  background: var(--color-background-secondary);
  border-radius: var(--border-radius-md);
  padding: 12px;
  font-family: 'Consolas', 'SF Mono', monospace;
  font-size: 13px;
  line-height: 2.2;
  margin-bottom: 8px;
  white-space: pre-wrap;
  word-break: break-all;
}

.code-segment {
  white-space: pre-wrap;
}

.blank-input {
  display: inline-block;
  font-family: 'Consolas', 'SF Mono', monospace;
  font-size: 13px;
  width: 80px;
  height: 22px;
  padding: 0 4px;
  text-align: center;
  border: 1px solid var(--color-border-info);
  border-radius: 4px;
  background: var(--color-background-info);
  vertical-align: baseline;
}

.blank-hint {
  font-size: 11px;
  color: var(--color-text-tertiary);
  margin: 0 0 8px;
}

/* ── 제출 영역 ─────────────────────────────────────────── */
.submit-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 12px;
}

.verdict-area {
  font-size: 12px;
}

.verdict--ac   { color: var(--color-text-success); font-weight: 500; }
.verdict--wa   { color: var(--color-text-danger); }
.verdict--pending { color: var(--color-text-warning); }
.verdict--graded  { color: var(--color-text-info); font-weight: 500; }
.verdict--submitted { color: var(--color-text-secondary); }

.btn-submit {
  font-size: 13px;
  padding: 7px 20px;
  border: 0.5px solid var(--color-border-info);
  border-radius: var(--border-radius-md);
  background: var(--color-background-info);
  color: var(--color-text-info);
  cursor: pointer;
  font-weight: 500;
}

.btn-submit:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.submit-error {
  font-size: 12px;
  color: var(--color-text-danger);
  margin-top: 6px;
}
</style>
