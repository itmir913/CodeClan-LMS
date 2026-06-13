<template>
  <div class="layout">
    <!-- 사이드바 -->
    <aside class="sidebar">
      <div class="sidebar-logo">
        <IconSchool :size="18" stroke-width="1.5" />
        <span>{{ auth.schoolName || 'CodeClan LMS' }}</span>
      </div>
      <nav class="sidebar-nav">
        <RouterLink :to="{ name: 'dashboard' }" class="nav-item">
          <IconLayoutDashboard :size="16" stroke-width="1.5" />대시보드
        </RouterLink>
        <div class="nav-group-label">수업</div>
        <a href="#" class="nav-item"><IconPlayerPlay :size="16" stroke-width="1.5" />시험 세션 운영</a>
        <div class="nav-group-label">준비</div>
        <a href="#" class="nav-item"><IconDatabase :size="16" stroke-width="1.5" />문제 은행</a>
        <a href="#" class="nav-item"><IconList :size="16" stroke-width="1.5" />차시 관리</a>
        <a href="#" class="nav-item"><IconFileText :size="16" stroke-width="1.5" />수행평가</a>
        <div class="nav-group-label">관리</div>
        <a href="#" class="nav-item active"><IconUsers :size="16" stroke-width="1.5" />학생/반 관리</a>
        <a href="#" class="nav-item"><IconHistory :size="16" stroke-width="1.5" />감사 로그</a>
        <a href="#" class="nav-item"><IconCloudDownload :size="16" stroke-width="1.5" />백업/시스템</a>
        <RouterLink :to="{ name: 'teacher-management' }" v-if="auth.isAdmin" class="nav-item">
          <IconUserCog :size="16" stroke-width="1.5" />교사 계정
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

    <!-- 메인: 좌(분반 목록) + 우(학생 상세) -->
    <div class="main-split">
      <!-- 좌: 분반 목록 -->
      <div class="list-panel">
        <div class="panel-header">
          <span class="panel-title">분반 목록</span>
          <button v-if="auth.isAdmin" class="btn-icon" @click="openCreateDivisionModal" title="분반 추가">
            <IconPlus :size="15" />
          </button>
        </div>

        <div v-if="div.loading && div.divisions.length === 0" class="panel-loading">
          <div class="spinner"></div>
        </div>
        <div v-else-if="div.error" class="panel-error">{{ div.error }}</div>
        <div v-else-if="div.divisions.length === 0" class="panel-empty">분반이 없습니다</div>

        <button
          v-for="d in div.divisions"
          :key="d.id"
          class="division-item"
          :class="{ active: div.selectedDivisionId === d.id }"
          @click="div.selectDivision(d.id)"
        >
          <div class="div-item-name">{{ d.name }}</div>
          <div class="div-item-meta">학생 {{ d.student_count }}명 · 교사 {{ d.teacher_count }}명</div>
        </button>
      </div>

      <!-- 우: 학생 상세 -->
      <div class="detail-panel">
        <div v-if="div.selectedDivisionId === null" class="detail-empty">
          <IconUsers :size="32" class="empty-icon" />
          <div>분반을 선택하면 학생 명부가 표시됩니다</div>
        </div>

        <template v-else>
          <div class="detail-header">
            <span class="detail-title">{{ selectedDivision?.name }}</span>
            <div class="detail-actions">
              <button v-if="auth.isAdmin" class="btn-secondary" @click="openBulkModal">
                <IconUpload :size="13" />일괄 등록
              </button>
              <button v-if="auth.isAdmin" class="btn-secondary" @click="openAddStudentModal">
                <IconPlus :size="13" />학생 추가
              </button>
              <button v-if="auth.isAdmin" class="btn-secondary" @click="openEditDivisionModal">
                <IconPencil :size="13" />분반 수정
              </button>
            </div>
          </div>

          <div v-if="div.loading" class="detail-loading">
            <div class="spinner"></div><span>불러오는 중...</span>
          </div>
          <div v-else-if="div.error" class="error-banner">
            <IconAlertTriangle :size="14" />{{ div.error }}
          </div>
          <div v-else-if="div.students.length === 0" class="detail-empty-students">
            등록된 학생이 없습니다
          </div>
          <div v-else class="student-table-wrap">
            <table class="student-table">
              <thead>
                <tr>
                  <th>학번</th>
                  <th>이름</th>
                  <th>비밀번호 상태</th>
                  <th>등록일</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="s in div.students" :key="s.id">
                  <td class="mono">{{ s.student_number }}</td>
                  <td>{{ s.name }}</td>
                  <td>
                    <span v-if="s.password_reset_required" class="badge badge-warn">
                      <IconAlertTriangle :size="11" />초기화 필요
                    </span>
                    <span v-else class="badge badge-ok">
                      <IconCheck :size="11" />정상
                    </span>
                  </td>
                  <td class="date-col">{{ formatDate(s.created_at) }}</td>
                  <td class="action-col">
                    <button class="btn-icon-sm" title="비밀번호 초기화" @click="openResetPasswordModal(s)">
                      <IconKey :size="13" />
                    </button>
                    <button v-if="auth.isAdmin" class="btn-icon-sm btn-danger" title="학생 삭제" @click="confirmDeleteStudent(s)">
                      <IconTrash :size="13" />
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
      </div>
    </div>

    <!-- 분반 생성 모달 -->
    <div v-if="modals.createDivision" class="modal-backdrop" @click.self="closeModal">
      <div class="modal">
        <div class="modal-title">분반 추가</div>
        <label>분반 이름</label>
        <input v-model="form.divisionName" type="text" placeholder="예) 프로그래밍 308" @keyup.enter="submitCreateDivision" />
        <div class="modal-error" v-if="modalError">{{ modalError }}</div>
        <div class="modal-actions">
          <button @click="closeModal">취소</button>
          <button class="btn-primary" @click="submitCreateDivision" :disabled="modalLoading">
            {{ modalLoading ? '추가 중...' : '추가' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 분반 수정 모달 -->
    <div v-if="modals.editDivision" class="modal-backdrop" @click.self="closeModal">
      <div class="modal">
        <div class="modal-title">분반 수정</div>
        <label>분반 이름</label>
        <input v-model="form.divisionName" type="text" @keyup.enter="submitEditDivision" />
        <div class="modal-error" v-if="modalError">{{ modalError }}</div>
        <div class="modal-footer-split">
          <button class="btn-danger-outline" @click="submitDeleteDivision" :disabled="modalLoading">
            <IconTrash :size="13" />삭제
          </button>
          <div class="modal-actions">
            <button @click="closeModal">취소</button>
            <button class="btn-primary" @click="submitEditDivision" :disabled="modalLoading">저장</button>
          </div>
        </div>
      </div>
    </div>

    <!-- 학생 단건 추가 모달 -->
    <div v-if="modals.addStudent" class="modal-backdrop" @click.self="closeModal">
      <div class="modal">
        <div class="modal-title">학생 추가</div>
        <label>학번</label>
        <input v-model="form.studentNumber" type="text" placeholder="예) 20240001" />
        <label>이름</label>
        <input v-model="form.studentName" type="text" placeholder="홍길동" />
        <label>초기 비밀번호</label>
        <input v-model="form.studentPassword" type="password" placeholder="최소 4자" />
        <div class="modal-error" v-if="modalError">{{ modalError }}</div>
        <div class="modal-actions">
          <button @click="closeModal">취소</button>
          <button class="btn-primary" @click="submitAddStudent" :disabled="modalLoading">
            {{ modalLoading ? '추가 중...' : '추가' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 일괄 등록 모달 -->
    <div v-if="modals.bulk" class="modal-backdrop" @click.self="closeModal">
      <div class="modal modal-wide">
        <div class="modal-title">학생 일괄 등록</div>
        <p class="modal-hint">JSON 배열 형식으로 입력하세요. 각 행: <code>{"student_number":"...","name":"...","password":"..."}</code></p>
        <textarea v-model="form.bulkJson" class="bulk-textarea" placeholder='[{"student_number":"20240001","name":"김철수","password":"init1234"}]'></textarea>
        <div class="modal-error" v-if="modalError">{{ modalError }}</div>
        <div class="bulk-result" v-if="bulkResult">
          <span class="badge badge-ok">등록 {{ bulkResult.inserted }}명</span>
          <span class="badge badge-warn" v-if="bulkResult.skipped > 0">건너뜀 {{ bulkResult.skipped }}명</span>
          <span class="badge badge-err" v-if="bulkResult.errors.length > 0">오류 {{ bulkResult.errors.length }}건</span>
        </div>
        <div class="modal-actions">
          <button @click="closeModal">닫기</button>
          <button class="btn-primary" @click="submitBulkImport" :disabled="modalLoading">
            {{ modalLoading ? '등록 중...' : '일괄 등록' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 비밀번호 초기화 모달 -->
    <div v-if="modals.resetPassword" class="modal-backdrop" @click.self="closeModal">
      <div class="modal">
        <div class="modal-title">비밀번호 초기화 — {{ selectedStudent?.name }}</div>
        <label>새 비밀번호</label>
        <input v-model="form.newPassword" type="password" placeholder="최소 4자" @keyup.enter="submitResetPassword" />
        <div class="modal-error" v-if="modalError">{{ modalError }}</div>
        <div class="modal-actions">
          <button @click="closeModal">취소</button>
          <button class="btn-primary" @click="submitResetPassword" :disabled="modalLoading">초기화</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useDivisionStore } from '@/stores/division'
import type { StudentRow } from '@/api/client'
import {
  IconSchool, IconLayoutDashboard, IconPlayerPlay, IconDatabase, IconList,
  IconFileText, IconUsers, IconHistory, IconCloudDownload, IconUserCog,
  IconPlus, IconUpload, IconPencil, IconAlertTriangle, IconCheck,
  IconKey, IconTrash,
} from '@tabler/icons-vue'

const router = useRouter()
const auth = useAuthStore()
const div = useDivisionStore()

const selectedDivision = computed(() => div.divisions.find(d => d.id === div.selectedDivisionId))
const selectedStudent = ref<StudentRow | null>(null)

// ─── 모달 상태 ───────────────────────────────────────────
const modals = reactive({
  createDivision: false,
  editDivision: false,
  addStudent: false,
  bulk: false,
  resetPassword: false,
})

const form = reactive({
  divisionName: '',
  studentNumber: '',
  studentName: '',
  studentPassword: '',
  newPassword: '',
  bulkJson: '',
})

const modalLoading = ref(false)
const modalError = ref<string | null>(null)
const bulkResult = ref<{ inserted: number; skipped: number; errors: string[] } | null>(null)

function closeModal() {
  Object.keys(modals).forEach(k => { (modals as Record<string, boolean>)[k] = false })
  modalError.value = null
  modalLoading.value = false
  bulkResult.value = null
  selectedStudent.value = null
}

// ─── 분반 생성 ───────────────────────────────────────────
function openCreateDivisionModal() {
  form.divisionName = ''
  closeModal()
  modals.createDivision = true
}

async function submitCreateDivision() {
  if (!form.divisionName.trim()) { modalError.value = '분반 이름을 입력하세요'; return }
  modalLoading.value = true
  modalError.value = null
  try {
    await div.createDivision(form.divisionName.trim())
    closeModal()
  } catch (e) {
    modalError.value = e instanceof Error ? e.message : '오류가 발생했습니다'
  } finally {
    modalLoading.value = false
  }
}

// ─── 분반 수정/삭제 ──────────────────────────────────────
function openEditDivisionModal() {
  form.divisionName = selectedDivision.value?.name ?? ''
  closeModal()
  modals.editDivision = true
}

async function submitEditDivision() {
  if (!form.divisionName.trim()) { modalError.value = '분반 이름을 입력하세요'; return }
  modalLoading.value = true
  modalError.value = null
  try {
    await div.updateDivision(div.selectedDivisionId!, form.divisionName.trim())
    closeModal()
  } catch (e) {
    modalError.value = e instanceof Error ? e.message : '오류가 발생했습니다'
  } finally {
    modalLoading.value = false
  }
}

async function submitDeleteDivision() {
  if (!confirm(`분반 "${selectedDivision.value?.name}"을(를) 삭제하시겠습니까?`)) return
  modalLoading.value = true
  modalError.value = null
  try {
    await div.deleteDivision(div.selectedDivisionId!)
    closeModal()
  } catch (e) {
    modalError.value = e instanceof Error ? e.message : '오류가 발생했습니다'
  } finally {
    modalLoading.value = false
  }
}

// ─── 학생 추가 ───────────────────────────────────────────
function openAddStudentModal() {
  form.studentNumber = ''
  form.studentName = ''
  form.studentPassword = ''
  closeModal()
  modals.addStudent = true
}

async function submitAddStudent() {
  if (!form.studentNumber.trim() || !form.studentName.trim() || !form.studentPassword) {
    modalError.value = '학번, 이름, 초기 비밀번호를 모두 입력하세요'
    return
  }
  modalLoading.value = true
  modalError.value = null
  try {
    await div.addStudent(div.selectedDivisionId!, {
      student_number: form.studentNumber.trim(),
      name: form.studentName.trim(),
      password: form.studentPassword,
    })
    closeModal()
  } catch (e) {
    modalError.value = e instanceof Error ? e.message : '오류가 발생했습니다'
  } finally {
    modalLoading.value = false
  }
}

// ─── 일괄 등록 ───────────────────────────────────────────
function openBulkModal() {
  form.bulkJson = ''
  closeModal()
  modals.bulk = true
}

async function submitBulkImport() {
  modalError.value = null
  let items: Array<{ student_number: string; name: string; password: string }>
  try {
    items = JSON.parse(form.bulkJson)
    if (!Array.isArray(items)) throw new Error('배열 형식이어야 합니다')
  } catch (e) {
    modalError.value = `JSON 파싱 오류: ${e instanceof Error ? e.message : '잘못된 형식'}`
    return
  }
  modalLoading.value = true
  try {
    bulkResult.value = await div.bulkImport(div.selectedDivisionId!, items)
  } catch (e) {
    modalError.value = e instanceof Error ? e.message : '오류가 발생했습니다'
  } finally {
    modalLoading.value = false
  }
}

// ─── 비밀번호 초기화 ─────────────────────────────────────
function openResetPasswordModal(s: StudentRow) {
  selectedStudent.value = s
  form.newPassword = ''
  closeModal()
  modals.resetPassword = true
}

async function submitResetPassword() {
  if (!form.newPassword) { modalError.value = '새 비밀번호를 입력하세요'; return }
  modalLoading.value = true
  modalError.value = null
  try {
    await div.resetStudentPassword(selectedStudent.value!.id, form.newPassword)
    const s = div.students.find(x => x.id === selectedStudent.value!.id)
    if (s) s.password_reset_required = true
    closeModal()
  } catch (e) {
    modalError.value = e instanceof Error ? e.message : '오류가 발생했습니다'
  } finally {
    modalLoading.value = false
  }
}

// ─── 학생 삭제 ───────────────────────────────────────────
async function confirmDeleteStudent(s: StudentRow) {
  if (!confirm(`${s.name}(${s.student_number}) 학생을 삭제하시겠습니까? 제출 기록은 남습니다.`)) return
  try {
    await div.deleteStudent(s.id)
  } catch (e) {
    alert(e instanceof Error ? e.message : '삭제 실패')
  }
}

// ─── 기타 ────────────────────────────────────────────────
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
  await div.fetchDivisions()
})
</script>

<style scoped>
.layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

/* ── 사이드바 (대시보드와 동일 스타일) ── */
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

/* ── 2단 패널 레이아웃 ── */
.main-split {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.list-panel {
  width: 220px;
  flex-shrink: 0;
  border-right: 1px solid var(--color-border-secondary);
  background: var(--color-background-primary);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 16px 10px;
  border-bottom: 1px solid var(--color-border-secondary);
}

.panel-title { font-weight: 600; font-size: 13px; }

.btn-icon {
  width: 26px;
  height: 26px;
  padding: 0;
  border: 1px solid var(--color-border-primary);
  border-radius: var(--border-radius-sm);
  background: var(--color-background-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.panel-loading, .panel-error, .panel-empty {
  padding: 16px;
  font-size: 12px;
  color: var(--color-text-tertiary);
  display: flex;
  align-items: center;
  gap: 8px;
}

.panel-error { color: var(--color-text-danger); }

.division-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 10px 16px;
  text-align: left;
  border: none;
  border-bottom: 1px solid var(--color-border-tertiary);
  background: none;
  cursor: pointer;
  transition: background 0.1s;
  width: 100%;
}

.division-item:hover { background: var(--color-background-secondary); }
.division-item.active { background: var(--color-background-info); }

.div-item-name { font-size: 13px; font-weight: 500; }
.div-item-meta { font-size: 11px; color: var(--color-text-tertiary); }

/* ── 상세 패널 ── */
.detail-panel {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  background: var(--color-background-secondary);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.detail-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--color-text-tertiary);
  font-size: 13px;
}

.empty-icon { color: var(--color-text-tertiary); }

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--color-background-primary);
  border: 1px solid var(--color-border-secondary);
  border-radius: var(--border-radius-lg);
  padding: 14px 16px;
}

.detail-title { font-weight: 600; font-size: 15px; }

.detail-actions { display: flex; gap: 8px; }

.btn-secondary {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  padding: 5px 10px;
}

.detail-loading {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--color-text-secondary);
  padding: 12px 0;
}

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

.detail-empty-students {
  font-size: 13px;
  color: var(--color-text-tertiary);
  padding: 12px 0;
}

/* ── 학생 테이블 ── */
.student-table-wrap {
  background: var(--color-background-primary);
  border: 1px solid var(--color-border-secondary);
  border-radius: var(--border-radius-lg);
  overflow: hidden;
}

.student-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.student-table th {
  padding: 10px 12px;
  text-align: left;
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-secondary);
  background: var(--color-background-secondary);
  border-bottom: 1px solid var(--color-border-secondary);
}

