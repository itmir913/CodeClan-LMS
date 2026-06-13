<template>
  <div class="live-root">
    <!-- 로딩 -->
    <div v-if="loading" class="center-state">
      <div class="spinner" />
      <span>불러오는 중...</span>
    </div>

    <!-- 에러 -->
    <div v-else-if="error" class="center-state error-state">
      <span>{{ error }}</span>
      <button @click="load">다시 시도</button>
    </div>

    <!-- 본문 -->
    <template v-else-if="lesson">
      <!-- 상단 breadcrumb + 종료 버튼 -->
      <div class="top-bar">
        <div class="breadcrumb">
          <IconPresentation :size="16" class="icon-success" />
          <span class="bc-division">{{ divisionName }}</span>
          <IconChevronRight :size="14" class="bc-sep" />
          <span class="bc-current">차시 운영 — {{ lesson.title }}</span>
        </div>
        <RouterLink :to="{ name: 'lesson-management' }" class="btn-exit">
          <IconX :size="14" /> 운영 종료
        </RouterLink>
      </div>

      <div class="live-body">
        <!-- 좌: 차시 정보 + 문항 목록 + 수행평가 연계 -->
        <div class="main-col">

          <!-- 차시 카드 -->
          <div class="card">
            <div class="card-top">
              <div>
                <div class="card-title">{{ lesson.title }}</div>
                <div v-if="lesson.description" class="card-desc">{{ lesson.description }}</div>
              </div>
              <span
                class="release-badge"
                :class="divisionRelease?.is_released ? 'badge--success' : 'badge--warning'"
              >
                {{ divisionRelease?.is_released ? '학생에게 공개 중' : '학생에게 미공개' }}
              </span>
            </div>
            <div class="release-actions">
              <button
                class="btn-release"
                :class="divisionRelease?.is_released ? 'btn-release--on' : 'btn-release--off'"
                :disabled="toggling"
                @click="toggleRelease"
              >
                <IconEye v-if="!divisionRelease?.is_released" :size="14" />
                <IconEyeOff v-else :size="14" />
                {{ divisionRelease?.is_released ? '비공개로 전환' : '지금 공개하기' }}
              </button>
              <span class="release-hint">
                <IconInfoCircle :size="13" />
                공개 즉시 학생 화면에 차시 내용이 표시됩니다
              </span>
            </div>
          </div>

          <!-- 배정 문항 목록 -->
          <div class="card">
            <div class="card-header">진행 순서 (배정 문항)</div>
            <div v-if="lesson.problems.length === 0" class="empty-note">배정된 문항이 없습니다</div>
            <div v-else class="problem-seq">
              <div
                v-for="(p, idx) in lesson.problems"
                :key="p.id"
                class="seq-row"
                :class="{ 'seq-row--active': currentProblemIdx === idx }"
                @click="currentProblemIdx = idx"
              >
                <span v-if="currentProblemIdx === idx" class="seq-status">진행중</span>
                <span v-else class="seq-num">{{ idx + 1 }}</span>
                <span class="type-badge" :class="`type-${p.problem_type}`">
                  {{ typeLabel(p.problem_type) }}
                </span>
                <span class="seq-title">{{ p.problem_title }}</span>
              </div>
            </div>
            <div v-if="lesson.problems.length > 0" class="seq-nav">
              <button
                :disabled="currentProblemIdx <= 0"
                @click="currentProblemIdx--"
              >
                <IconArrowLeft :size="13" /> 이전 문항
              </button>
              <button
                :disabled="currentProblemIdx >= lesson.problems.length - 1"
                @click="currentProblemIdx++"
              >
                다음 문항 <IconArrowRight :size="13" />
              </button>
            </div>
          </div>

          <!-- 이어서 수행평가 카드 -->
          <div v-if="assessments.length > 0" class="card card--assess">
            <div class="assess-top">
              <div>
                <div class="card-header">이어서 수행평가를 진행하나요?</div>
                <div class="assess-list">
                  <span
                    v-for="a in assessments"
                    :key="a.id"
                    class="assess-chip"
                  >{{ a.title }} ({{ a.problem_count }}문항)</span>
                </div>
              </div>
              <RouterLink :to="{ name: 'session-management' }" class="btn-session">
                <IconPlayerPlay :size="14" /> 세션 생성/시작
              </RouterLink>
            </div>
          </div>
        </div>

        <!-- 우: 실시간 출결 위젯 -->
        <div class="side-col">
          <AttendanceWidget
            :lesson-id="lessonId"
            :division-id="divisionId"
          />
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { api, type LessonDetail, type DivisionAssessmentRow } from '@/api/client'
import AttendanceWidget from '@/components/AttendanceWidget.vue'
import {
  IconPresentation, IconChevronRight, IconX, IconEye, IconEyeOff,
  IconInfoCircle, IconArrowLeft, IconArrowRight, IconPlayerPlay,
} from '@tabler/icons-vue'

