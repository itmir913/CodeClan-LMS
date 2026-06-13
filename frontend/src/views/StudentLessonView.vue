<template>
  <div class="page-bg">
    <!-- 로딩 -->
    <div v-if="loading" class="center-state">
      <div class="spinner" />
      <span>불러오는 중...</span>
    </div>

    <!-- 에러 -->
    <div v-else-if="error" class="center-state error-state">
      <span>{{ error }}</span>
      <button class="btn-secondary" @click="load">다시 시도</button>
    </div>

    <!-- 본문 -->
    <template v-else-if="lesson">
      <!-- 상단 바 -->
      <div class="top-bar">
        <RouterLink :to="{ name: 'student-home' }" class="back-link">
          ← 목록으로
        </RouterLink>
        <span class="lesson-title-bar">{{ lesson.title }}</span>
        <span class="lesson-order">차시 {{ lesson.order_no }}</span>
      </div>

      <div class="container">
        <!-- 차시 설명 -->
        <div v-if="lesson.description" class="desc-card">
          {{ lesson.description }}
        </div>

        <!-- 문항 없음 -->
        <div v-if="lesson.problems.length === 0" class="empty-state">
          배정된 문항이 없습니다.
        </div>

        <!-- 문항 목록 -->
        <template v-else>
          <div class="problem-nav">
            <span class="nav-label">문항 {{ currentIdx + 1 }} / {{ lesson.problems.length }}</span>
            <div class="nav-btns">
              <button :disabled="currentIdx <= 0" @click="currentIdx--">← 이전</button>
              <button :disabled="currentIdx >= lesson.problems.length - 1" @click="currentIdx++">다음 →</button>
            </div>
          </div>

          <!-- 현재 문항 카드 -->
          <div class="problem-card">
            <div class="problem-header">
              <span class="problem-no">{{ currentIdx + 1 }}</span>
              <span class="type-badge" :class="`type-${current.problem_type}`">
                {{ typeLabel(current.problem_type) }}
              </span>
              <span class="problem-title">{{ current.problem_title }}</span>
            </div>
            <div v-if="current.description" class="problem-desc">
              {{ current.description }}
            </div>

            <!-- 유형별 추가 정보 -->
            <div v-if="parsedConfig" class="type-config-area">
              <!-- 유형 1: 실행결과 맞히기 -->
              <template v-if="current.problem_type === 1">
                <div v-if="parsedConfig.code" class="code-block">
                  <div class="code-label">코드</div>
                  <pre>{{ parsedConfig.code }}</pre>
                </div>
              </template>

              <!-- 유형 2: 코드 작성형 -->
              <template v-else-if="current.problem_type === 2">
                <div v-if="parsedConfig.template_code" class="code-block">
                  <div class="code-label">템플릿 코드</div>
                  <pre>{{ parsedConfig.template_code }}</pre>
                </div>
              </template>

              <!-- 유형 4: 빈칸채우기 -->
              <template v-else-if="current.problem_type === 4">
                <div v-if="parsedConfig.code_template" class="code-block">
                  <div class="code-label">코드 (빈칸 채우기)</div>
                  <pre>{{ parsedConfig.code_template }}</pre>
                </div>
              </template>
            </div>
          </div>

          <!-- 문항 인덱스 -->
          <div class="problem-index">
            <button
              v-for="(p, idx) in lesson.problems"
              :key="p.id"
              class="index-btn"
              :class="{ 'index-btn--active': idx === currentIdx }"
              @click="currentIdx = idx"
            >{{ idx + 1 }}</button>
          </div>
        </template>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter, RouterLink } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { api, type StudentLessonDetail } from '@/api/client'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const lessonId = Number(route.params.id)
const lesson = ref<StudentLessonDetail | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const currentIdx = ref(0)
let heartbeatTimer: ReturnType<typeof setInterval> | null = null

const current = computed(() => lesson.value!.problems[currentIdx.value])

const parsedConfig = computed((): Record<string, string> | null => {
  if (!current.value?.type_config) return null
  try {
    return JSON.parse(current.value.type_config)
  } catch {
    return null
  }
})

function typeLabel(t: number): string {
  return (
    { 1: '①실행결과', 2: '②코드작성', 3: '③과제형', 4: '④빈칸채우기' }[t] ?? `유형${t}`
  )
}

