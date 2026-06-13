<template>
  <div class="page-bg">
    <!-- 로딩 -->
    <div v-if="initialLoading" class="center-state">
      <div class="spinner" />
      <span>불러오는 중...</span>
    </div>

    <!-- 에러 -->
    <div v-else-if="initError" class="center-state error-state">
      <span>{{ initError }}</span>
      <button class="btn-secondary" @click="init">다시 시도</button>
    </div>

    <!-- 시험 모드 (RUNNING) -->
    <StudentExamView
      v-else-if="store.activeSession?.status === 'RUNNING'"
      :session="store.activeSession"
      :student-name="auth.student?.name ?? ''"
      @logout="logout"
    />

    <!-- 일반 화면 -->
    <template v-else>
      <!-- LOBBY 배너 -->
      <div v-if="store.activeSession?.status === 'LOBBY'" class="lobby-banner">
        <span class="lobby-icon">📋</span>
        <span>수행평가 대기 중 — <strong>{{ store.activeSession.assessment_title }}</strong>. 선생님의 시작 안내를 기다려주세요.</span>
      </div>

      <div class="container">
        <!-- 헤더 -->
        <header class="header">
          <div class="header-left">
            <span class="school-name">{{ auth.schoolName }}</span>
          </div>
          <div class="header-right">
            <div class="user-info">
              <span class="user-name">{{ auth.student?.name }}</span>
              <span class="user-division">{{ auth.student?.division_name }}</span>
            </div>
            <button class="btn-ghost" @click="logout">로그아웃</button>
          </div>
        </header>

        <!-- 에러 배너 -->
        <div v-if="store.error" class="error-banner">{{ store.error }}</div>

        <!-- 탭 -->
        <nav class="tabs">
          <button
            class="tab-btn"
            :class="{ 'tab-btn--active': activeTab === 'lessons' }"
            @click="activeTab = 'lessons'"
          >
            수업 (차시)
          </button>
          <button
            class="tab-btn"
            :class="{ 'tab-btn--active': activeTab === 'assessments' }"
            @click="activeTab = 'assessments'"
          >
            수행평가
          </button>
        </nav>

        <!-- 로딩 스피너 (탭 콘텐츠) -->
        <div v-if="store.loading" class="tab-loading">
          <div class="spinner" />
        </div>

        <!-- 수업 탭 -->
        <section v-else-if="activeTab === 'lessons'" class="tab-content">
          <div v-if="store.lessons.length === 0" class="empty-state">
            공개된 수업이 없습니다.
          </div>
          <ul v-else class="lesson-list">
            <li
              v-for="lesson in store.lessons"
              :key="lesson.id"
              class="lesson-item"
            >
              <div class="lesson-icon">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>
                </svg>
              </div>
              <div class="lesson-info">
                <span class="lesson-title">{{ lesson.title }}</span>
                <span class="lesson-desc" v-if="lesson.description">{{ lesson.description }}</span>
              </div>
              <span class="lesson-meta">{{ lesson.problem_count }}문항</span>
            </li>
          </ul>
        </section>

        <!-- 수행평가 탭 -->
        <section v-else-if="activeTab === 'assessments'" class="tab-content">
          <div v-if="store.assessments.length === 0" class="empty-state">
            연결된 수행평가가 없습니다.
          </div>

          <template v-else>
            <!-- 예정된 수행평가 -->
            <div v-if="upcomingAssessments.length > 0" class="assess-section">
              <div class="assess-section-title">예정된 수행평가</div>
              <ul class="assessment-list">
                <li
                  v-for="a in upcomingAssessments"
                  :key="a.id"
                  class="assessment-item"
                >
                  <div class="assessment-icon assessment-icon--upcoming">
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
                  </div>
                  <div class="assessment-info">
                    <span class="assessment-title">{{ a.title }}</span>
                    <span class="assessment-sub" v-if="a.description">{{ a.description }}</span>
                    <span class="assessment-meta">{{ a.problem_count }}문항</span>
                  </div>
                  <span
                    v-if="a.session_status"
                    class="badge"
                    :class="sessionBadgeClass(a.session_status)"
                  >{{ sessionStatusLabel(a.session_status) }}</span>
                  <span v-else class="badge badge--neutral">안내</span>
                </li>
              </ul>
            </div>

            <!-- 지난 수행평가 결과 -->
            <div v-if="pastAssessments.length > 0" class="assess-section">
              <div class="assess-section-title">지난 수행평가 결과</div>
              <ul class="assessment-list">
                <li
                  v-for="a in pastAssessments"
                  :key="a.id"
                  class="assessment-item"
                >
                  <div class="assessment-icon" :class="a.is_result_released ? 'assessment-icon--released' : 'assessment-icon--closed'">
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
                  </div>
                  <div class="assessment-info">
                    <span class="assessment-title">{{ a.title }}</span>
                    <span class="assessment-sub" v-if="a.description">{{ a.description }}</span>
                    <span class="assessment-meta">{{ a.problem_count }}문항</span>
                  </div>

                  <!-- 결과 미공개 -->
                  <span v-if="!a.is_result_released" class="badge badge--neutral">
                    결과 미공개
                  </span>

                  <!-- 결과 공개: 점수 + 상세 링크 -->
                  <div v-else class="result-area">
                    <div class="result-score">
                      <span class="result-score-mine">{{ a.my_score ?? 0 }}</span>
                      <span class="result-score-sep"> / </span>
                      <span class="result-score-max">{{ a.total_max_score }}점</span>
                    </div>
                    <RouterLink
                      v-if="a.session_id"
                      :to="{ name: 'student-result', params: { sessionId: a.session_id } }"
                      class="result-detail-link"
                    >
                      상세 결과 보기 →
                    </RouterLink>
                  </div>
                </li>
              </ul>
            </div>
          </template>
        </section>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { RouterLink } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useStudentStore } from '@/stores/student'
