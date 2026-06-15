<template>
  <div class="min-h-screen" style="background: var(--color-bg-primary)">

    <!-- Top Nav -->
    <header class="sticky top-0 z-30 h-16 border-b"
            style="background: var(--color-bg-secondary); border-color: var(--color-border)">
      <div class="h-full max-w-full flex items-center justify-between px-6">

        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg flex items-center justify-center font-bold text-white shrink-0"
               style="background: var(--color-accent)">C</div>
          <span class="font-semibold" style="color: var(--color-text-primary)">CodeClan LMS</span>
        </div>

        <div class="flex items-center gap-2">
          <span class="hidden sm:inline font-medium mr-1" style="color: var(--color-text-primary)">
            {{ $t('auth.teacherGreeting', { name: auth.teacher?.name }) }}
          </span>

          <button
            class="w-9 h-9 p-0 rounded-lg flex items-center justify-center border bg-transparent"
            style="border-color: var(--color-border); color: var(--color-text-muted);"
            @click="showSettings = true"
            :aria-label="$t('common.settings')"
          >
            <IconSettings :size="18" />
          </button>

          <button
            class="w-9 h-9 p-0 rounded-lg flex items-center justify-center border bg-transparent"
            style="border-color: var(--color-border); color: var(--color-text-muted);"
            @click="toggleTheme"
            :aria-label="$t('auth.toggleTheme')"
          >
            <IconMoon v-if="!isDark" :size="18" />
            <IconSun v-else :size="18" />
          </button>

          <LanguageSelector />

          <button
            class="h-9 px-3 rounded-lg font-medium border bg-transparent"
            style="border-color: var(--color-border); color: var(--color-text-muted);"
            @click="onLogout"
            :disabled="isLoggingOut"
          >
            {{ $t('common.logout') }}
          </button>
        </div>

      </div>
    </header>

    <!-- Main -->
    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 pb-16">

      <!-- Section header -->
      <div class="flex items-center justify-between mb-5">
        <h2 class="font-semibold tracking-widest uppercase"
            style="color: var(--color-text-muted)">
          {{ $t('classes.myClasses') }}
          <span v-if="!classStore.loading">({{ classStore.classes.length }})</span>
        </h2>
        <div class="flex items-center gap-2">
          <button
            class="h-9 px-4 rounded-lg flex items-center gap-2 font-medium border bg-transparent"
            style="color: var(--color-text-muted); border-color: var(--color-border)"
            @click="$router.push('/problem-bank')"
          >
            <IconBooks :size="16" />
            {{ $t('problems.title') }}
          </button>
          <button
            class="h-9 px-4 rounded-lg flex items-center gap-2 font-medium border-0"
            style="background: var(--color-accent); color: var(--color-accent-text);"
            @click="openAddModal"
          >
            <IconPlus :size="17" />
            {{ $t('classes.addClass') }}
          </button>
        </div>
      </div>

      <!-- Loading -->
      <div v-if="classStore.loading"
           class="flex items-center justify-center py-24 gap-3"
           style="color: var(--color-text-muted)">
        <IconLoader2 :size="22" class="spin" />
        <span>{{ $t('common.loading') }}</span>
      </div>

      <!-- Error -->
      <div v-else-if="classStore.error"
           class="flex items-center gap-3 rounded-xl border px-5 py-4"
           style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
           role="alert">
        <IconAlertCircle :size="20" class="shrink-0" />
        <span>{{ $t(`errors.${classStore.error}`, $t('errors.ERR_UNKNOWN')) }}</span>
        <button
          class="ml-auto h-8 px-3 rounded-lg font-medium border bg-transparent"
          style="border-color: var(--color-danger-border); color: var(--color-danger)"
          @click="classStore.fetchClasses()"
        >{{ $t('common.retry') }}</button>
      </div>

      <!-- Empty state -->
      <div v-else-if="classStore.classes.length === 0"
           class="flex flex-col items-center justify-center py-24 gap-3">
        <IconSchool :size="48" style="color: var(--color-text-tertiary)" />
        <p class="font-medium" style="color: var(--color-text-muted)">{{ $t('classes.noClasses') }}</p>
        <p style="color: var(--color-text-tertiary)">{{ $t('classes.noClassesHint') }}</p>
      </div>

      <!-- Card grid -->
      <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        <article
          v-for="cls in classStore.classes"
          :key="cls.id"
          class="class-card rounded-xl border flex flex-col overflow-hidden cursor-pointer"
          style="background: var(--color-bg-secondary); border-color: var(--color-border); box-shadow: var(--shadow-card)"
          @click="router.push(`/classes/${cls.id}`)"
        >
          <span class="h-1 block shrink-0" :style="{ background: cardAccentColor(cls.id) }"></span>

          <div class="flex flex-col flex-1 p-5">
            <div class="flex items-center justify-between gap-2">
              <div>
                <h3 class="font-bold" style="color: var(--color-text-primary)">{{ cls.name }}</h3>
                <p class="mt-1" style="color: var(--color-text-muted)">{{ cls.subject_name }}</p>
              </div>
              <div class="flex items-center gap-1 shrink-0">
                <button
                  class="w-8 h-8 p-0 rounded-lg flex items-center justify-center border bg-transparent"
                  style="border-color: var(--color-border); color: var(--color-text-muted)"
                  @click.stop="openEditModal(cls)"
                  :aria-label="$t('classes.editClass')"
                >
                  <IconPencil :size="15" />
                </button>
                <button
                  class="w-8 h-8 p-0 rounded-lg flex items-center justify-center border bg-transparent"
                  style="border-color: var(--color-border); color: var(--color-text-muted)"
                  @click.stop="openDeleteModal(cls)"
                  :aria-label="$t('classes.deleteClass')"
                >
                  <IconTrash :size="15" />
                </button>
              </div>
            </div>

            <div class="mt-4 pt-3 border-t flex items-center gap-2"
                 style="border-color: var(--color-border)">
              <template v-if="cls.has_active_session">
                <span class="inline-flex items-center gap-1.5 rounded-full px-3 py-1 font-semibold session-live-badge">
                  <span class="w-2 h-2 rounded-full shrink-0" style="background: currentColor"></span>
                  {{ $t('classes.sessionLive') }}
                </span>
              </template>
              <template v-else>
                <IconUsers :size="16" style="color: var(--color-text-tertiary)" />
                <span style="color: var(--color-text-muted)">
                  {{ $t('classes.students', { count: cls.student_count }) }}
                </span>
              </template>
            </div>
          </div>
        </article>
      </div>

    </main>

    <!-- ── Add Class Modal ── -->
    <Teleport to="body">
      <div v-if="showAddModal"
           class="fixed inset-0 z-50 overflow-y-auto"
           style="background: var(--color-modal-overlay)">
        <div class="flex min-h-full items-center justify-center px-4 py-4">
        <div class="w-full max-w-lg rounded-xl p-6 border"
             style="background: var(--color-bg-secondary); border-color: var(--color-border); box-shadow: var(--shadow-dropdown)">
          <h2 class="font-semibold mb-5" style="color: var(--color-text-primary)">
            {{ $t('classes.addClass') }}
          </h2>

          <div v-if="classStore.subjects.length === 0"
               class="mb-4 rounded-lg border px-4 py-3"
               style="background: var(--color-warning-bg); border-color: var(--color-border); color: var(--color-warning)">
            {{ $t('classes.noSubjects') }}
          </div>

          <form @submit.prevent="onAddSubmit" novalidate class="flex flex-col gap-4">
            <div class="flex flex-col gap-2">
              <label class="font-medium" style="color: var(--color-text-primary)">
                {{ $t('classes.subject') }}
              </label>
              <select
                v-model.number="addForm.subject_id"
                :disabled="isAdding || classStore.subjects.length === 0"
                class="h-12 w-full px-4 rounded-lg border outline-none"
                style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)"
              >
                <option value="" disabled>{{ $t('classes.subjectPlaceholder') }}</option>
                <option v-for="s in classStore.subjects" :key="s.id" :value="s.id">{{ s.name }}</option>
              </select>
            </div>

            <div class="flex flex-col gap-2">
              <label class="font-medium" style="color: var(--color-text-primary)">
                {{ $t('classes.className') }}
              </label>
              <input
                v-model="addForm.name"
                type="text"
                :placeholder="$t('classes.classNamePlaceholder')"
                :disabled="isAdding"
                class="h-12 w-full px-4 rounded-lg border outline-none"
                style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)"
              />
            </div>

            <div v-if="addError"
                 class="flex items-center gap-2 rounded-lg border px-4 py-3"
                 style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
                 role="alert">
              <IconAlertCircle :size="18" class="shrink-0" />
              <span>{{ addError }}</span>
            </div>

            <div class="flex justify-end gap-3 pt-1">
              <button
                type="button"
                class="h-10 px-5 rounded-lg font-medium border bg-transparent"
                style="border-color: var(--color-border); color: var(--color-text-primary);"
                @click="closeModals"
              >{{ $t('classes.cancel') }}</button>
              <button
                type="submit"
                :disabled="isAdding"
                class="h-10 px-5 rounded-lg font-medium flex items-center gap-2 border-0"
                style="background: var(--color-accent); color: var(--color-accent-text);"
                :class="isAdding ? 'opacity-60 cursor-not-allowed' : ''"
              >
                <IconLoader2 v-if="isAdding" :size="17" class="spin" />
                {{ isAdding ? $t('classes.adding') : $t('classes.add') }}
              </button>
            </div>
          </form>
        </div>
        </div>
      </div>
    </Teleport>

    <!-- ── Edit Class Modal ── -->
    <Teleport to="body">
      <div v-if="showEditModal"
           class="fixed inset-0 z-50 overflow-y-auto"
           style="background: var(--color-modal-overlay)">
        <div class="flex min-h-full items-center justify-center px-4 py-4">
        <div class="w-full max-w-lg rounded-xl p-6 border"
             style="background: var(--color-bg-secondary); border-color: var(--color-border); box-shadow: var(--shadow-dropdown)">
          <h2 class="font-semibold mb-5" style="color: var(--color-text-primary)">
            {{ $t('classes.editClass') }}
          </h2>

          <form @submit.prevent="onEditSubmit" novalidate class="flex flex-col gap-4">
            <div class="flex flex-col gap-2">
              <label class="font-medium" style="color: var(--color-text-primary)">
                {{ $t('classes.subject') }}
              </label>
              <select
                v-model.number="editForm.subject_id"
                :disabled="isSaving"
                class="h-12 w-full px-4 rounded-lg border outline-none"
                style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)"
              >
                <option v-for="s in classStore.subjects" :key="s.id" :value="s.id">{{ s.name }}</option>
              </select>
            </div>

            <div class="flex flex-col gap-2">
              <label class="font-medium" style="color: var(--color-text-primary)">
                {{ $t('classes.className') }}
              </label>
              <input
                v-model="editForm.name"
                type="text"
                :placeholder="$t('classes.classNamePlaceholder')"
                :disabled="isSaving"
                class="h-12 w-full px-4 rounded-lg border outline-none"
                style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)"
              />
            </div>

            <div v-if="editError"
                 class="flex items-center gap-2 rounded-lg border px-4 py-3"
                 style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
                 role="alert">
              <IconAlertCircle :size="18" class="shrink-0" />
              <span>{{ editError }}</span>
            </div>

            <div class="flex justify-end gap-3 pt-1">
              <button
                type="button"
                class="h-10 px-5 rounded-lg font-medium border bg-transparent"
                style="border-color: var(--color-border); color: var(--color-text-primary);"
                @click="closeModals"
              >{{ $t('classes.cancel') }}</button>
              <button
                type="submit"
                :disabled="isSaving"
                class="h-10 px-5 rounded-lg font-medium flex items-center gap-2 border-0"
                style="background: var(--color-accent); color: var(--color-accent-text);"
                :class="isSaving ? 'opacity-60 cursor-not-allowed' : ''"
              >
                <IconLoader2 v-if="isSaving" :size="17" class="spin" />
                {{ isSaving ? $t('classes.saving') : $t('classes.save') }}
              </button>
            </div>
          </form>
        </div>
        </div>
      </div>
    </Teleport>

    <!-- ── Delete Confirm Modal ── -->
    <Teleport to="body">
      <div v-if="showDeleteModal"
           class="fixed inset-0 z-50 overflow-y-auto"
           style="background: var(--color-modal-overlay)">
        <div class="flex min-h-full items-center justify-center px-4 py-4">
        <div class="w-full max-w-md rounded-xl p-6 border"
             style="background: var(--color-bg-secondary); border-color: var(--color-border); box-shadow: var(--shadow-dropdown)">
          <h2 class="font-semibold mb-2" style="color: var(--color-text-primary)">
            {{ $t('classes.deleteClass') }}
          </h2>
          <p class="mb-1" style="color: var(--color-text-primary)">
            {{ $t('classes.deleteConfirm', { name: deleteTarget?.name }) }}
          </p>
          <p class="mb-5" style="color: var(--color-text-muted)">
            {{ $t('classes.deleteConfirmHint') }}
          </p>

          <div v-if="deleteError"
               class="mb-4 flex items-center gap-2 rounded-lg border px-4 py-3"
               style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
               role="alert">
            <IconAlertCircle :size="18" class="shrink-0" />
            <span>{{ deleteError }}</span>
          </div>

          <div class="flex justify-end gap-3">
            <button
              class="h-10 px-5 rounded-lg font-medium border bg-transparent"
              style="border-color: var(--color-border); color: var(--color-text-primary);"
              @click="closeModals"
              :disabled="isDeleting"
            >{{ $t('classes.cancel') }}</button>
            <button
              :disabled="isDeleting"
              class="h-10 px-5 rounded-lg font-medium flex items-center gap-2 border-0"
              style="background: var(--color-danger); color: var(--color-accent-text);"
              :class="isDeleting ? 'opacity-60 cursor-not-allowed' : ''"
              @click="onDeleteConfirm"
            >
              <IconLoader2 v-if="isDeleting" :size="17" class="spin" />
              {{ isDeleting ? $t('classes.deleting') : $t('classes.delete') }}
            </button>
          </div>
        </div>
        </div>
      </div>
    </Teleport>

    <SettingsModal v-model="showSettings" />

  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  IconMoon, IconSun, IconPlus, IconLoader2, IconAlertCircle,
  IconPencil, IconTrash, IconUsers, IconSchool, IconSettings, IconBooks,
} from '@tabler/icons-vue'
import { useAuthStore } from '@/stores/auth'
import { useClassStore } from '@/stores/class'
import LanguageSelector from '@/components/LanguageSelector.vue'
import SettingsModal from '@/components/SettingsModal.vue'
import type { ClassItem } from '@/api/client'

