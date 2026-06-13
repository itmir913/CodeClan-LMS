<template>
  <aside class="app-sidebar">
    <div class="sidebar-logo">
      <IconSchool :size="18" stroke-width="1.5" />
      <span>{{ auth.schoolName || 'CodeClan LMS' }}</span>
    </div>

    <nav class="sidebar-nav">
      <!-- 대시보드 -->
      <RouterLink :to="{ name: 'dashboard' }" class="nav-item" active-class="nav-item--active">
        <IconLayoutDashboard :size="16" stroke-width="1.5" />
        대시보드
      </RouterLink>

      <!-- 수업 그룹 -->
      <div class="nav-group-label">수업</div>
      <RouterLink :to="{ name: 'session-management' }" class="nav-item" active-class="nav-item--active">
        <IconPlayerPlay :size="16" stroke-width="1.5" />
        시험 세션 운영
      </RouterLink>

      <!-- 준비 그룹 -->
      <div class="nav-group-label">준비</div>
      <RouterLink :to="{ name: 'problem-bank' }" class="nav-item" active-class="nav-item--active">
        <IconDatabase :size="16" stroke-width="1.5" />
        문제 은행
      </RouterLink>
      <RouterLink :to="{ name: 'lesson-management' }" class="nav-item" active-class="nav-item--active">
        <IconList :size="16" stroke-width="1.5" />
        차시 관리
      </RouterLink>
      <RouterLink :to="{ name: 'assessment-management' }" class="nav-item" active-class="nav-item--active">
        <IconFileText :size="16" stroke-width="1.5" />
        수행평가
      </RouterLink>

      <!-- 관리 그룹 -->
      <div class="nav-group-label">관리</div>
      <RouterLink :to="{ name: 'division-management' }" class="nav-item" active-class="nav-item--active">
        <IconUsers :size="16" stroke-width="1.5" />
        학생/반 관리
      </RouterLink>
      <RouterLink v-if="auth.isAdmin" :to="{ name: 'teacher-management' }" class="nav-item" active-class="nav-item--active">
        <IconUserCog :size="16" stroke-width="1.5" />
        교사 계정
      </RouterLink>
      <RouterLink :to="{ name: 'audit-logs' }" class="nav-item" active-class="nav-item--active">
        <IconClipboardList :size="16" stroke-width="1.5" />
        감사 로그
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
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  IconSchool, IconLayoutDashboard, IconPlayerPlay,
  IconDatabase, IconList, IconFileText,
  IconUsers, IconUserCog, IconClipboardList,
} from '@tabler/icons-vue'

const router = useRouter()
const auth = useAuthStore()

async function logout() {
  await auth.logout()
  router.replace({ name: 'login' })
}
</script>

<style scoped>
.app-sidebar {
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

.nav-item--active {
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
  border-radius: var(--border-radius-md);
}

.logout-btn {
  background: none;
  border: none;
  color: var(--color-text-tertiary);
  font-size: 12px;
  cursor: pointer;
  text-align: left;
  padding: 0;
}
.logout-btn:hover { color: var(--color-text-danger); }
</style>
