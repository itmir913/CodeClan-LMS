<template>
  <div class="layout">
    <aside class="sidebar">
      <div class="sidebar-logo">
        <IconSchool :size="18" stroke-width="1.5" />
        <span>{{ auth.schoolName || 'CodeClan LMS' }}</span>
      </div>
      <nav class="sidebar-nav">
        <RouterLink :to="{ name: 'dashboard' }" class="nav-item">
          <IconLayoutDashboard :size="16" stroke-width="1.5" />대시보드
        </RouterLink>
        <div class="nav-group-label">관리</div>
        <RouterLink :to="{ name: 'division-management' }" class="nav-item">
          <IconUsers :size="16" stroke-width="1.5" />학생/반 관리
        </RouterLink>
        <a href="#" class="nav-item active">
          <IconUserCog :size="16" stroke-width="1.5" />교사 계정
        </a>
      </nav>
      <div class="sidebar-footer">
        <div class="user-info" v-if="auth.teacher">
          <span class="user-name">{{ auth.teacher.name }}</span>
          <span class="user-role">관리자</span>
        </div>
        <button class="logout-btn" @click="logout">로그아웃</button>
      </div>
    </aside>

    <main class="main-content">
      <div class="page-header">
        <span class="page-title">교사 계정 관리</span>
        <button class="btn-primary" @click="openCreateModal">
          <IconPlus :size="13" />교사 추가
        </button>
      </div>

      <div v-if="teacher.loading && teacher.teachers.length === 0" class="center-msg">
        <div class="spinner"></div>불러오는 중...
      </div>
      <div v-else-if="teacher.error" class="error-banner">
        <IconAlertTriangle :size="14" />{{ teacher.error }}
      </div>

      <div v-else class="teacher-table-wrap">
        <table class="teacher-table">
          <thead>
            <tr>
              <th>이름</th>
              <th>아이디</th>
              <th>권한</th>
              <th>담당 분반</th>
              <th>등록일</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="t in teacher.teachers" :key="t.id">
              <td class="teacher-name">{{ t.name }}</td>
              <td class="mono">{{ t.username }}</td>
              <td>
                <span class="badge" :class="t.role === 'admin' ? 'badge-admin' : 'badge-teacher'">
                  {{ t.role === 'admin' ? '관리자' : '일반 교사' }}
                </span>
              </td>
              <td>{{ t.division_count }}개</td>
              <td class="date-col">{{ formatDate(t.created_at) }}</td>
              <td class="action-col">
                <button class="btn-icon-sm" title="수정" @click="openEditModal(t)">
                  <IconPencil :size="13" />
                </button>
                <button
                  class="btn-icon-sm btn-danger"
                  title="삭제"
                  :disabled="t.id === auth.teacher?.id"
                  @click="confirmDelete(t)"
                >
                  <IconTrash :size="13" />
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- 생성 모달 -->
      <div v-if="modal.open && modal.mode === 'create'" class="modal-backdrop" @click.self="closeModal">
        <div class="modal">
          <div class="modal-title">교사 추가</div>
          <label>이름</label>
          <input v-model="form.name" type="text" placeholder="홍길동" />
          <label>아이디</label>
          <input v-model="form.username" type="text" placeholder="hong" />
          <label>비밀번호</label>
          <input v-model="form.password" type="password" />
          <label>권한</label>
          <select v-model="form.role">
            <option value="teacher">일반 교사</option>
            <option value="admin">관리자</option>
          </select>
          <div class="modal-error" v-if="modalError">{{ modalError }}</div>
          <div class="modal-actions">
            <button @click="closeModal">취소</button>
            <button class="btn-primary" @click="submitCreate" :disabled="modal.loading">
              {{ modal.loading ? '추가 중...' : '추가' }}
            </button>
          </div>
        </div>
      </div>

      <!-- 수정 모달 -->
      <div v-if="modal.open && modal.mode === 'edit'" class="modal-backdrop" @click.self="closeModal">
        <div class="modal">
          <div class="modal-title">교사 정보 수정 — {{ modal.target?.name }}</div>
          <label>이름</label>
          <input v-model="form.name" type="text" />
          <label>권한</label>
          <select v-model="form.role">
            <option value="teacher">일반 교사</option>
            <option value="admin">관리자</option>
          </select>
          <label>새 비밀번호 (변경 시만 입력)</label>
          <input v-model="form.password" type="password" placeholder="변경하지 않으면 비워두세요" />
          <div class="modal-error" v-if="modalError">{{ modalError }}</div>
          <div class="modal-actions">
            <button @click="closeModal">취소</button>
            <button class="btn-primary" @click="submitEdit" :disabled="modal.loading">저장</button>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useTeacherStore } from '@/stores/teacher'
import type { TeacherRow } from '@/api/client'
import {
  IconSchool, IconLayoutDashboard, IconUsers, IconUserCog,
  IconPlus, IconPencil, IconTrash, IconAlertTriangle,
} from '@tabler/icons-vue'

const router = useRouter()
const auth = useAuthStore()
const teacher = useTeacherStore()

const modal = reactive({
  open: false,
  mode: 'create' as 'create' | 'edit',
  target: null as TeacherRow | null,
  loading: false,
})

const form = reactive({ name: '', username: '', password: '', role: 'teacher' })
const modalError = ref<string | null>(null)

function closeModal() {
  modal.open = false
  modal.target = null
  modal.loading = false
  modalError.value = null
}

function openCreateModal() {
  form.name = ''; form.username = ''; form.password = ''; form.role = 'teacher'
  modal.mode = 'create'
  modal.open = true
}

function openEditModal(t: TeacherRow) {
  form.name = t.name; form.username = t.username; form.password = ''; form.role = t.role
  modal.target = t
  modal.mode = 'edit'
  modal.open = true
}

