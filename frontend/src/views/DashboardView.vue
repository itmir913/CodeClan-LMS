<template>
  <div class="layout">
    <!-- 사이드바 -->
    <aside class="sidebar">
      <div class="sidebar-logo">
        <IconSchool :size="18" stroke-width="1.5" />
        <span>{{ auth.schoolName || 'CodeClan LMS' }}</span>
      </div>

      <nav class="sidebar-nav">
        <a href="#" class="nav-item active">
          <IconLayoutDashboard :size="16" stroke-width="1.5" />
          대시보드
        </a>

        <div class="nav-group-label">수업</div>
        <RouterLink :to="{ name: 'session-management' }" class="nav-item">
          <IconPlayerPlay :size="16" stroke-width="1.5" />
          시험 세션 운영
        </RouterLink>

        <div class="nav-group-label">준비</div>
        <RouterLink :to="{ name: 'problem-bank' }" class="nav-item">
          <IconDatabase :size="16" stroke-width="1.5" />
          문제 은행
        </RouterLink>
        <RouterLink :to="{ name: 'lesson-management' }" class="nav-item">
          <IconList :size="16" stroke-width="1.5" />
          차시 관리
        </RouterLink>
        <RouterLink :to="{ name: 'assessment-management' }" class="nav-item">
          <IconFileText :size="16" stroke-width="1.5" />
          수행평가
        </RouterLink>

        <div class="nav-group-label">관리</div>
        <RouterLink :to="{ name: 'division-management' }" class="nav-item">
          <IconUsers :size="16" stroke-width="1.5" />
          학생/반 관리
        </RouterLink>
        <a href="#" class="nav-item">
          <IconHistory :size="16" stroke-width="1.5" />
          감사 로그
        </a>
        <a href="#" class="nav-item">
          <IconCloudDownload :size="16" stroke-width="1.5" />
          백업/시스템
        </a>
        <RouterLink v-if="auth.isAdmin" :to="{ name: 'teacher-management' }" class="nav-item">
          <IconUserCog :size="16" stroke-width="1.5" />
          교사 계정
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

    <!-- 메인 콘텐츠 -->
    <main class="main-content">
      <!-- 로딩 -->
      <div v-if="dashboard.loading" class="center-msg">
        <div class="spinner"></div>
        <span>불러오는 중...</span>
      </div>

      <!-- 에러 -->
      <div v-else-if="dashboard.error" class="error-banner">
        <IconAlertTriangle :size="16" />
        {{ dashboard.error }}
        <button class="retry-btn" @click="dashboard.fetch()">다시 시도</button>
      </div>

      <template v-else-if="dashboard.data">
        <!-- 헤더 -->
        <div class="page-header">
          <div class="greeting">안녕하세요, {{ dashboard.data.teacher_name }} 님</div>
          <div class="sub-info">
            담당 분반 {{ dashboard.data.division_count }}개 ·
            문제 {{ dashboard.data.stats.problem_count }}건 ·
            수행평가 {{ dashboard.data.stats.assessment_count }}건
          </div>
        </div>

        <!-- 내 분반 -->
        <section class="section">
          <div class="section-title">
            <IconPlayerPlay :size="16" class="icon-success" />
            <span>내 분반 — 수업 운영</span>
            <span class="section-hint">분반에 입장해 차시를 설명하거나 수행평가 세션을 진행합니다</span>
          </div>

          <div v-if="dashboard.data.divisions.length === 0" class="empty-state">
            담당 분반이 없습니다. 관리자에게 분반 배정을 요청하세요.
          </div>

          <div v-else class="division-grid">
            <div
              v-for="div in dashboard.data.divisions"
              :key="div.id"
              class="division-card"
              :class="{ 'card-running': div.active_session?.status === 'RUNNING', 'card-lobby': div.active_session?.status === 'LOBBY' }"
            >
              <div class="card-top">
                <span class="div-name">{{ div.name }}</span>
                <span
                  v-if="div.active_session"
                  class="session-badge"
                  :class="div.active_session.status === 'RUNNING' ? 'badge-running' : 'badge-lobby'"
                >
                  <IconAlertTriangle v-if="div.active_session.status === 'RUNNING'" :size="12" />
                  <IconDoor v-else :size="12" />
                  {{ div.active_session.status === 'RUNNING' ? '세션 운영중' : '대기실' }}
                </span>
                <span v-else class="no-session-badge">
                  학생 {{ div.student_count }}명
                </span>
              </div>

              <div v-if="div.active_session" class="session-info">
                {{ div.active_session.assessment_title }} ·
                {{ div.active_session.submission_count }}/{{ div.active_session.student_count }} 제출
              </div>
              <div v-else class="no-session-info">
                진행 중인 세션 없음
              </div>

              <div class="card-actions">
                <button v-if="div.active_session" class="btn-primary">
                  <IconArrowRight :size="13" />
                  세션 운영 화면
                </button>
                <template v-else>
                  <button class="btn-secondary">
                    <IconPresentation :size="13" />
                    차시 운영
                  </button>
                  <button class="btn-secondary">
                    <IconPlayerPlay :size="13" />
                    수행평가 세션
                  </button>
                </template>
              </div>
            </div>
          </div>
        </section>

        <!-- 수업 준비 -->
        <section class="section">
          <div class="section-title">
            <IconEdit :size="16" class="icon-secondary" />
            <span>수업 준비</span>
            <span class="section-hint">차시·문항·수행평가는 수업 전에 미리 작성/배정해 둡니다</span>
          </div>
          <div class="prep-grid">
            <div class="prep-card">
              <IconDatabase :size="22" class="icon-secondary" />
              <div class="prep-title">문제 은행</div>
              <div class="prep-desc">전체 {{ dashboard.data.stats.problem_count }}문항</div>
            </div>
            <div class="prep-card">
              <IconList :size="22" class="icon-secondary" />
              <div class="prep-title">차시 관리</div>
              <div class="prep-desc">총 {{ dashboard.data.stats.lesson_count }}차시</div>
            </div>
            <div class="prep-card">
              <IconFileText :size="22" class="icon-secondary" />
              <div class="prep-title">수행평가 관리</div>
              <div class="prep-desc">전체 {{ dashboard.data.stats.assessment_count }}건</div>
            </div>
            <div class="prep-card">
              <IconUsers :size="22" class="icon-secondary" />
              <div class="prep-title">학생/반 관리</div>
              <div class="prep-desc">명부·분반 설정</div>
            </div>
          </div>
        </section>

        <!-- 최근 활동 -->
        <section v-if="dashboard.data.recent_logs.length > 0" class="section">
          <div class="section-title">
            <IconHistory :size="16" class="icon-secondary" />
            <span>최근 활동</span>
          </div>
          <div class="log-list">
            <div v-for="log in dashboard.data.recent_logs" :key="log.id" class="log-item">
              <span class="log-time">{{ formatTime(log.created_at) }}</span>
              <span class="log-detail">{{ log.detail ?? log.action_type }}</span>
            </div>
          </div>
        </section>
      </template>
    </main>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useDashboardStore } from '@/stores/dashboard'