const router = useRouter()
const route = useRoute()
const auth = useAuthStore()

const lessonId = Number(route.params.lessonId)
const divisionId = Number(route.params.divisionId)

const lesson = ref<LessonDetail | null>(null)
const assessments = ref<DivisionAssessmentRow[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const toggling = ref(false)
const currentProblemIdx = ref(0)

const divisionRelease = computed(() =>
  lesson.value?.releases.find(r => r.division_id === divisionId) ?? null
)

const divisionName = computed(() =>
  divisionRelease.value?.division_name ?? `분반 ${divisionId}`
)

function typeLabel(t: number): string {
  return { 1: '①실행결과', 2: '②코드작성', 3: '③과제형', 4: '④빈칸채우기' }[t] ?? `유형${t}`
}

async function load() {
  loading.value = true
  error.value = null
  try {
    const [lessonData, assessData] = await Promise.all([
      api.lessons.get(lessonId),
      api.divisions.assessments(divisionId),
    ])
    lesson.value = lessonData
    assessments.value = assessData
  } catch (e) {
    error.value = e instanceof Error ? e.message : '차시 정보를 불러오지 못했습니다'
  } finally {
    loading.value = false
  }
}

async function toggleRelease() {
  if (!lesson.value || !divisionRelease.value) return
  toggling.value = true
  try {
    const newState = !divisionRelease.value.is_released
    const result = await api.lessons.toggleRelease(lessonId, divisionId, newState)
    const rel = lesson.value.releases.find(r => r.division_id === divisionId)
    if (rel) {
      rel.is_released = newState
      rel.released_at = result.released_at
    }
  } catch (e) {
    alert(e instanceof Error ? e.message : '공개 전환 실패')
  } finally {
    toggling.value = false
  }
}

onMounted(async () => {
  if (!auth.isTeacherLoggedIn) {
    await auth.fetchTeacherMe()
    if (!auth.isTeacherLoggedIn) {
      router.replace({ name: 'login' })
      return
    }
  }
  await load()
})
</script>

<style scoped>
.live-root {
  min-height: 100vh;
  background: var(--color-background-secondary);
  display: flex;
  flex-direction: column;
}

/* ── 상단 바 ── */
.top-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 20px;
  background: var(--color-background-primary);
  border-bottom: 1px solid var(--color-border-secondary);
}

.breadcrumb {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.icon-success { color: var(--color-text-success); }
.bc-division { color: var(--color-text-secondary); }
.bc-sep { color: var(--color-text-tertiary); }
.bc-current { font-weight: 500; color: var(--color-text-primary); }

.btn-exit {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  padding: 4px 10px;
  border: 0.5px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md);
  color: var(--color-text-secondary);
  text-decoration: none;
  background: none;
  cursor: pointer;
}
.btn-exit:hover { color: var(--color-text-primary); background: var(--color-background-secondary); }

/* ── 본문 레이아웃 ── */
.live-body {
  flex: 1;
  display: flex;
  gap: 1rem;
  padding: 1rem;
  max-width: 1200px;
  margin: 0 auto;
  width: 100%;
  box-sizing: border-box;
}