async function submitCreate() {
  if (!form.name.trim() || !form.username.trim() || !form.password) {
    modalError.value = '이름, 아이디, 비밀번호를 모두 입력하세요'
    return
  }
  modal.loading = true; modalError.value = null
  try {
    await teacher.createTeacher({ name: form.name.trim(), username: form.username.trim(), password: form.password, role: form.role })
    closeModal()
  } catch (e) {
    modalError.value = e instanceof Error ? e.message : '오류가 발생했습니다'
  } finally {
    modal.loading = false
  }
}

async function submitEdit() {
  modal.loading = true; modalError.value = null
  try {
    await teacher.updateTeacher(modal.target!.id, {
      name: form.name.trim() || undefined,
      role: form.role || undefined,
      password: form.password || undefined,
    })
    closeModal()
  } catch (e) {
    modalError.value = e instanceof Error ? e.message : '오류가 발생했습니다'
  } finally {
    modal.loading = false
  }
}

async function confirmDelete(t: TeacherRow) {
  if (!confirm(`"${t.name}" 계정을 삭제하시겠습니까?`)) return
  try {
    await teacher.deleteTeacher(t.id)
  } catch (e) {
    alert(e instanceof Error ? e.message : '삭제 실패')
  }
}

async function logout() {
  await auth.logoutTeacher()
  router.replace({ name: 'login' })
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString('ko-KR', { year: 'numeric', month: 'short', day: 'numeric' })
}

onMounted(async () => {
  if (!auth.isTeacherLoggedIn) {
    await auth.fetchTeacherMe()
    if (!auth.isTeacherLoggedIn) { router.replace({ name: 'login' }); return }
  }
  if (!auth.isAdmin) { router.replace({ name: 'dashboard' }); return }
  await teacher.fetchTeachers()
})
</script>

<style scoped>
.layout { display: flex; height: 100vh; overflow: hidden; }

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
}

.sidebar-nav { flex: 1; padding: 8px 0; }

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
  transition: background 0.1s;
}

.nav-item:hover { background: var(--color-background-secondary); color: var(--color-text-primary); }
.nav-item.active { background: var(--color-background-info); color: var(--color-text-info); font-weight: 500; }

.sidebar-footer {
  padding: 12px 16px;
  border-top: 1px solid var(--color-border-secondary);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.user-info { display: flex; align-items: center; gap: 8px; }
.user-name { font-size: 13px; font-weight: 500; }
.user-role { font-size: 10px; background: var(--color-background-info); color: var(--color-text-info); padding: 2px 6px; border-radius: 4px; }
.logout-btn { font-size: 12px; color: var(--color-text-secondary); padding: 4px 0; border: none; background: none; cursor: pointer; text-align: left; }

/* ── 메인 ── */
.main-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  background: var(--color-background-secondary);
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.page-title { font-weight: 600; font-size: 16px; }

.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  padding: 7px 14px;
  background: var(--color-accent);
  color: #fff;
  border-color: transparent;
}

.btn-primary:hover { background: var(--color-accent-hover); }

.center-msg { display: flex; align-items: center; gap: 8px; font-size: 13px; color: var(--color-text-secondary); }

.error-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--color-background-danger);
  color: var(--color-text-danger);
  padding: 10px 14px;
  border-radius: var(--border-radius-md);
  font-size: 13px;
}

/* ── 테이블 ── */
.teacher-table-wrap {
  background: var(--color-background-primary);
  border: 1px solid var(--color-border-secondary);
  border-radius: var(--border-radius-lg);
  overflow: hidden;
}

.teacher-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.teacher-table th {
  padding: 10px 12px;
  text-align: left;
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-secondary);
  background: var(--color-background-secondary);
  border-bottom: 1px solid var(--color-border-secondary);
}

.teacher-table td {
  padding: 9px 12px;
  border-bottom: 1px solid var(--color-border-tertiary);
  vertical-align: middle;
}

.teacher-table tr:last-child td { border-bottom: none; }

.teacher-name { font-weight: 500; }
.mono { font-family: 'Courier New', monospace; font-size: 12px; }
.date-col { color: var(--color-text-tertiary); font-size: 11px; }

.action-col { display: flex; gap: 4px; justify-content: flex-end; }

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

.btn-icon-sm:disabled { opacity: 0.3; cursor: not-allowed; }
.btn-icon-sm:hover:not(:disabled) { background: var(--color-background-secondary); }
.btn-icon-sm.btn-danger { color: var(--color-text-danger); }
.btn-icon-sm.btn-danger:hover:not(:disabled) { background: var(--color-background-danger); }

.badge {
  display: inline-flex;
  align-items: center;
  font-size: 11px;
  padding: 2px 7px;
  border-radius: 4px;
  font-weight: 500;
}

.badge-admin { background: var(--color-background-danger); color: var(--color-text-danger); }
.badge-teacher { background: var(--color-background-info); color: var(--color-text-info); }

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--color-border-primary);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

/* ── 모달 ── */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  background: var(--color-background-primary);
  border-radius: var(--border-radius-lg);
  padding: 20px;
  width: 360px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
}

.modal-title { font-weight: 600; font-size: 15px; margin-bottom: 4px; }

.modal-error {
  font-size: 12px;
  color: var(--color-text-danger);
  background: var(--color-background-danger);
  padding: 8px 10px;
  border-radius: var(--border-radius-sm);
}

.modal-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }

select {
  display: block;
  width: 100%;
  padding: 7px 10px;
  border: 1px solid var(--color-border-primary);
  border-radius: var(--border-radius-sm);
  font-size: 13px;
  background: var(--color-background-primary);
  color: var(--color-text-primary);
}
</style>