import {
  IconSchool,
  IconLayoutDashboard,
  IconPlayerPlay,
  IconDatabase,
  IconList,
  IconFileText,
  IconUsers,
  IconHistory,
  IconCloudDownload,
  IconUserCog,
  IconAlertTriangle,
  IconEdit,
  IconArrowRight,
  IconPresentation,
  IconDoor,
} from '@tabler/icons-vue'

const router = useRouter()
const auth = useAuthStore()
const dashboard = useDashboardStore()

onMounted(async () => {
  if (!auth.isTeacherLoggedIn) {
    await auth.fetchTeacherMe()
    if (!auth.isTeacherLoggedIn) {
      router.replace({ name: 'login' })
      return
    }
  }
  await auth.fetchSchoolName()
  await dashboard.fetch()
})

async function logout() {
  await auth.logoutTeacher()
  router.replace({ name: 'login' })
}

function formatTime(iso: string): string {
  const d = new Date(iso)
  const now = new Date()
  const diffMs = now.getTime() - d.getTime()
  const diffHours = diffMs / (1000 * 60 * 60)
  if (diffHours < 24) return d.toLocaleTimeString('ko-KR', { hour: '2-digit', minute: '2-digit' })
  if (diffHours < 48) return '어제'
  return d.toLocaleDateString('ko-KR', { month: 'short', day: 'numeric' })
}
</script>

<style scoped>
.layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

/* ── 사이드바 ── */
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
  color: var(--color-text-primary);
}

.sidebar-nav {
  flex: 1;
  padding: 8px 0;
  overflow-y: auto;
}

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
  transition: background 0.1s, color 0.1s;
}

.nav-item:hover {
  background: var(--color-background-secondary);
  color: var(--color-text-primary);
}