const { t } = useI18n()
const router = useRouter()
const auth = useAuthStore()
const classStore = useClassStore()

const isDark = ref(document.documentElement.getAttribute('data-theme') === 'dark')
const isLoggingOut = ref(false)
const showSettings = ref(false)

// ── Modal state ────────────────────────────────────────────────
const showAddModal = ref(false)
const showEditModal = ref(false)
const showDeleteModal = ref(false)
const deleteTarget = ref<ClassItem | null>(null)
const editTarget = ref<ClassItem | null>(null)

const addForm = ref({ name: '', subject_id: '' as number | '' })
const editForm = ref({ name: '', subject_id: 0 })

const isAdding = ref(false)
const isSaving = ref(false)
const isDeleting = ref(false)
const addError = ref<string | null>(null)
const editError = ref<string | null>(null)
const deleteError = ref<string | null>(null)

// ── Card colors ────────────────────────────────────────────────
const ACCENT_VARS = [
  'var(--color-card-accent-0)',
  'var(--color-card-accent-1)',
  'var(--color-card-accent-2)',
  'var(--color-card-accent-3)',
  'var(--color-card-accent-4)',
  'var(--color-card-accent-5)',
]

function cardAccentColor(id: number) {
  return ACCENT_VARS[id % ACCENT_VARS.length]
}

