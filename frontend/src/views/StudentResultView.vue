<template>
  <div class="page-bg">
    <!-- 로딩 -->
    <div v-if="loading" class="center-state">
      <div class="spinner" />
      <span>결과를 불러오는 중...</span>
    </div>

    <!-- 에러 -->
    <div v-else-if="error" class="center-state error-state">
      <span>{{ error }}</span>
      <button class="btn-secondary" @click="load">다시 시도</button>
    </div>

    <!-- 결과 화면 -->
    <template v-else>
      <div class="container">
        <!-- 헤더 -->
        <header class="result-header">
          <RouterLink :to="{ name: 'student-home' }" class="back-link">
            ← 수행평가 목록
          </RouterLink>
          <div class="header-info">
            <h1 class="result-title">{{ assessmentTitle }}</h1>
            <div class="score-summary">
              <span class="score-label">내 점수</span>
              <span class="score-value">
                <span class="score-mine">{{ myTotal }}</span>
                <span class="score-sep"> / </span>
                <span class="score-max">{{ maxTotal }}점</span>
              </span>
            </div>
          </div>
        </header>

        <!-- 문제 목록 -->
        <div v-if="problems.length === 0" class="empty-state">
          등록된 문제가 없습니다.
        </div>

        <div v-else class="problem-list">
          <div
            v-for="(p, idx) in problems"
            :key="p.problem_id"
            class="problem-card"
          >
            <div class="problem-header">
              <div class="problem-meta">
                <span class="problem-num">{{ idx + 1 }}번</span>
                <span class="type-badge" :class="`type-${p.problem_type}`">
                  {{ typeLabel(p.problem_type) }}
                </span>
              </div>
              <div class="problem-score-area">
                <span v-if="p.submitted_score !== null" class="score-chip score-chip--graded">
                  {{ p.submitted_score }} / {{ p.max_score }}점
                </span>
                <span v-else-if="p.verdict === 'AC'" class="score-chip score-chip--ac">
                  정답 ({{ p.max_score }}점)
                </span>
                <span v-else-if="p.verdict === 'WA'" class="score-chip score-chip--wa">
                  오답 (0점)
                </span>
                <span v-else-if="p.verdict === 'PENDING'" class="score-chip score-chip--pending">
                  채점 대기
                </span>
                <span v-else-if="!p.submission_id" class="score-chip score-chip--missing">
                  미제출
                </span>
                <span v-else class="score-chip score-chip--pending">
                  채점 전
                </span>
              </div>
            </div>

            <h3 class="problem-title">{{ p.title }}</h3>
            <p v-if="p.description" class="problem-desc">{{ p.description }}</p>

            <!-- 내 답안 -->
            <div v-if="p.submitted_content" class="my-answer">
              <div class="answer-label">내 답안</div>
              <template v-if="p.problem_type === 4">
                <div class="blank-result">
                  <template v-for="(seg, si) in codeSegments(p)" :key="si">
                    <span v-if="seg.type === 'text'" class="code-segment">{{ seg.value }}</span>
                    <span v-else class="blank-answer">{{ blankAnswer(p, seg.id) }}</span>
                  </template>
                </div>
              </template>
              <pre v-else class="answer-pre">{{ p.submitted_content }}</pre>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { api } from '@/api/client'
import type { SessionProblemRow } from '@/api/client'

const route = useRoute()
const sessionId = Number(route.params.sessionId)

const problems = ref<SessionProblemRow[]>([])
const assessmentTitle = ref('')
const loading = ref(false)
const error = ref<string | null>(null)

const myTotal = computed(() =>
  problems.value.reduce((sum, p) => {
    if (p.submitted_score !== null) return sum + p.submitted_score
    if (p.verdict === 'AC') return sum + p.max_score
    return sum
  }, 0)
)

const maxTotal = computed(() =>
  problems.value.reduce((sum, p) => sum + p.max_score, 0)
)

async function load() {
  loading.value = true
  error.value = null
  try {
    problems.value = await api.student.sessionResultProblems(sessionId)
  } catch (e) {
    error.value = e instanceof Error ? e.message : '결과를 불러올 수 없습니다'
  } finally {
    loading.value = false
  }
}

onMounted(load)

function typeLabel(t: number) {
  return { 1: '①실행결과', 2: '②코드작성', 3: '③과제형', 4: '④빈칸채우기' }[t] ?? `유형${t}`
}

type ParsedConfig = Record<string, unknown>

function parsedConfig(p: SessionProblemRow): ParsedConfig {
  try { return JSON.parse(p.type_config) as ParsedConfig }
  catch { return {} }
}