async function load() {
  loading.value = true
  error.value = null
  try {
    lesson.value = await api.student.lessonDetail(lessonId)
  } catch (e) {
    error.value = e instanceof Error ? e.message : '차시를 불러오지 못했습니다'
  } finally {
    loading.value = false
  }
}

function startHeartbeat() {
  api.attendance.heartbeat('lesson', lessonId).catch(() => {})
  heartbeatTimer = setInterval(() => {
    api.attendance.heartbeat('lesson', lessonId).catch(() => {})
  }, 15000)
}

onMounted(async () => {
  if (!auth.isStudentLoggedIn) {
    await auth.fetchStudentMe()
    if (!auth.isStudentLoggedIn) {
      router.replace({ name: 'login' })
      return
    }
  }
  await load()
  startHeartbeat()
})

onUnmounted(() => {
  if (heartbeatTimer !== null) clearInterval(heartbeatTimer)
})
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

/* ── 상단 바 ── */
.top-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 1.5rem;
  background: var(--color-background-primary);
  border-bottom: 0.5px solid var(--color-border-secondary);
}

.back-link {
  color: var(--color-text-info);
  text-decoration: none;
  white-space: nowrap;
  flex-shrink: 0;
}

.back-link:hover { text-decoration: underline; }

.lesson-title-bar {
  font-weight: 600;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.lesson-order {
  color: var(--color-text-tertiary);
  flex-shrink: 0;
  white-space: nowrap;
}

/* ── 콘텐츠 ── */
.container {
  max-width: 760px;
  margin: 0 auto;
  padding: 1.25rem 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.desc-card {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-md);
  padding: 12px 14px;
  color: var(--color-text-secondary);
}

.empty-state {
  text-align: center;
  color: var(--color-text-tertiary);
  padding: 3rem 0;
}

/* ── 문항 내비 ── */
.problem-nav {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.nav-label {
  color: var(--color-text-secondary);
  font-weight: 500;
}

.nav-btns {
  display: flex;
  gap: 8px;
}

.nav-btns button {
  padding: 5px 12px;
  border: 0.5px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md);
  background: var(--color-background-primary);
  color: var(--color-text-primary);
  cursor: pointer;
}

.nav-btns button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* ── 문항 카드 ── */
.problem-card {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-lg);
  padding: 1.25rem;
}

.problem-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.problem-no {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--color-background-secondary);
  color: var(--color-text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  flex-shrink: 0;
}

.type-badge {
  padding: 2px 8px;
  border-radius: var(--border-radius-md);
  background: var(--color-background-secondary);
  color: var(--color-text-secondary);
  white-space: nowrap;
}

.problem-title {
  font-weight: 500;
  flex: 1;
}

.problem-desc {
  color: var(--color-text-secondary);
  line-height: 1.6;
  margin-bottom: 12px;
  white-space: pre-wrap;
}

.type-config-area {
  margin-top: 12px;
  border-top: 0.5px solid var(--color-border-tertiary);
  padding-top: 12px;
}

.code-block {
  background: var(--color-background-secondary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-md);
  overflow: hidden;
}

.code-label {
  padding: 5px 12px;
  color: var(--color-text-tertiary);
  border-bottom: 0.5px solid var(--color-border-tertiary);
}

.code-block pre {
  margin: 0;
  padding: 10px 12px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  font-family: monospace;
  color: var(--color-text-primary);
}

/* ── 문항 인덱스 ── */
.problem-index {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.index-btn {
  width: 32px;
  height: 32px;
  border-radius: var(--border-radius-md);
  border: 0.5px solid var(--color-border-secondary);
  background: var(--color-background-primary);
  color: var(--color-text-secondary);
  cursor: pointer;
  font-weight: 500;
  transition: background 0.1s;
}

.index-btn:hover {
  background: var(--color-background-secondary);
}

.index-btn--active {
  background: var(--color-background-info);
  color: var(--color-text-info);
  border-color: var(--color-border-info);
}

/* ── 버튼 ── */
.btn-secondary {
  padding: 6px 14px;
  border: 0.5px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md);
  background: var(--color-background-primary);
  cursor: pointer;
}
</style>