.student-table td {
  padding: 9px 12px;
  border-bottom: 1px solid var(--color-border-tertiary);
  vertical-align: middle;
}

.student-table tr:last-child td { border-bottom: none; }

.mono { font-family: 'Courier New', monospace; font-size: 12px; }

.date-col { color: var(--color-text-tertiary); font-size: 11px; }

.action-col {
  display: flex;
  gap: 4px;
  justify-content: flex-end;
}

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

.btn-icon-sm:hover { background: var(--color-background-secondary); }
.btn-icon-sm.btn-danger { color: var(--color-text-danger); }
.btn-icon-sm.btn-danger:hover { background: var(--color-background-danger); }

/* ── 배지 ── */
.badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  padding: 2px 7px;
  border-radius: 4px;
  font-weight: 500;
}

.badge-warn { background: var(--color-background-warning); color: var(--color-text-warning); }
.badge-ok { background: var(--color-background-success); color: var(--color-text-success); }
.badge-err { background: var(--color-background-danger); color: var(--color-text-danger); }

/* ── 스피너 ── */
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

.modal-wide { width: 520px; }

.modal-title { font-weight: 600; font-size: 15px; margin-bottom: 4px; }

.modal-hint { font-size: 12px; color: var(--color-text-secondary); margin: 0; }

.modal-error {
  font-size: 12px;
  color: var(--color-text-danger);
  background: var(--color-background-danger);
  padding: 8px 10px;
  border-radius: var(--border-radius-sm);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
}

.modal-footer-split {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 4px;
}

.btn-primary {
  background: var(--color-accent);
  color: #fff;
  border-color: transparent;
  padding: 7px 14px;
  font-size: 13px;
}

.btn-primary:hover { background: var(--color-accent-hover); }

.btn-danger-outline {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--color-text-danger);
  border-color: var(--color-border-danger);
  background: none;
  padding: 5px 10px;
}

.btn-danger-outline:hover { background: var(--color-background-danger); }

.bulk-textarea {
  width: 100%;
  height: 140px;
  font-family: 'Courier New', monospace;
  font-size: 12px;
  padding: 8px;
  border: 1px solid var(--color-border-primary);
  border-radius: var(--border-radius-sm);
  resize: vertical;
}

.bulk-result { display: flex; gap: 6px; flex-wrap: wrap; }
</style>