// ── Theme ──────────────────────────────────────────────────────
function toggleTheme() {
  isDark.value = !isDark.value
  document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
}

// ── Logout ─────────────────────────────────────────────────────
async function onLogout() {
  if (isLoggingOut.value) return
  isLoggingOut.value = true
  try {
    await auth.logoutTeacher()
    router.push('/login')
  } finally {
    isLoggingOut.value = false
  }
}

// ── Modal helpers ──────────────────────────────────────────────
function closeModals() {
  showAddModal.value = false
  showEditModal.value = false
  showDeleteModal.value = false
  deleteTarget.value = null
  editTarget.value = null
  addError.value = null
  editError.value = null
  deleteError.value = null
  addForm.value = { name: '', subject_id: '' }
}

function openAddModal() {
  closeModals()
  showAddModal.value = true
}

function openEditModal(cls: ClassItem) {
  closeModals()
  editTarget.value = cls
  editForm.value = { name: cls.name, subject_id: cls.subject_id }
  showEditModal.value = true
}

function openDeleteModal(cls: ClassItem) {
  closeModals()
  deleteTarget.value = cls
  showDeleteModal.value = true
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') closeModals()
}

// ── Actions ────────────────────────────────────────────────────
async function onAddSubmit() {
  if (isAdding.value) return
  addError.value = null
  isAdding.value = true
  try {
    await classStore.createClass(addForm.value.name, addForm.value.subject_id as number)
    closeModals()
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    addError.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isAdding.value = false
  }
}