.main-col {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.side-col {
  width: 220px;
  flex-shrink: 0;
}

/* ── 카드 ── */
.card {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-lg);
  padding: 1rem 1.25rem;
}

.card-header {
  font-weight: 500;
  font-size: 14px;
  margin-bottom: 10px;
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 10px;
}

.card-title {
  font-weight: 500;
  font-size: 15px;
}

.card-desc {
  color: var(--color-text-secondary);
  font-size: 13px;
  margin-top: 4px;
}

.release-badge {
  font-size: 11px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: var(--border-radius-md);
  white-space: nowrap;
}

.badge--success { background: var(--color-background-success); color: var(--color-text-success); }
.badge--warning { background: var(--color-background-warning); color: var(--color-text-warning); }

.release-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.btn-release {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  font-weight: 500;
  padding: 5px 12px;
  border-radius: var(--border-radius-md);
  cursor: pointer;
  transition: opacity 0.1s;
}
.btn-release:disabled { opacity: 0.5; cursor: not-allowed; }

.btn-release--off {
  background: var(--color-background-info);
  color: var(--color-text-info);
  border: 0.5px solid transparent;
}

.btn-release--on {
  background: var(--color-background-secondary);
  color: var(--color-text-secondary);
  border: 0.5px solid var(--color-border-secondary);
}

.release-hint {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--color-text-tertiary);
}

/* ── 문항 순서 ── */
.problem-seq {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 10px;
}

.seq-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: var(--border-radius-md);
  border: 0.5px solid var(--color-border-tertiary);
  cursor: pointer;
  transition: background 0.1s;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.seq-row:hover { background: var(--color-background-secondary); }

.seq-row--active {
  background: var(--color-background-info);
  border-color: var(--color-border-info);
  color: var(--color-text-info);
}

.seq-status {
  font-size: 11px;
  font-weight: 500;
  background: var(--color-background-primary);
  color: var(--color-text-info);
  padding: 2px 8px;
  border-radius: var(--border-radius-md);
  white-space: nowrap;
}

.seq-num {
  font-size: 11px;
  color: var(--color-text-tertiary);
  min-width: 16px;
}

.type-badge {
  font-size: 11px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: var(--border-radius-md);
  background: var(--color-background-secondary);
  color: var(--color-text-secondary);
  white-space: nowrap;
}

.seq-row--active .type-badge {
  background: var(--color-background-success);
  color: var(--color-text-success);
}

.seq-title { flex: 1; }

.seq-nav {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.seq-nav button {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  padding: 4px 10px;
}

/* ── 수행평가 카드 ── */
.card--assess { }

.assess-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.assess-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 6px;
}

.assess-chip {
  font-size: 12px;
  color: var(--color-text-secondary);
  background: var(--color-background-secondary);
  padding: 2px 8px;
  border-radius: var(--border-radius-md);
}

.btn-session {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  font-weight: 500;
  padding: 5px 12px;
  background: none;
  border: 0.5px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md);
  color: var(--color-text-primary);
  text-decoration: none;
  white-space: nowrap;
  cursor: pointer;
}
.btn-session:hover { background: var(--color-background-secondary); }

/* ── 학생 목록 ── */
.student-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 70vh;
  overflow-y: auto;
}

.student-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-size: 13px;
  padding: 4px 0;
  border-bottom: 0.5px solid var(--color-border-tertiary);
}

.student-row:last-child { border-bottom: none; }

.s-name { font-weight: 500; }
.s-no { font-size: 12px; color: var(--color-text-tertiary); }

/* ── 공통 상태 ── */
.center-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  font-size: 13px;
  color: var(--color-text-secondary);
  padding: 4rem;
}

.error-state { color: var(--color-text-danger); }

.empty-note {
  font-size: 13px;
  color: var(--color-text-tertiary);
  padding: 8px 0;
}

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
