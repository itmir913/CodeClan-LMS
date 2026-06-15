<template>
  <div class="flex h-screen overflow-hidden" style="background: var(--color-bg-primary)">

    <!-- ── 좌측 사이드바 (항상 다크) ── -->
    <aside class="w-60 flex-shrink-0 flex flex-col overflow-y-auto"
           style="background: #0f172a; border-right: 1px solid #1e293b">

      <!-- 상단: 뒤로 가기 + 수업명 -->
      <div class="px-4 pt-5 pb-0">
        <button
          class="flex items-center gap-2 h-9 px-2 rounded-lg border-0"
          style="background: transparent; color: #64748b"
          @click="goBack"
        >
          <IconArrowLeft :size="16" />
          <span>{{ $t('classes.backToHome') }}</span>
        </button>
        <div class="mt-4 pb-5">
          <div v-if="classLoading" class="h-5 w-32 rounded animate-pulse"
               style="background: #1e293b"></div>
          <template v-else>
            <div class="font-bold leading-snug" style="font-size: 17px; color: #f1f5f9; letter-spacing: -0.01em">
              {{ classDetail?.name }}
            </div>
            <div class="mt-1" style="font-size: 16px; color: #64748b">
              {{ classDetail?.subject_name }}
            </div>
          </template>
        </div>
      </div>

      <div style="height: 1px; background: #1e293b; margin: 0 16px"></div>

      <!-- 탭 네비게이션 -->
      <nav class="px-2.5 pt-3 flex flex-col gap-0.5">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          class="flex items-center justify-start gap-3 h-11 px-3 rounded-lg border-0 w-full"
          :style="activeTab === tab.key
            ? { boxShadow: 'inset 3px 0 0 #93c5fd', background: 'rgba(147,197,253,.1)', color: '#f8fafc', fontWeight: 600 }
            : { background: 'transparent', color: '#64748b' }"
          @click="activeTab = tab.key"
        >
          <component :is="tab.icon" :size="18" />
          <span>{{ $t(tab.labelKey) }}</span>
        </button>
      </nav>

      <div class="flex-1"></div>

      <!-- 하단: 프로필 -->
      <div style="height: 1px; background: #1e293b; margin: 0 16px 12px"></div>
      <div class="px-2.5 pb-5">
        <div class="flex items-center gap-2.5 px-2.5 py-2 rounded-lg"
             style="color: #b8c5d3">
          <div class="w-9 h-9 rounded-full flex items-center justify-center font-bold flex-shrink-0"
               style="background: rgba(37,99,235,.4); color: #93c5fd; font-size: 16px">
            {{ auth.teacher?.name?.charAt(0) ?? '?' }}
          </div>
          <span class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap"
                style="font-size: 16px">
            {{ $t('auth.teacherGreeting', { name: auth.teacher?.name }) }}
          </span>
        </div>
      </div>
    </aside>

    <!-- ── 우측 메인 영역 ── -->
    <div class="flex-1 flex flex-col min-w-0 overflow-hidden">

      <!-- 상단 바 -->
      <header class="h-16 flex-shrink-0 flex items-center justify-between px-6 border-b"
              style="background: var(--color-bg-secondary); border-color: var(--color-border)">
        <span class="font-semibold" style="font-size: 18px; color: var(--color-text-primary)">
          {{ classDetail?.name ?? '' }}
        </span>
        <div class="flex items-center gap-2">
          <button
            class="w-9 h-9 p-0 rounded-lg flex items-center justify-center"
            style="border: 1px solid var(--color-border); color: var(--color-text-muted); background: transparent"
            @click="toggleTheme"
            :aria-label="$t('auth.toggleTheme')"
          >
            <IconMoon v-if="!isDark" :size="18" />
            <IconSun v-else :size="18" />
          </button>
          <LanguageSelector />
        </div>
      </header>

      <!-- 콘텐츠 -->
      <main class="flex-1 overflow-y-auto px-7 py-7">

        <!-- ── 학생 탭 ── -->
        <template v-if="activeTab === 'students'">
          <!-- 헤더 -->
          <div class="flex items-center justify-between gap-3 mb-5 flex-wrap">
            <div>
              <h2 class="font-bold" style="font-size: 20px; color: var(--color-text-primary)">
                {{ $t('students.title') }}
              </h2>
              <p class="mt-1" style="color: var(--color-text-muted)">
                {{ $t('students.count', { count: studentStore.students.length }) }}
              </p>
            </div>
            <div class="flex items-center gap-2 flex-wrap">
              <!-- 검색 -->
              <div class="relative">
                <IconSearch :size="15" class="absolute left-3 top-1/2 -translate-y-1/2"
                            style="color: var(--color-text-muted)" />
                <input
                  v-model="searchQuery"
                  type="text"
                  :placeholder="$t('students.searchPlaceholder')"
                  class="h-9 w-48 pl-8 pr-3 rounded-lg border outline-none"
                  style="background: var(--color-bg-secondary); border-color: var(--color-border); color: var(--color-text-primary)"
                />
              </div>
              <!-- 개별 추가 -->
              <button
                class="h-9 px-4 rounded-lg flex items-center gap-1.5 font-medium"
                style="border: 1px solid var(--color-border); color: var(--color-text-primary); background: var(--color-bg-secondary)"
                @click="openAddModal"
              >
                <IconUserPlus :size="15" />
                {{ $t('students.addStudent') }}
              </button>
              <!-- 엑셀 임포트 -->
              <button
                class="h-9 px-4 rounded-lg flex items-center gap-1.5 font-semibold"
                style="background: var(--color-accent); color: var(--color-accent-text); border: none"
                @click="showImportModal = true"
              >
                <IconUpload :size="15" />
                {{ $t('students.importExcel') }}
              </button>
            </div>
          </div>

          <!-- 로딩 -->
          <div v-if="studentStore.loading"
               class="flex items-center gap-3 py-10"
               style="color: var(--color-text-muted)">
            <IconLoader2 :size="20" class="spin" />
            <span>{{ $t('common.loading') }}</span>
          </div>

          <!-- 에러 -->
          <div v-else-if="studentStore.error"
               class="flex items-center gap-3 rounded-xl border px-5 py-4"
               style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
               role="alert">
            <IconAlertCircle :size="20" class="shrink-0" />
            <span>{{ $t(`errors.${studentStore.error}`, $t('errors.ERR_UNKNOWN')) }}</span>
            <button
              class="ml-auto h-8 px-3 rounded-lg font-medium"
              style="background: transparent; border: 1px solid var(--color-danger-border); color: var(--color-danger)"
              @click="loadStudents"
            >{{ $t('common.retry') }}</button>
          </div>

          <!-- 빈 상태 -->
          <div v-else-if="filteredStudents.length === 0 && !searchQuery"
               class="flex flex-col items-center justify-center py-20 rounded-xl border"
               style="border: 1.5px dashed var(--color-border); border-radius: 16px">
            <div class="w-16 h-16 rounded-2xl flex items-center justify-center mb-4"
                 style="background: var(--color-info-bg); color: var(--color-accent)">
              <IconUsers :size="28" />
            </div>
            <p class="font-semibold mb-2" style="font-size: 18px; color: var(--color-text-primary)">
              {{ $t('students.noStudents') }}
            </p>
            <p style="color: var(--color-text-muted)">{{ $t('students.noStudentsHint') }}</p>
          </div>

          <!-- 검색 결과 없음 -->
          <div v-else-if="filteredStudents.length === 0 && searchQuery"
               class="py-10 text-center" style="color: var(--color-text-muted)">
            {{ $t('common.noResults') }}
          </div>

          <!-- 학생 테이블 -->
          <div v-else class="rounded-xl border overflow-hidden"
               style="border-color: var(--color-border)">
            <table class="w-full">
              <thead>
                <tr style="background: var(--color-bg-tertiary); border-bottom: 1px solid var(--color-border)">
                  <th class="px-5 py-3 text-left font-semibold" style="color: var(--color-text-muted)">
                    {{ $t('students.number') }}
                  </th>
                  <th class="px-5 py-3 text-left font-semibold" style="color: var(--color-text-muted)">
                    {{ $t('students.name') }}
                  </th>
                  <th class="px-5 py-3 text-left font-semibold hidden sm:table-cell"
                      style="color: var(--color-text-muted)">
                    {{ $t('students.username') }}
                  </th>
                  <th class="px-5 py-3 text-left font-semibold hidden md:table-cell"
                      style="color: var(--color-text-muted)">
                    {{ $t('students.passwordResetRequired') }}
                  </th>
                  <th class="px-5 py-3 w-24"></th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="student in filteredStudents"
                  :key="student.id"
                  class="border-t"
                  style="border-color: var(--color-border)"
                >
                  <td class="px-5 py-3 font-medium" style="color: var(--color-text-muted)">
                    {{ student.number }}
                  </td>
                  <td class="px-5 py-3 font-medium" style="color: var(--color-text-primary)">
                    {{ student.name }}
                  </td>
                  <td class="px-5 py-3 hidden sm:table-cell" style="color: var(--color-text-muted); font-family: monospace">
                    {{ student.username }}
                  </td>
                  <td class="px-5 py-3 hidden md:table-cell">
                    <span
                      class="inline-flex items-center gap-1.5 rounded-full px-3 py-0.5 font-medium"
                      :style="student.password_reset_required
                        ? { background: 'var(--color-warning-bg)', color: 'var(--color-warning)' }
                        : { background: 'var(--color-success-bg)', color: 'var(--color-success)' }"
                    >
                      <span class="w-1.5 h-1.5 rounded-full"
                            :style="student.password_reset_required
                              ? { background: 'var(--color-warning)' }
                              : { background: 'var(--color-success)' }"></span>
                      {{ student.password_reset_required
                          ? $t('students.passwordResetRequired')
                          : $t('students.passwordSet') }}
                    </span>
                  </td>
                  <td class="px-5 py-3">
                    <div class="flex items-center gap-1 justify-end">
                      <button
                        class="w-8 h-8 p-0 rounded-lg flex items-center justify-center"
                        style="background: transparent; border: 1px solid var(--color-border); color: var(--color-text-muted)"
                        @click="openResetPasswordModal(student)"
                        :aria-label="$t('students.resetPassword')"
                      >
                        <IconKey :size="14" />
                      </button>
                      <button
                        class="w-8 h-8 p-0 rounded-lg flex items-center justify-center"
                        style="background: transparent; border: 1px solid var(--color-border); color: var(--color-text-muted)"
                        @click="openDeleteModal(student)"
                        :aria-label="$t('students.deleteStudent')"
                      >
                        <IconTrash :size="14" />
                      </button>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>

        <!-- ── 다른 탭들 (스텁) ── -->
        <template v-else>
          <div class="flex flex-col items-center justify-center py-32"
               style="color: var(--color-text-muted)">
            <IconTool :size="40" class="mb-4 opacity-40" />
            <p class="font-semibold" style="font-size: 18px; color: var(--color-text-primary)">
              {{ $t('classes.tabComingSoon') }}
            </p>
          </div>
        </template>

      </main>
    </div>

    <!-- ── 개별 추가 모달 ── -->
    <Teleport to="body">
      <div v-if="showAddModal"
           class="fixed inset-0 z-50 flex items-center justify-center px-4"
           style="background: rgba(0,0,0,0.45)">
        <div class="w-full max-w-sm rounded-xl p-6"
             style="background: var(--color-bg-secondary); border: 1px solid var(--color-border); box-shadow: var(--shadow-dropdown)">
          <h2 class="font-semibold mb-5" style="color: var(--color-text-primary)">
            {{ $t('students.addStudent') }}
          </h2>
          <form @submit.prevent="onAddSubmit" novalidate class="flex flex-col gap-4">
            <div class="flex flex-col gap-2">
              <label class="font-medium" style="color: var(--color-text-primary)">{{ $t('students.name') }}</label>
              <input v-model="addForm.name" type="text" :disabled="isAdding"
                     class="h-12 w-full px-4 rounded-lg border outline-none"
                     style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)" />
            </div>
            <div class="grid grid-cols-3 gap-3">
              <div class="flex flex-col gap-2">
                <label class="font-medium" style="color: var(--color-text-primary)">{{ $t('students.grade') }}</label>
                <input v-model.number="addForm.grade" type="number" min="1" max="6" :disabled="isAdding"
                       class="h-12 w-full px-3 rounded-lg border outline-none"
                       style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)" />
              </div>
              <div class="flex flex-col gap-2">
                <label class="font-medium" style="color: var(--color-text-primary)">{{ $t('students.classNo') }}</label>
                <input v-model.number="addForm.class_no" type="number" min="1" max="99" :disabled="isAdding"
                       class="h-12 w-full px-3 rounded-lg border outline-none"
                       style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)" />
              </div>
              <div class="flex flex-col gap-2">
                <label class="font-medium" style="color: var(--color-text-primary)">{{ $t('students.number') }}</label>
                <input v-model.number="addForm.number" type="number" min="1" max="99" :disabled="isAdding"
                       class="h-12 w-full px-3 rounded-lg border outline-none"
                       style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)" />
              </div>
            </div>
            <div v-if="addError"
                 class="flex items-center gap-2 rounded-lg border px-4 py-3"
                 style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
                 role="alert">
              <IconAlertCircle :size="18" class="shrink-0" />
              <span>{{ addError }}</span>
            </div>
            <div class="flex justify-end gap-3 pt-1">
              <button type="button" class="h-10 px-5 rounded-lg font-medium"
                      style="border: 1px solid var(--color-border); color: var(--color-text-primary); background: transparent"
                      @click="closeModals">{{ $t('students.cancel') }}</button>
              <button type="submit" :disabled="isAdding"
                      class="h-10 px-5 rounded-lg font-medium flex items-center gap-2"
                      style="background: var(--color-accent); color: var(--color-accent-text); border: none"
                      :class="isAdding ? 'opacity-60 cursor-not-allowed' : ''">
                <IconLoader2 v-if="isAdding" :size="17" class="spin" />
                {{ isAdding ? $t('students.adding') : $t('students.add') }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </Teleport>

    <!-- ── 엑셀 임포트 모달 ── -->
    <ImportModal
      v-model:show="showImportModal"
      :title="$t('students.importExcel')"
      template-filename="students_template"
      :template-headers="studentTemplateHeaders"
      :template-sample="[['3', '1', '1', 'Hong Gildong', '30101'], ['3', '1', '2', 'Kim Cheolsu', '']]"
      :synonym-map="studentSynonymMap"
      :required-fields="['grade', 'class_no', 'number', 'name']"
      :columns="studentImportColumns"
      :on-import="handleImportStudents"
    />

    <!-- ── 비밀번호 초기화 확인 모달 ── -->
    <Teleport to="body">
      <div v-if="showResetModal"
           class="fixed inset-0 z-50 flex items-center justify-center px-4"
           style="background: rgba(0,0,0,0.45)">
        <div class="w-full max-w-sm rounded-xl p-6"
             style="background: var(--color-bg-secondary); border: 1px solid var(--color-border); box-shadow: var(--shadow-dropdown)">
          <h2 class="font-semibold mb-2" style="color: var(--color-text-primary)">
            {{ $t('students.resetPassword') }}
          </h2>
          <p class="mb-1" style="color: var(--color-text-primary)">
            {{ $t('students.resetPasswordConfirm', { name: resetTarget?.name }) }}
          </p>
          <p class="mb-5" style="color: var(--color-text-muted)">
            {{ $t('students.resetPasswordConfirmHint') }}
          </p>
          <div v-if="resetError"
               class="mb-4 flex items-center gap-2 rounded-lg border px-4 py-3"
               style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
               role="alert">
            <IconAlertCircle :size="18" class="shrink-0" />
            <span>{{ resetError }}</span>
          </div>
          <div class="flex justify-end gap-3">
            <button class="h-10 px-5 rounded-lg font-medium"
                    style="border: 1px solid var(--color-border); color: var(--color-text-primary); background: transparent"
                    @click="closeModals" :disabled="isResetting">{{ $t('students.cancel') }}</button>
            <button :disabled="isResetting"
                    class="h-10 px-5 rounded-lg font-medium flex items-center gap-2"
                    style="background: var(--color-accent); color: var(--color-accent-text); border: none"
                    :class="isResetting ? 'opacity-60 cursor-not-allowed' : ''"
                    @click="onResetConfirm">
              <IconLoader2 v-if="isResetting" :size="17" class="spin" />
              {{ isResetting ? $t('students.resetting') : $t('students.reset') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- ── 학생 삭제 확인 모달 ── -->
    <Teleport to="body">
      <div v-if="showDeleteModal"
           class="fixed inset-0 z-50 flex items-center justify-center px-4"
           style="background: rgba(0,0,0,0.45)">
        <div class="w-full max-w-sm rounded-xl p-6"
             style="background: var(--color-bg-secondary); border: 1px solid var(--color-border); box-shadow: var(--shadow-dropdown)">
          <h2 class="font-semibold mb-2" style="color: var(--color-text-primary)">
            {{ $t('students.deleteStudent') }}
          </h2>
          <p class="mb-1" style="color: var(--color-text-primary)">
            {{ $t('students.deleteConfirm', { name: deleteTarget?.name }) }}
          </p>
          <p class="mb-5" style="color: var(--color-text-muted)">
            {{ $t('students.deleteConfirmHint') }}
          </p>
          <div v-if="deleteError"
               class="mb-4 flex items-center gap-2 rounded-lg border px-4 py-3"
               style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
               role="alert">
            <IconAlertCircle :size="18" class="shrink-0" />
            <span>{{ deleteError }}</span>
          </div>
          <div class="flex justify-end gap-3">
            <button class="h-10 px-5 rounded-lg font-medium"
                    style="border: 1px solid var(--color-border); color: var(--color-text-primary); background: transparent"
                    @click="closeModals" :disabled="isDeleting">{{ $t('students.cancel') }}</button>
            <button :disabled="isDeleting"
                    class="h-10 px-5 rounded-lg font-medium flex items-center gap-2"
                    style="background: var(--color-danger); color: #fff; border: none"
                    :class="isDeleting ? 'opacity-60 cursor-not-allowed' : ''"
                    @click="onDeleteConfirm">
              <IconLoader2 v-if="isDeleting" :size="17" class="spin" />
              {{ isDeleting ? $t('students.deleting') : $t('students.delete') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  IconArrowLeft, IconMoon, IconSun, IconLoader2, IconAlertCircle,
  IconUsers, IconUserPlus, IconUpload, IconSearch, IconKey, IconTrash, IconTool,
  IconBook2, IconClipboardList, IconCalendarCheck,
} from '@tabler/icons-vue'
import { useAuthStore } from '@/stores/auth'
import { useStudentStore } from '@/stores/student'
import { api, type ClassDetail, type StudentItem, type AddStudentBody } from '@/api/client'
import LanguageSelector from '@/components/LanguageSelector.vue'
import ImportModal from '@/components/ImportModal.vue'
import type { SynonymMap } from '@/utils/excelImport'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const auth = useAuthStore()
const studentStore = useStudentStore()

const classId = computed(() => Number(route.params.id))
const classDetail = ref<ClassDetail | null>(null)
const classLoading = ref(false)

const isDark = ref(document.documentElement.getAttribute('data-theme') === 'dark')
const activeTab = ref<'students' | 'lessons' | 'assessments' | 'attendance'>('students')
const searchQuery = ref('')

// ── 탭 정의 ──────────────────────────────────────────
const tabs = [
  { key: 'students',    labelKey: 'classes.tabStudents',    icon: IconUsers },
  { key: 'lessons',     labelKey: 'classes.tabLessons',     icon: IconBook2 },
  { key: 'assessments', labelKey: 'classes.tabAssessments', icon: IconClipboardList },
  { key: 'attendance',  labelKey: 'classes.tabAttendance',  icon: IconCalendarCheck },
] as const

// ── 학생 필터링 ───────────────────────────────────────
const filteredStudents = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return studentStore.students
  return studentStore.students.filter(
    (s) =>
      s.name.toLowerCase().includes(q) ||
      String(s.number).includes(q) ||
      s.username.toLowerCase().includes(q),
  )
})

// ── 개별 추가 모달 ────────────────────────────────────
const showAddModal = ref(false)
const addForm = ref<AddStudentBody>({ name: '', grade: 3, class_no: 1, number: 1 })
const isAdding = ref(false)
const addError = ref<string | null>(null)

function openAddModal() {
  closeModals()
  addForm.value = { name: '', grade: 3, class_no: 1, number: 1 }
  showAddModal.value = true
}

async function onAddSubmit() {
  if (isAdding.value) return
  addError.value = null
  isAdding.value = true
  try {
    await studentStore.addStudent(classId.value, addForm.value)
    closeModals()
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    addError.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isAdding.value = false
  }
}

// ── 엑셀 임포트 모달 ─────────────────────────────────
const showImportModal = ref(false)

const studentSynonymMap: SynonymMap = {
  grade: ['grade', '학년', 'year'],
  class_no: ['class', 'class_no', '반', '학반', 'division'],
  number: ['number', 'no', '번호', 'student_no'],
  name: ['name', '이름', '성명', '학생명', 'student_name'],
  username: ['username', 'student_id', '학번', 'id', '아이디'],
}

const studentImportColumns = [
  { key: 'grade', labelKey: 'students.grade' },
  { key: 'class_no', labelKey: 'students.classNo' },
  { key: 'number', labelKey: 'students.number' },
  { key: 'name', labelKey: 'students.name' },
  { key: 'username', labelKey: 'students.username' },
]
const studentTemplateHeaders = computed(() => [
  t('students.grade'),
  t('students.classNo'),
  t('students.number'),
  t('students.name'),
  t('students.username'),
])

async function handleImportStudents(rows: Record<string, string>[]) {
  const data = rows.map((r) => ({
    grade: parseInt(r.grade, 10),
    class_no: parseInt(r.class_no, 10),
    number: parseInt(r.number, 10),
    name: r.name,
    username: r.username || undefined,
  }))
  await studentStore.importStudents(classId.value, data)
}

// ── 비밀번호 초기화 모달 ──────────────────────────────
const showResetModal = ref(false)
const resetTarget = ref<StudentItem | null>(null)
const isResetting = ref(false)
const resetError = ref<string | null>(null)

function openResetPasswordModal(student: StudentItem) {
  closeModals()
  resetTarget.value = student
  showResetModal.value = true
}

async function onResetConfirm() {
  if (isResetting.value) return
  resetError.value = null
  isResetting.value = true
  try {
    await studentStore.resetStudentPassword(resetTarget.value!.id)
    closeModals()
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    resetError.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isResetting.value = false
  }
}

// ── 삭제 모달 ─────────────────────────────────────────
const showDeleteModal = ref(false)
const deleteTarget = ref<StudentItem | null>(null)
const isDeleting = ref(false)
const deleteError = ref<string | null>(null)

function openDeleteModal(student: StudentItem) {
  closeModals()
  deleteTarget.value = student
  showDeleteModal.value = true
}

async function onDeleteConfirm() {
  if (isDeleting.value) return
  deleteError.value = null
  isDeleting.value = true
  try {
    await studentStore.deleteStudent(classId.value, deleteTarget.value!.id)
    closeModals()
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    deleteError.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isDeleting.value = false
  }
}

// ── 공통 ──────────────────────────────────────────────
function closeModals() {
  showAddModal.value = false
  showImportModal.value = false
  showResetModal.value = false
  showDeleteModal.value = false
  resetTarget.value = null
  deleteTarget.value = null
  addError.value = null
  resetError.value = null
  deleteError.value = null
}

function toggleTheme() {
  isDark.value = !isDark.value
  document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
}

function goBack() {
  const role = auth.teacher?.role
  if (role === 'admin') {
    router.push('/admin')
  } else {
    router.push('/teacher')
  }
}

async function loadStudents() {
  await studentStore.fetchStudents(classId.value)
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') closeModals()
}

onMounted(async () => {
  if (!auth.teacher) {
    try { await auth.fetchTeacherMe() } catch { router.push('/login'); return }
  }
  classLoading.value = true
  try {
    classDetail.value = await api.classes.get(classId.value)
  } catch {
    router.push(auth.teacher?.role === 'admin' ? '/admin' : '/teacher')
    return
  } finally {
    classLoading.value = false
  }
  await loadStudents()
  document.addEventListener('keydown', onKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', onKeydown)
})

// classId 변경 시 재로드 (탐색)
watch(classId, async (newId) => {
  if (!newId) return
  classLoading.value = true
  try {
    classDetail.value = await api.classes.get(newId)
  } finally {
    classLoading.value = false
  }
  await loadStudents()
})
</script>