async function onEditSubmit() {
  if (isSaving.value) return
  editError.value = null
  isSaving.value = true
  try {
    await classStore.updateClass(editTarget.value!.id, editForm.value.name, editForm.value.subject_id)
    closeModals()
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    editError.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isSaving.value = false
  }
}

async function onDeleteConfirm() {
  if (isDeleting.value) return
  deleteError.value = null
  isDeleting.value = true
  try {
    await classStore.deleteClass(deleteTarget.value!.id)
    closeModals()
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    deleteError.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isDeleting.value = false
  }
}

// ── Lifecycle ──────────────────────────────────────────────────
onMounted(async () => {
  if (!auth.teacher) {
    try { await auth.fetchTeacherMe() } catch { router.push('/login'); return }
  }
  await Promise.all([classStore.fetchClasses(), classStore.fetchSubjects()])
  document.addEventListener('keydown', onKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', onKeydown)
})
</script>

<style scoped>
.class-card {
  transition: transform 0.15s ease, box-shadow 0.15s ease, border-color 0.15s ease;
}
.class-card:hover {
  box-shadow: 0 14px 28px -16px rgba(15, 23, 42, 0.35);
  border-color: var(--color-border-strong);
}
[data-theme="dark"] .class-card:hover {
  box-shadow: 0 14px 28px -16px rgba(2, 6, 23, 0.6);
}
.session-live-badge {
  color: var(--color-danger);
  background: var(--color-danger-bg);
  border: 1px solid var(--color-danger-border);
}
</style>
