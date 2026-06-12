<template>
  <div class="layout">
    <aside class="sidebar">
      <div class="sidebar-logo">CodeClan LMS</div>

      <nav class="sidebar-nav">
        <a href="#" class="nav-item active">대시보드</a>

        <div class="nav-group-label">수업</div>
        <a href="#" class="nav-item">시험 세션 운영</a>

        <div class="nav-group-label">준비</div>
        <a href="#" class="nav-item">문제 은행</a>
        <a href="#" class="nav-item">차시 관리</a>
        <a href="#" class="nav-item">수행평가</a>

        <div class="nav-group-label">관리</div>
        <a href="#" class="nav-item">학생/반 관리</a>
        <a href="#" class="nav-item">감사 로그</a>
        <a href="#" class="nav-item">백업·시스템</a>
        <a href="#" class="nav-item">계정 설정</a>
      </nav>

      <div class="sidebar-footer">
        <div class="user-info" v-if="me">
          <span class="user-name">{{ me.name }}</span>
          <span class="user-role">{{ me.role === 'admin' ? '관리자' : '교사' }}</span>
        </div>
        <button class="logout-btn" @click="logout">로그아웃</button>
      </div>
    </aside>

    <main class="main-content">
      <div class="page-header">
        <h1 class="page-title">대시보드</h1>
      </div>

      <div class="placeholder-card">
        <p>담당 분반 현황이 여기에 표시됩니다.</p>
        <p style="font-size: 12px; color: var(--color-text-tertiary); margin-top: 4px;">
          차시 운영 및 수행평가 관리 기능을 순차적으로 구현 예정입니다.
        </p>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { api } from '@/api/client'

const router = useRouter()
const me = ref<{ id: number; username: string; name: string; role: string } | null>(null)

onMounted(async () => {
  try {
    me.value = await api.auth.me()
  } catch {
    router.replace({ name: 'login' })
  }
})

async function logout() {
  await api.auth.logout()
  router.replace({ name: 'login' })
}
</script>

<style scoped>
.layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.sidebar {
  width: 200px;
  flex-shrink: 0;
  background: var(--color-background-primary);
  border-right: 1px solid var(--color-border-secondary);
  display: flex;
  flex-direction: column;
}

.sidebar-logo {
  padding: 16px;
  font-weight: 700;
  font-size: 14px;
  border-bottom: 1px solid var(--color-border-secondary);
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
  display: block;
  padding: 7px 16px;
  font-size: 13px;
  color: var(--color-text-secondary);
  text-decoration: none;
  border-radius: 0;
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

.user-name {
  font-size: 13px;
  font-weight: 500;
}

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

.logout-btn:hover {
  color: var(--color-text-primary);
  background: none;
}

.main-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  background: var(--color-background-secondary);
}

.page-header {
  margin-bottom: 20px;
}

.page-title {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
}

.placeholder-card {
  background: var(--color-background-primary);
  border: 1px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md);
  padding: 24px;
  color: var(--color-text-secondary);
}
</style>