type Seg = { type: 'text'; value: string } | { type: 'blank'; id: string }

function codeSegments(p: SessionProblemRow): Seg[] {
  const cfg = parsedConfig(p)
  const code = (cfg.base_code as string) ?? ''
  const parts = code.split(/(\[\[\d+\]\])/)
  return parts.map(part => {
    const match = part.match(/^\[\[(\d+)\]\]$/)
    if (match) return { type: 'blank' as const, id: match[1] }
    return { type: 'text' as const, value: part }
  })
}

function blankAnswer(p: SessionProblemRow, id: string): string {
  if (!p.submitted_content) return '(미입력)'
  try {
    const map = JSON.parse(p.submitted_content) as Record<string, string>
    return map[id] ?? '(미입력)'
  } catch { return '(미입력)' }
}
</script>

<style scoped>
.page-bg {
  min-height: 100vh;
  background: var(--color-background-secondary);
}

.center-state {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--color-text-secondary);
}

.error-state { color: var(--color-text-danger); }

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--color-border-secondary);
  border-top-color: var(--color-text-info);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

.container {
  max-width: 760px;
  margin: 0 auto;
  padding: 1.5rem;
}

.back-link {
  display: inline-block;
  color: var(--color-text-secondary);
  text-decoration: none;
  font-size: 13px;
  margin-bottom: 1rem;
}
.back-link:hover { color: var(--color-text-primary); }

.result-header {
  margin-bottom: 1.5rem;
}

.header-info {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  flex-wrap: wrap;
  gap: 8px;
}

.result-title {
  font-size: 20px;
  font-weight: 700;
  margin: 0;
}

.score-summary {
  display: flex;
  align-items: center;
  gap: 8px;
}

.score-label {
  font-size: 13px;
  color: var(--color-text-secondary);
}

.score-value {
  display: flex;
  align-items: baseline;
}

.score-mine {
  font-size: 22px;
  font-weight: 700;
  color: var(--color-text-info);
}

.score-sep { font-size: 14px; color: var(--color-text-tertiary); margin: 0 2px; }

.score-max { font-size: 14px; color: var(--color-text-secondary); }

.empty-state {
  text-align: center;
  color: var(--color-text-tertiary);
  padding: 3rem 0;
}

.problem-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.problem-card {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-lg);
  padding: 1rem 1.25rem;
}

.problem-header {
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
  font-weight: 600;
  font-size: 13px;
}

.type-badge {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: var(--border-radius-md);
  background: var(--color-background-secondary);
  color: var(--color-text-secondary);
}

.type-1 { background: var(--color-background-info); color: var(--color-text-info); }
.type-2 { background: var(--color-background-success); color: var(--color-text-success); }
.type-3 { background: var(--color-background-warning); color: var(--color-text-warning); }
.type-4 { background: var(--color-background-secondary); color: var(--color-text-secondary); }

.score-chip {
  font-size: 12px;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: var(--border-radius-md);
}

.score-chip--graded { background: var(--color-background-info); color: var(--color-text-info); }
.score-chip--ac     { background: var(--color-background-success); color: var(--color-text-success); }
.score-chip--wa     { background: var(--color-background-danger); color: var(--color-text-danger); }
.score-chip--pending { background: var(--color-background-warning); color: var(--color-text-warning); }
.score-chip--missing { background: var(--color-background-secondary); color: var(--color-text-tertiary); }

.problem-title {
  font-size: 15px;
  font-weight: 500;
  margin: 0 0 4px;
}

.problem-desc {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin: 0 0 10px;
}

.my-answer {
  border-top: 0.5px solid var(--color-border-tertiary);
  padding-top: 10px;
  margin-top: 10px;
}

.answer-label {
  font-size: 11px;
  color: var(--color-text-tertiary);
  margin-bottom: 6px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.answer-pre {
  background: var(--color-background-secondary);
  border-radius: var(--border-radius-md);
  padding: 10px 12px;
  font-family: monospace;
  font-size: 13px;
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
  max-height: 200px;
  overflow-y: auto;
}

.blank-result {
  background: var(--color-background-secondary);
  border-radius: var(--border-radius-md);
  padding: 10px 12px;
  font-family: monospace;
  font-size: 13px;
  white-space: pre-wrap;
}

.code-segment { color: var(--color-text-secondary); }

.blank-answer {
  background: var(--color-background-info);
  color: var(--color-text-info);
  border-radius: 3px;
  padding: 0 4px;
  font-weight: 600;
}

.btn-secondary {
  font-size: 13px;
  padding: 6px 14px;
  border: 0.5px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md);
  background: var(--color-background-primary);
  cursor: pointer;
}
</style>
