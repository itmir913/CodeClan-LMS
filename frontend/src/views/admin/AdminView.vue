<template>
  <div class="min-h-screen flex flex-col" style="background: var(--color-bg-primary)">

    <!-- Top Nav -->
    <header class="sticky top-0 z-30 h-16 border-b flex-shrink-0"
            style="background: var(--color-bg-secondary); border-color: var(--color-border)">
      <div class="h-full max-w-full flex items-center justify-between px-6">

        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg flex items-center justify-center font-bold text-white shrink-0"
               style="background: var(--color-accent)">C</div>
          <span class="font-semibold" style="color: var(--color-text-primary)">CodeClan LMS</span>
          <span class="rounded-full px-2.5 py-0.5 font-semibold"
                style="background: var(--color-accent); color: var(--color-accent-text)">
            {{ $t('admin.adminBadge') }}
          </span>
        </div>

        <div class="flex items-center gap-2">
          <span class="hidden sm:inline font-medium mr-1" style="color: var(--color-text-primary)">
            {{ $t('auth.teacherGreeting', { name: auth.teacher?.name }) }}
          </span>

          <button
            class="w-9 h-9 p-0 rounded-lg flex items-center justify-center"
            style="border: 1px solid var(--color-border); color: var(--color-text-muted); background: transparent"
            @click="showSettings = true"
            :aria-label="$t('common.settings')"
          >
            <IconSettings :size="18" />
          </button>

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

          <button
            class="h-9 px-3 rounded-lg font-medium"
            style="border: 1px solid var(--color-border); color: var(--color-text-muted); background: transparent"
            @click="onLogout"
            :disabled="isLoggingOut"
          >
            {{ $t('common.logout') }}
          </button>
        </div>

      </div>
    </header>

    <!-- Body: Sidebar + Content -->
    <div class="flex flex-1 min-h-0">

      <!-- ── 사이드바 ── -->
      <aside class="w-56 flex-shrink-0 border-r overflow-y-auto flex flex-col"
             style="background: var(--color-bg-secondary); border-color: var(--color-border)">

        <!-- 수업 전체 섹션 -->
        <div class="p-3 pt-4">
          <button
            class="flex items-center gap-2.5 w-full h-10 px-3 rounded-lg font-medium border-0 text-left"
            :style="activeSection === 'classes'
              ? { background: 'var(--color-info-bg)', color: 'var(--color-accent)', fontWeight: 600 }
              : { background: 'transparent', color: 'var(--color-text-muted)' }"
            @click="activeSection = 'classes'"
          >
            <IconLayoutGrid :size="17" />
            {{ $t('classes.allClasses') }}
          </button>
        </div>

        <div class="mx-3 mb-1" style="height: 1px; background: var(--color-border)"></div>

        <!-- 교사·과목 관리 섹션 -->
        <div class="p-3 flex flex-col gap-0.5">
          <button
            class="flex items-center gap-2.5 w-full h-10 px-3 rounded-lg font-medium border-0 text-left"
            :style="activeSection === 'teachers'
              ? { background: 'var(--color-info-bg)', color: 'var(--color-accent)', fontWeight: 600 }
              : { background: 'transparent', color: 'var(--color-text-muted)' }"
            @click="activeSection = 'teachers'"
          >
            <IconUsers :size="17" />
            {{ $t('classes.teacherManage') }}
          </button>
          <button
            class="flex items-center gap-2.5 w-full h-10 px-3 rounded-lg font-medium border-0 text-left"
            :style="activeSection === 'subjects'
              ? { background: 'var(--color-info-bg)', color: 'var(--color-accent)', fontWeight: 600 }
              : { background: 'transparent', color: 'var(--color-text-muted)' }"
            @click="activeSection = 'subjects'"
          >
            <IconBook :size="17" />
            {{ $t('classes.subjectManage') }}
          </button>
        </div>

        <!-- 수업 목록 (수업 전체 탭 선택 시) -->
        <template v-if="activeSection === 'classes'">
          <div class="mx-3" style="height: 1px; background: var(--color-border)"></div>
          <div class="p-2 flex-1 overflow-y-auto">
            <div v-if="classStore.loading"
                 class="flex items-center gap-2 px-3 py-2"
                 style="color: var(--color-text-muted)">
              <IconLoader2 :size="16" class="spin" />
            </div>
            <div v-else-if="classStore.classes.length === 0"
                 class="px-3 py-3"
                 style="color: var(--color-text-muted)">
              {{ $t('classes.noClassesAdmin') }}
            </div>
            <router-link
              v-else
              v-for="cls in classStore.classes"
              :key="cls.id"
              :to="`/classes/${cls.id}`"
              class="flex flex-col px-3 py-2.5 rounded-lg no-underline"
              style="color: var(--color-text-primary)"
              active-class=""
            >
              <span class="font-medium leading-tight" style="font-size: 16px">{{ cls.name }}</span>
              <span class="mt-0.5" style="font-size: 16px; color: var(--color-text-muted)">
                {{ cls.subject_name }}
              </span>
            </router-link>
          </div>
        </template>

      </aside>

      <!-- ── 메인 콘텐츠 ── -->
      <main class="flex-1 overflow-y-auto px-6 py-6 pb-16">

        <!-- 수업 전체 탭 -->
        <template v-if="activeSection === 'classes'">
          <div class="flex items-center justify-between mb-6">
            <h2 class="font-semibold tracking-widest uppercase"
                style="color: var(--color-text-muted)">{{ $t('classes.allClasses') }}</h2>
          </div>

          <div v-if="classStore.loading"
               class="flex items-center gap-3 py-8"
               style="color: var(--color-text-muted)">
            <IconLoader2 :size="20" class="spin" />
            <span>{{ $t('common.loading') }}</span>
          </div>

          <div v-else-if="classStore.error"
               class="flex items-center gap-3 rounded-xl border px-5 py-4"
               style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
               role="alert">
            <IconAlertCircle :size="20" class="shrink-0" />
            <span>{{ $t(`errors.${classStore.error}`, $t('errors.ERR_UNKNOWN')) }}</span>
          </div>

          <div v-else-if="classStore.classes.length === 0"
               class="py-8 text-center"
               style="color: var(--color-text-muted)">
            {{ $t('classes.noClassesAdmin') }}
          </div>

          <div v-else class="rounded-xl border overflow-hidden"
               style="border-color: var(--color-border)">
            <table class="w-full">
              <thead>
                <tr style="background: var(--color-bg-tertiary); border-bottom: 1px solid var(--color-border)">
                  <th class="px-5 py-3 text-left font-semibold" style="color: var(--color-text-muted)">
                    {{ $t('classes.className') }}
                  </th>
                  <th class="px-5 py-3 text-left font-semibold hidden sm:table-cell"
                      style="color: var(--color-text-muted)">
                    {{ $t('classes.subject') }}
                  </th>
                  <th class="px-5 py-3 text-left font-semibold hidden md:table-cell"
                      style="color: var(--color-text-muted)">
                    {{ $t('admin.teacherName') }}
                  </th>
                  <th class="px-5 py-3 text-left font-semibold"
                      style="color: var(--color-text-muted)">
                    {{ $t('classes.students') }}
                  </th>
                  <th class="px-5 py-3 w-16"></th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="cls in classStore.classes"
                  :key="cls.id"
                  class="border-t"
                  style="border-color: var(--color-border)"
                >
                  <td class="px-5 py-3 font-medium" style="color: var(--color-text-primary)">
                    {{ cls.name }}
                  </td>
                  <td class="px-5 py-3 hidden sm:table-cell" style="color: var(--color-text-muted)">
                    {{ cls.subject_name }}
                  </td>
                  <td class="px-5 py-3 hidden md:table-cell" style="color: var(--color-text-muted)">
                    {{ cls.teacher_id }}
                  </td>
                  <td class="px-5 py-3" style="color: var(--color-text-muted)">
                    {{ $t('classes.students', { count: cls.student_count }) }}
                  </td>
                  <td class="px-5 py-3">
                    <router-link
                      :to="`/classes/${cls.id}`"
                      class="w-8 h-8 rounded-lg flex items-center justify-center no-underline"
                      style="border: 1px solid var(--color-border); color: var(--color-text-muted); background: transparent"
                    >
                      <IconChevronRight :size="14" />
                    </router-link>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>

        <!-- 교사 관리 탭 -->
        <template v-else-if="activeSection === 'teachers'">
          <div class="flex items-center justify-between mb-6">
            <h2 class="font-semibold tracking-widest uppercase"
                style="color: var(--color-text-muted)">{{ $t('admin.teachers') }}</h2>
            <button
              class="h-9 px-4 rounded-lg flex items-center gap-2 font-medium"
              style="background: var(--color-accent); color: var(--color-accent-text); border: none"
              @click="openAddTeacherModal"
            >
              <IconPlus :size="17" />
              {{ $t('admin.addTeacher') }}
            </button>
          </div>

          <!-- Loading -->
          <div v-if="adminStore.loading"
               class="flex items-center gap-3 py-8"
               style="color: var(--color-text-muted)">
            <IconLoader2 :size="20" class="spin" />
            <span>{{ $t('common.loading') }}</span>
          </div>

          <!-- Error -->
          <div v-else-if="adminStore.error"
               class="flex items-center gap-3 rounded-xl border px-5 py-4"
               style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
               role="alert">
            <IconAlertCircle :size="20" class="shrink-0" />
            <span>{{ $t(`errors.${adminStore.error}`, $t('errors.ERR_UNKNOWN')) }}</span>
            <button
              class="ml-auto h-8 px-3 rounded-lg font-medium"
              style="background: transparent; border: 1px solid var(--color-danger-border); color: var(--color-danger)"
              @click="adminStore.fetchTeachers()"
            >{{ $t('common.retry') }}</button>
          </div>

          <!-- Empty -->
          <div v-else-if="adminStore.teachers.length === 0"
               class="py-8 text-center"
               style="color: var(--color-text-muted)">
            {{ $t('admin.noTeachers') }}
          </div>

          <!-- Table -->
          <div v-else class="rounded-xl border overflow-hidden"
               style="border-color: var(--color-border)">
            <table class="w-full">
              <thead>
                <tr style="background: var(--color-bg-tertiary); border-bottom: 1px solid var(--color-border)">
                  <th class="px-5 py-3 text-left font-semibold" style="color: var(--color-text-muted)">
                    {{ $t('admin.teacherName') }}
                  </th>
                  <th class="px-5 py-3 text-left font-semibold hidden sm:table-cell"
                      style="color: var(--color-text-muted)">
                    {{ $t('admin.teacherUsername') }}
                  </th>
                  <th class="px-5 py-3 text-left font-semibold" style="color: var(--color-text-muted)">
                    {{ $t('admin.teacherRole') }}
                  </th>
                  <th class="px-5 py-3 text-left font-semibold hidden lg:table-cell"
                      style="color: var(--color-text-muted)">
                    {{ $t('admin.createdAt') }}
                  </th>
                  <th class="px-5 py-3 w-20"></th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="teacher in adminStore.teachers"
                  :key="teacher.id"
                  class="border-t"
                  style="border-color: var(--color-border)"
                >
                  <td class="px-5 py-3 font-medium" style="color: var(--color-text-primary)">
                    {{ teacher.name }}
                  </td>
                  <td class="px-5 py-3 hidden sm:table-cell" style="color: var(--color-text-muted)">
                    {{ teacher.username }}
                  </td>
                  <td class="px-5 py-3">
                    <span
                      class="inline-block rounded-full px-2.5 py-0.5 font-medium"
                      :style="teacher.role === 'admin'
                        ? { background: 'var(--color-info-bg)', color: 'var(--color-info)' }
                        : { background: 'var(--color-bg-tertiary)', color: 'var(--color-text-muted)' }"
                    >
                      {{ teacher.role === 'admin' ? $t('admin.roleAdmin') : $t('admin.roleTeacher') }}
                    </span>
                  </td>
                  <td class="px-5 py-3 hidden lg:table-cell" style="color: var(--color-text-muted)">
                    {{ teacher.created_at.slice(0, 10) }}
                  </td>
                  <td class="px-5 py-3">
                    <div class="flex items-center gap-1">
                      <button
                        class="w-8 h-8 p-0 rounded-lg flex items-center justify-center"
                        style="background: transparent; border: 1px solid var(--color-border); color: var(--color-text-muted)"
                        @click="openEditTeacherModal(teacher)"
                        :aria-label="$t('admin.editTeacher')"
                      >
                        <IconPencil :size="14" />
                      </button>
                      <button
                        class="w-8 h-8 p-0 rounded-lg flex items-center justify-center"
                        style="background: transparent; border: 1px solid var(--color-border); color: var(--color-text-muted)"
                        @click="openDeleteTeacherModal(teacher)"
                        :aria-label="$t('admin.deleteTeacher')"
                        :disabled="teacher.id === auth.teacher?.id"
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

        <!-- 과목 관리 탭 -->
        <template v-else-if="activeSection === 'subjects'">
          <div class="flex items-center justify-between mb-6">
            <h2 class="font-semibold tracking-widest uppercase"
                style="color: var(--color-text-muted)">{{ $t('admin.subjects') }}</h2>
            <button
              class="h-9 px-4 rounded-lg flex items-center gap-2 font-medium"
              style="background: var(--color-accent); color: var(--color-accent-text); border: none"
              @click="openAddSubjectModal"
            >
              <IconPlus :size="17" />
              {{ $t('admin.addSubject') }}
            </button>
          </div>

          <div v-if="adminStore.subjects.length === 0"
               class="py-8 text-center"
               style="color: var(--color-text-muted)">
            {{ $t('admin.noSubjects') }}
          </div>

          <div v-else class="rounded-xl border overflow-hidden"
               style="border-color: var(--color-border)">
            <table class="w-full">
              <tbody>
                <tr
                  v-for="subject in adminStore.subjects"
                  :key="subject.id"
                  class="border-t first:border-t-0"
                  style="border-color: var(--color-border)"
                >
                  <td class="px-5 py-3 font-medium" style="color: var(--color-text-primary)">
                    {{ subject.name }}
                  </td>
                  <td class="px-5 py-3 w-16">
                    <button
                      class="w-8 h-8 p-0 rounded-lg flex items-center justify-center"
                      style="background: transparent; border: 1px solid var(--color-border); color: var(--color-text-muted)"
                      @click="openDeleteSubjectModal(subject)"
                      :aria-label="$t('admin.deleteTeacher')"
                    >
                      <IconTrash :size="14" />
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>

      </main>
    </div>

    <!-- ── Add Teacher Modal ── -->
    <Teleport to="body">
      <div v-if="showAddTeacherModal"
           class="fixed inset-0 z-50 flex items-center justify-center px-4"
           style="background: rgba(0,0,0,0.45)">
        <div class="w-full max-w-md rounded-xl p-6"
             style="background: var(--color-bg-secondary); border: 1px solid var(--color-border); box-shadow: var(--shadow-dropdown)">
          <h2 class="font-semibold mb-5" style="color: var(--color-text-primary)">
            {{ $t('admin.addTeacher') }}
          </h2>
          <form @submit.prevent="onAddTeacherSubmit" novalidate class="flex flex-col gap-4">
            <div class="flex flex-col gap-2">
              <label class="font-medium" style="color: var(--color-text-primary)">{{ $t('admin.teacherName') }}</label>
              <input v-model="addTeacherForm.name" type="text" :disabled="isAddingTeacher"
                     class="h-12 w-full px-4 rounded-lg border outline-none"
                     style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)" />
            </div>
            <div class="flex flex-col gap-2">
              <label class="font-medium" style="color: var(--color-text-primary)">{{ $t('admin.teacherUsername') }}</label>
              <input v-model="addTeacherForm.username" type="text" autocomplete="off" :disabled="isAddingTeacher"
                     class="h-12 w-full px-4 rounded-lg border outline-none"
                     style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)" />
            </div>
            <div class="flex flex-col gap-2">
              <label class="font-medium" style="color: var(--color-text-primary)">{{ $t('admin.teacherPassword') }}</label>
              <input v-model="addTeacherForm.password" type="password" autocomplete="new-password" :disabled="isAddingTeacher"
                     class="h-12 w-full px-4 rounded-lg border outline-none"
                     style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)" />
            </div>
            <div class="flex flex-col gap-2">
              <label class="font-medium" style="color: var(--color-text-primary)">{{ $t('admin.teacherRole') }}</label>
              <select v-model="addTeacherForm.role" :disabled="isAddingTeacher"
                      class="h-12 w-full px-4 rounded-lg border outline-none"
                      style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)">
                <option value="teacher">{{ $t('admin.roleTeacher') }}</option>
                <option value="admin">{{ $t('admin.roleAdmin') }}</option>
              </select>
            </div>
            <div v-if="addTeacherError"
                 class="flex items-center gap-2 rounded-lg border px-4 py-3"
                 style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
                 role="alert">
              <IconAlertCircle :size="18" class="shrink-0" />
              <span>{{ addTeacherError }}</span>
            </div>
            <div class="flex justify-end gap-3 pt-1">
              <button type="button" class="h-10 px-5 rounded-lg font-medium"
                      style="border: 1px solid var(--color-border); color: var(--color-text-primary); background: transparent"
                      @click="closeModals">{{ $t('admin.cancel') }}</button>
              <button type="submit" :disabled="isAddingTeacher"
                      class="h-10 px-5 rounded-lg font-medium flex items-center gap-2"
                      style="background: var(--color-accent); color: var(--color-accent-text); border: none"
                      :class="isAddingTeacher ? 'opacity-60 cursor-not-allowed' : ''">
                <IconLoader2 v-if="isAddingTeacher" :size="17" class="spin" />
                {{ isAddingTeacher ? $t('admin.adding') : $t('admin.add') }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </Teleport>

    <!-- ── Edit Teacher Modal ── -->
    <Teleport to="body">
      <div v-if="showEditTeacherModal"
           class="fixed inset-0 z-50 flex items-center justify-center px-4"
           style="background: rgba(0,0,0,0.45)">
        <div class="w-full max-w-md rounded-xl p-6"
             style="background: var(--color-bg-secondary); border: 1px solid var(--color-border); box-shadow: var(--shadow-dropdown)">
          <h2 class="font-semibold mb-5" style="color: var(--color-text-primary)">
            {{ $t('admin.editTeacher') }}
          </h2>
          <form @submit.prevent="onEditTeacherSubmit" novalidate class="flex flex-col gap-4">
            <div class="flex flex-col gap-2">
              <label class="font-medium" style="color: var(--color-text-primary)">{{ $t('admin.teacherName') }}</label>
              <input v-model="editTeacherForm.name" type="text" :disabled="isSavingTeacher"
                     class="h-12 w-full px-4 rounded-lg border outline-none"
                     style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)" />
            </div>
            <div class="flex flex-col gap-2">
              <label class="font-medium" style="color: var(--color-text-primary)">{{ $t('admin.teacherRole') }}</label>
              <select v-model="editTeacherForm.role" :disabled="isSavingTeacher"
                      class="h-12 w-full px-4 rounded-lg border outline-none"
                      style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)">
                <option value="teacher">{{ $t('admin.roleTeacher') }}</option>
                <option value="admin">{{ $t('admin.roleAdmin') }}</option>
              </select>
            </div>
            <div class="flex flex-col gap-2">
              <label class="font-medium" style="color: var(--color-text-primary)">
                {{ $t('admin.teacherNewPassword') }}
              </label>
              <p class="mt-0.5" style="color: var(--color-text-muted)">{{ $t('admin.teacherNewPasswordHint') }}</p>
              <input v-model="editTeacherForm.password" type="password" autocomplete="new-password" :disabled="isSavingTeacher"
                     class="h-12 w-full px-4 rounded-lg border outline-none"
                     style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)" />
            </div>
            <div v-if="editTeacherError"
                 class="flex items-center gap-2 rounded-lg border px-4 py-3"
                 style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
                 role="alert">
              <IconAlertCircle :size="18" class="shrink-0" />
              <span>{{ editTeacherError }}</span>
            </div>
            <div class="flex justify-end gap-3 pt-1">
              <button type="button" class="h-10 px-5 rounded-lg font-medium"
                      style="border: 1px solid var(--color-border); color: var(--color-text-primary); background: transparent"
                      @click="closeModals">{{ $t('admin.cancel') }}</button>
              <button type="submit" :disabled="isSavingTeacher"
                      class="h-10 px-5 rounded-lg font-medium flex items-center gap-2"
                      style="background: var(--color-accent); color: var(--color-accent-text); border: none"
                      :class="isSavingTeacher ? 'opacity-60 cursor-not-allowed' : ''">
                <IconLoader2 v-if="isSavingTeacher" :size="17" class="spin" />
                {{ isSavingTeacher ? $t('admin.saving') : $t('admin.save') }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </Teleport>

    <!-- ── Delete Teacher Modal ── -->
    <Teleport to="body">
      <div v-if="showDeleteTeacherModal"
           class="fixed inset-0 z-50 flex items-center justify-center px-4"
           style="background: rgba(0,0,0,0.45)">
        <div class="w-full max-w-sm rounded-xl p-6"
             style="background: var(--color-bg-secondary); border: 1px solid var(--color-border); box-shadow: var(--shadow-dropdown)">
          <h2 class="font-semibold mb-2" style="color: var(--color-text-primary)">{{ $t('admin.deleteTeacher') }}</h2>
          <p class="mb-1" style="color: var(--color-text-primary)">
            {{ $t('admin.deleteTeacherConfirm', { name: deleteTeacherTarget?.name }) }}
          </p>
          <p class="mb-5" style="color: var(--color-text-muted)">{{ $t('admin.deleteTeacherConfirmHint') }}</p>
          <div v-if="deleteTeacherError"
               class="mb-4 flex items-center gap-2 rounded-lg border px-4 py-3"
               style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
               role="alert">
            <IconAlertCircle :size="18" class="shrink-0" />
            <span>{{ deleteTeacherError }}</span>
          </div>
          <div class="flex justify-end gap-3">
            <button class="h-10 px-5 rounded-lg font-medium"
                    style="border: 1px solid var(--color-border); color: var(--color-text-primary); background: transparent"
                    @click="closeModals" :disabled="isDeletingTeacher">{{ $t('admin.cancel') }}</button>
            <button :disabled="isDeletingTeacher"
                    class="h-10 px-5 rounded-lg font-medium flex items-center gap-2"
                    style="background: var(--color-danger); color: #fff; border: none"
                    :class="isDeletingTeacher ? 'opacity-60 cursor-not-allowed' : ''"
                    @click="onDeleteTeacherConfirm">
              <IconLoader2 v-if="isDeletingTeacher" :size="17" class="spin" />
              {{ isDeletingTeacher ? $t('admin.deleting') : $t('admin.delete') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- ── Add Subject Modal ── -->
    <Teleport to="body">
      <div v-if="showAddSubjectModal"
           class="fixed inset-0 z-50 flex items-center justify-center px-4"
           style="background: rgba(0,0,0,0.45)">
        <div class="w-full max-w-sm rounded-xl p-6"
             style="background: var(--color-bg-secondary); border: 1px solid var(--color-border); box-shadow: var(--shadow-dropdown)">
          <h2 class="font-semibold mb-5" style="color: var(--color-text-primary)">{{ $t('admin.addSubject') }}</h2>
          <form @submit.prevent="onAddSubjectSubmit" novalidate class="flex flex-col gap-4">
            <div class="flex flex-col gap-2">
              <label class="font-medium" style="color: var(--color-text-primary)">{{ $t('admin.subjectName') }}</label>
              <input v-model="addSubjectName" type="text" :placeholder="$t('admin.subjectNamePlaceholder')"
                     :disabled="isAddingSubject"
                     class="h-12 w-full px-4 rounded-lg border outline-none"
                     style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)" />
            </div>
            <div v-if="addSubjectError"
                 class="flex items-center gap-2 rounded-lg border px-4 py-3"
                 style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
                 role="alert">
              <IconAlertCircle :size="18" class="shrink-0" />
              <span>{{ addSubjectError }}</span>
            </div>
            <div class="flex justify-end gap-3 pt-1">
              <button type="button" class="h-10 px-5 rounded-lg font-medium"
                      style="border: 1px solid var(--color-border); color: var(--color-text-primary); background: transparent"
                      @click="closeModals">{{ $t('admin.cancel') }}</button>
              <button type="submit" :disabled="isAddingSubject"
                      class="h-10 px-5 rounded-lg font-medium flex items-center gap-2"
                      style="background: var(--color-accent); color: var(--color-accent-text); border: none"
                      :class="isAddingSubject ? 'opacity-60 cursor-not-allowed' : ''">
                <IconLoader2 v-if="isAddingSubject" :size="17" class="spin" />
                {{ isAddingSubject ? $t('admin.adding') : $t('admin.add') }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </Teleport>

    <!-- ── Delete Subject Modal ── -->
    <Teleport to="body">
      <div v-if="showDeleteSubjectModal"
           class="fixed inset-0 z-50 flex items-center justify-center px-4"
           style="background: rgba(0,0,0,0.45)">
        <div class="w-full max-w-sm rounded-xl p-6"
             style="background: var(--color-bg-secondary); border: 1px solid var(--color-border); box-shadow: var(--shadow-dropdown)">
          <h2 class="font-semibold mb-2" style="color: var(--color-text-primary)">{{ $t('admin.subjects') }}</h2>
          <p class="mb-1" style="color: var(--color-text-primary)">
            {{ $t('admin.deleteSubjectConfirm', { name: deleteSubjectTarget?.name }) }}
          </p>
          <p class="mb-5" style="color: var(--color-text-muted)">{{ $t('admin.deleteSubjectConfirmHint') }}</p>
          <div v-if="deleteSubjectError"
               class="mb-4 flex items-center gap-2 rounded-lg border px-4 py-3"
               style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
               role="alert">
            <IconAlertCircle :size="18" class="shrink-0" />
            <span>{{ deleteSubjectError }}</span>
          </div>
          <div class="flex justify-end gap-3">
            <button class="h-10 px-5 rounded-lg font-medium"
                    style="border: 1px solid var(--color-border); color: var(--color-text-primary); background: transparent"
                    @click="closeModals" :disabled="isDeletingSubject">{{ $t('admin.cancel') }}</button>
            <button :disabled="isDeletingSubject"
                    class="h-10 px-5 rounded-lg font-medium flex items-center gap-2"
                    style="background: var(--color-danger); color: #fff; border: none"
                    :class="isDeletingSubject ? 'opacity-60 cursor-not-allowed' : ''"
                    @click="onDeleteSubjectConfirm">
              <IconLoader2 v-if="isDeletingSubject" :size="17" class="spin" />
              {{ isDeletingSubject ? $t('admin.deleting') : $t('admin.delete') }}
            </button>
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
  IconMoon, IconSun, IconPlus, IconLoader2, IconAlertCircle, IconPencil, IconTrash,
  IconSettings, IconUsers, IconBook, IconLayoutGrid, IconChevronRight,
} from '@tabler/icons-vue'
import { useAuthStore } from '@/stores/auth'
import { useAdminStore } from '@/stores/admin'
import { useClassStore } from '@/stores/class'
import LanguageSelector from '@/components/LanguageSelector.vue'
import SettingsModal from '@/components/SettingsModal.vue'
import type { AdminTeacher, Subject } from '@/api/client'

const { t } = useI18n()
const router = useRouter()
const auth = useAuthStore()
const adminStore = useAdminStore()
const classStore = useClassStore()

const isDark = ref(document.documentElement.getAttribute('data-theme') === 'dark')
const isLoggingOut = ref(false)
const showSettings = ref(false)
const activeSection = ref<'classes' | 'teachers' | 'subjects'>('classes')

// ── Modal visibility flags ─────────────────────────────────────
const showAddTeacherModal = ref(false)
const showEditTeacherModal = ref(false)
const showDeleteTeacherModal = ref(false)
const showAddSubjectModal = ref(false)
const showDeleteSubjectModal = ref(false)

// ── Teacher modal state ────────────────────────────────────────
const editTeacherTarget = ref<AdminTeacher | null>(null)
const deleteTeacherTarget = ref<AdminTeacher | null>(null)
const addTeacherForm = ref({ name: '', username: '', password: '', role: 'teacher' })
const editTeacherForm = ref({ name: '', role: 'teacher', password: '' })
const isAddingTeacher = ref(false)
const isSavingTeacher = ref(false)
const isDeletingTeacher = ref(false)
const addTeacherError = ref<string | null>(null)
const editTeacherError = ref<string | null>(null)
const deleteTeacherError = ref<string | null>(null)

// ── Subject modal state ────────────────────────────────────────
const deleteSubjectTarget = ref<Subject | null>(null)
const addSubjectName = ref('')
const isAddingSubject = ref(false)
const isDeletingSubject = ref(false)
const addSubjectError = ref<string | null>(null)
const deleteSubjectError = ref<string | null>(null)

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
  showAddTeacherModal.value = false
  showEditTeacherModal.value = false
  showDeleteTeacherModal.value = false
  showAddSubjectModal.value = false
  showDeleteSubjectModal.value = false
  editTeacherTarget.value = null
  deleteTeacherTarget.value = null
  deleteSubjectTarget.value = null
  addTeacherForm.value = { name: '', username: '', password: '', role: 'teacher' }
  editTeacherForm.value = { name: '', role: 'teacher', password: '' }
  addSubjectName.value = ''
  addTeacherError.value = null
  editTeacherError.value = null
  deleteTeacherError.value = null
  addSubjectError.value = null
  deleteSubjectError.value = null
}

function openAddTeacherModal() { closeModals(); showAddTeacherModal.value = true }
function openEditTeacherModal(teacher: AdminTeacher) {
  closeModals()
  editTeacherTarget.value = teacher
  editTeacherForm.value = { name: teacher.name, role: teacher.role, password: '' }
  showEditTeacherModal.value = true
}
function openDeleteTeacherModal(teacher: AdminTeacher) {
  closeModals(); deleteTeacherTarget.value = teacher; showDeleteTeacherModal.value = true
}
function openAddSubjectModal() { closeModals(); showAddSubjectModal.value = true }
function openDeleteSubjectModal(s: Subject) {
  closeModals(); deleteSubjectTarget.value = s; showDeleteSubjectModal.value = true
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') closeModals()
}

// ── Teacher actions ────────────────────────────────────────────
async function onAddTeacherSubmit() {
  if (isAddingTeacher.value) return
  addTeacherError.value = null
  isAddingTeacher.value = true
  try {
    await adminStore.createTeacher(addTeacherForm.value)
    closeModals()
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    addTeacherError.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isAddingTeacher.value = false
  }
}

async function onEditTeacherSubmit() {
  if (isSavingTeacher.value) return
  editTeacherError.value = null
  isSavingTeacher.value = true
  try {
    const { name, role, password } = editTeacherForm.value
    await adminStore.updateTeacher(editTeacherTarget.value!.id, { name, role, password: password || undefined })
    closeModals()
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    editTeacherError.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isSavingTeacher.value = false
  }
}

async function onDeleteTeacherConfirm() {
  if (isDeletingTeacher.value) return
  deleteTeacherError.value = null
  isDeletingTeacher.value = true
  try {
    await adminStore.deleteTeacher(deleteTeacherTarget.value!.id)
    closeModals()
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    deleteTeacherError.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isDeletingTeacher.value = false
  }
}

// ── Subject actions ────────────────────────────────────────────
async function onAddSubjectSubmit() {
  if (isAddingSubject.value) return
  addSubjectError.value = null
  isAddingSubject.value = true
  try {
    await adminStore.createSubject(addSubjectName.value)
    closeModals()
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    addSubjectError.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isAddingSubject.value = false
  }
}

async function onDeleteSubjectConfirm() {
  if (isDeletingSubject.value) return
  deleteSubjectError.value = null
  isDeletingSubject.value = true
  try {
    await adminStore.deleteSubject(deleteSubjectTarget.value!.id)
    closeModals()
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    deleteSubjectError.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isDeletingSubject.value = false
  }
}

// ── Lifecycle ──────────────────────────────────────────────────
onMounted(async () => {
  if (!auth.teacher) {
    try { await auth.fetchTeacherMe() } catch { router.push('/login'); return }
  }
  await Promise.all([
    adminStore.fetchTeachers(),
    adminStore.fetchSubjects(),
    classStore.fetchClasses(),
  ])
  document.addEventListener('keydown', onKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', onKeydown)
})
</script>