import { api } from '@/api/client'
import StudentExamView from './StudentExamView.vue'

const router = useRouter()
const auth = useAuthStore()
const store = useStudentStore()

const activeTab = ref<'lessons' | 'assessments'>('lessons')
const initialLoading = ref(false)
const initError = ref<string | null>(null)
let pollTimer: ReturnType<typeof setInterval> | null = null
let heartbeatTimer: ReturnType<typeof setInterval> | null = null

const upcomingAssessments = computed(() =>
  store.assessments.filter(a => a.session_status !== 'CLOSED')
)
const pastAssessments = computed(() =>
  store.assessments.filter(a => a.session_status === 'CLOSED')
)

async function init() {
  initialLoading.value = true
  initError.value = null
  try {
    if (!auth.isStudentLoggedIn) {
      await auth.fetchStudentMe()
      if (!auth.isStudentLoggedIn) {
        router.replace({ name: 'login' })
        return
      }
    }
    if (auth.student?.password_reset_required) {
      router.replace({ name: 'student-change-password' })
      return
    }
    await auth.fetchSchoolName()
    await store.loadAll()
  } catch (e) {
    initError.value = e instanceof Error ? e.message : '오류가 발생했습니다'
  } finally {
    initialLoading.value = false
  }
}

function startPolling() {
  pollTimer = setInterval(() => {
    store.fetchActiveSession()
  }, 5000)

  heartbeatTimer = setInterval(() => {
    const s = store.activeSession
    if (s && (s.status === 'LOBBY' || s.status === 'RUNNING')) {
      api.attendance.heartbeat('session', s.id).catch(() => {})
    }
  }, 15000)
}

onMounted(async () => {
  await init()
  startPolling()
})

onUnmounted(() => {
  if (pollTimer !== null) clearInterval(pollTimer)
  if (heartbeatTimer !== null) clearInterval(heartbeatTimer)
})

async function logout() {
  if (pollTimer !== null) clearInterval(pollTimer)
  if (heartbeatTimer !== null) clearInterval(heartbeatTimer)
  await auth.logoutStudent()
  router.replace({ name: 'login' })
}

type SessionStatus = 'CREATED' | 'LOBBY' | 'RUNNING' | 'CLOSED'

function sessionStatusLabel(status: SessionStatus): string {
  const labels: Record<SessionStatus, string> = {
    CREATED: '예정',
    LOBBY: '대기중',
    RUNNING: '진행중',
    CLOSED: '종료',
  }
  return labels[status] ?? status
}

function sessionBadgeClass(status: SessionStatus): string {
  const map: Record<SessionStatus, string> = {
    CREATED: 'badge--neutral',
    LOBBY: 'badge--warning',
    RUNNING: 'badge--danger',
    CLOSED: 'badge--secondary',
  }
  return map[status] ?? 'badge--neutral'
}
</script>

<style scoped>
.page-bg {
  min-height: 100vh;
  background: var(--color-background-secondary);
}

/* ── 공통 상태 ─────────────────────────────────────── */

.center-state {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.error-state {
  color: var(--color-text-danger);
}

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--color-border-secondary);
  border-top-color: var(--color-text-info);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* ── LOBBY 배너 ────────────────────────────────────── */

.lobby-banner {
  background: var(--color-background-warning);
  color: var(--color-text-warning);
  padding: 10px 1.5rem;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  border-bottom: 0.5px solid var(--color-border-secondary);
}