.nav-item.active {
  background: var(--color-background-info);
  color: var(--color-text-info);
  font-weight: 500;
}

.sidebar-footer {
  padding: 12px 16px;
  border-top: 1px solid var(--color-border-secondary);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.user-name { font-size: 13px; font-weight: 500; }

.user-role {
  font-size: 10px;
  background: var(--color-background-info);
  color: var(--color-text-info);
  padding: 2px 6px;
  border-radius: 4px;
}

.logout-btn {
  font-size: 12px;
  color: var(--color-text-secondary);
  padding: 4px 0;
  border: none;
  background: none;
  cursor: pointer;
  text-align: left;
}
.logout-btn:hover { color: var(--color-text-primary); }

/* ── 메인 콘텐츠 ── */
.main-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  background: var(--color-background-secondary);
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.center-msg {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--color-text-secondary);
  font-size: 13px;
  padding: 40px 0;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--color-border-primary);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

.error-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--color-background-danger);
  color: var(--color-text-danger);
  padding: 12px 16px;
  border-radius: var(--border-radius-md);
  font-size: 13px;
}

.retry-btn {
  margin-left: auto;
  font-size: 12px;
  color: var(--color-text-danger);
  text-decoration: underline;
  background: none;
  border: none;
  cursor: pointer;
}

.page-header { }
.greeting { font-weight: 600; font-size: 17px; margin-bottom: 4px; }
.sub-info { font-size: 12px; color: var(--color-text-secondary); }

/* ── 섹션 ── */
.section {
  background: var(--color-background-primary);
  border: 1px solid var(--color-border-secondary);
  border-radius: var(--border-radius-lg);
  padding: 18px 20px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 14px;
  font-weight: 500;
  font-size: 14px;
}

.section-hint {
  font-size: 11px;
  color: var(--color-text-tertiary);
  font-weight: normal;
}

.icon-success { color: var(--color-text-success, #16a34a); }
.icon-secondary { color: var(--color-text-secondary); }

/* ── 분반 카드 ── */
.empty-state {
  font-size: 13px;
  color: var(--color-text-tertiary);
  padding: 16px 0;
}

.division-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 12px;
}

.division-card {
  border: 1px solid var(--color-border-secondary);
  border-radius: var(--border-radius-lg);
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  background: var(--color-background-primary);
  transition: box-shadow 0.15s;
}

.division-card:hover { box-shadow: 0 2px 8px rgba(0,0,0,0.06); }

.card-running {
  border: 2px solid var(--color-border-danger);
  background: var(--color-background-danger);
}

.card-lobby {
  border: 2px solid var(--color-border-info, var(--color-accent));
  background: var(--color-background-info);
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}

.div-name { font-weight: 600; font-size: 15px; }

.session-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: var(--border-radius-md);
  background: var(--color-background-primary);
}

.badge-running { color: var(--color-text-danger); }
.badge-lobby { color: var(--color-text-info); }

.no-session-badge {
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.session-info { font-size: 12px; color: var(--color-text-danger); }
.no-session-info { font-size: 12px; color: var(--color-text-tertiary); }

.card-actions {
  display: flex;
  gap: 6px;
  margin-top: 4px;
}

.btn-primary {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  font-weight: 500;
  padding: 5px 12px;
  background: var(--color-accent);
  color: #fff;
  border-color: transparent;
}

.btn-primary:hover {
  background: var(--color-accent-hover);
}

.btn-secondary {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  padding: 5px 10px;
}

/* ── 수업 준비 그리드 ── */
.prep-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

@media (max-width: 1100px) {
  .prep-grid { grid-template-columns: repeat(2, 1fr); }
}

.prep-card {
  border: 1px solid var(--color-border-secondary);
  border-radius: var(--border-radius-lg);
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  cursor: pointer;
  transition: background 0.1s, box-shadow 0.1s;
}

.prep-card:hover {
  background: var(--color-background-secondary);
  box-shadow: 0 1px 4px rgba(0,0,0,0.05);
}

.prep-title { font-weight: 500; font-size: 13px; }
.prep-desc { font-size: 11px; color: var(--color-text-secondary); }

/* ── 최근 활동 ── */
.log-list { display: flex; flex-direction: column; gap: 8px; }

.log-item {
  display: flex;
  gap: 12px;
  font-size: 12px;
  align-items: flex-start;
}

.log-time { color: var(--color-text-tertiary); min-width: 44px; flex-shrink: 0; }
.log-detail { color: var(--color-text-secondary); }
</style>