.lobby-icon {
  font-size: 15px;
}

/* ── 일반 레이아웃 ──────────────────────────────────── */

.container {
  max-width: 760px;
  margin: 0 auto;
  padding: 1.5rem;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 1rem;
  border-bottom: 0.5px solid var(--color-border-secondary);
  margin-bottom: 1.25rem;
}

.school-name {
  font-weight: 600;
  font-size: 15px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 14px;
}

.user-info {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 1px;
}

.user-name {
  font-size: 13px;
  font-weight: 500;
}

.user-division {
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.error-banner {
  background: var(--color-background-danger);
  color: var(--color-text-danger);
  font-size: 12px;
  padding: 8px 12px;
  border-radius: var(--border-radius-md);
  margin-bottom: 1rem;
}

/* ── 탭 ────────────────────────────────────────────── */

.tabs {
  display: flex;
  border-bottom: 0.5px solid var(--color-border-secondary);
  margin-bottom: 1.25rem;
}

.tab-btn {
  padding: 8px 16px;
  font-size: 13px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  margin-bottom: -0.5px;
  transition: color 0.15s, border-color 0.15s;
}

.tab-btn:hover {
  color: var(--color-text-primary);
}

.tab-btn--active {
  color: var(--color-text-info);
  border-bottom-color: var(--color-border-info);
  font-weight: 500;
}

.tab-loading {
  display: flex;
  justify-content: center;
  padding: 3rem;
}

.tab-content {
  list-style: none;
  margin: 0;
  padding: 0;
}

.empty-state {
  text-align: center;
  color: var(--color-text-tertiary);
  font-size: 13px;
  padding: 3rem 0;
}

/* ── 수업 목록 ──────────────────────────────────────── */

.lesson-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.lesson-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-md);
}

.lesson-icon {
  color: var(--color-text-info);
  flex-shrink: 0;
  display: flex;
  align-items: center;
}

.lesson-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.lesson-title {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.lesson-desc {
  font-size: 11px;
  color: var(--color-text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.lesson-meta {
  font-size: 12px;
  color: var(--color-text-tertiary);
  flex-shrink: 0;
}

/* ── 수행평가 목록 ─────────────────────────────────── */

.assess-section { margin-bottom: 1.5rem; }

.assess-section-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.assessment-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.assessment-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-md);
}

.assessment-icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
}

.assessment-icon--upcoming { color: var(--color-text-info); }
.assessment-icon--released { color: var(--color-text-success); }
.assessment-icon--closed   { color: var(--color-text-tertiary); }

.result-area {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
  flex-shrink: 0;
}

.result-score {
  display: flex;
  align-items: baseline;
}

.result-score-mine {
  font-size: 16px;
  font-weight: 700;
  color: var(--color-text-info);
}

.result-score-sep { font-size: 12px; color: var(--color-text-tertiary); margin: 0 2px; }
.result-score-max { font-size: 12px; color: var(--color-text-secondary); }

.result-detail-link {
  font-size: 12px;
  color: var(--color-text-info);
  text-decoration: none;
}
.result-detail-link:hover { text-decoration: underline; }

.assessment-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.assessment-title {
  font-size: 13px;
  font-weight: 500;
}

.assessment-sub {
  font-size: 11px;
  color: var(--color-text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.assessment-meta {
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.assessment-badges {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

/* ── 배지 ────────────────────────────────────────────── */

.badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: var(--border-radius-md);
  font-weight: 500;
  white-space: nowrap;
}

.badge--neutral {
  background: var(--color-background-secondary);
  color: var(--color-text-tertiary);
  border: 0.5px solid var(--color-border-tertiary);
}

.badge--secondary {
  background: var(--color-background-secondary);
  color: var(--color-text-secondary);
  border: 0.5px solid var(--color-border-secondary);
}

.badge--warning {
  background: var(--color-background-warning);
  color: var(--color-text-warning);
}

.badge--danger {
  background: var(--color-background-danger);
  color: var(--color-text-danger);
}

.badge--info {
  background: var(--color-background-info);
  color: var(--color-text-info);
}

/* ── 버튼 ─────────────────────────────────────────── */

.btn-ghost {
  font-size: 12px;
  color: var(--color-text-secondary);
  border: none;
  background: none;
  cursor: pointer;
  padding: 4px 8px;
}

.btn-ghost:hover {
  color: var(--color-text-primary);
}

.btn-secondary {
  font-size: 12px;
  padding: 6px 14px;
  border: 0.5px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md);
  background: var(--color-background-primary);
  cursor: pointer;
}
</style>
