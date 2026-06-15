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
            class="flex items-center gap-2.5 w-full min-h-10 py-2 px-3 rounded-lg font-medium border-0 leading-tight"
            :style="activeSection === 'classes'
              ? { background: 'var(--color-info-bg)', color: 'var(--color-accent)', fontWeight: 600 }
              : { background: 'transparent', color: 'var(--color-text-muted)' }"
            @click="activeSection = 'classes'"
          >
            <IconLayoutGrid :size="17" class="shrink-0" />
            <span class="flex-1 text-center">{{ $t('classes.allClasses') }}</span>
          </button>
        </div>

        <div class="mx-3 mb-1 h-px" style="background: var(--color-border)"></div>

        <!-- 교사·과목 관리 섹션 -->
        <div class="p-3 flex flex-col gap-0.5">
          <button
            class="flex items-center gap-2.5 w-full min-h-10 py-2 px-3 rounded-lg font-medium border-0 leading-tight"
            :style="activeSection === 'teachers'
              ? { background: 'var(--color-info-bg)', color: 'var(--color-accent)', fontWeight: 600 }
              : { background: 'transparent', color: 'var(--color-text-muted)' }"
            @click="activeSection = 'teachers'"
          >
            <IconUsers :size="17" class="shrink-0" />
            <span class="flex-1 text-center">{{ $t('classes.teacherManage') }}</span>
          </button>
          <button
            class="flex items-center gap-2.5 w-full min-h-10 py-2 px-3 rounded-lg font-medium border-0 leading-tight"
            :style="activeSection === 'subjects'
              ? { background: 'var(--color-info-bg)', color: 'var(--color-accent)', fontWeight: 600 }
              : { background: 'transparent', color: 'var(--color-text-muted)' }"
            @click="activeSection = 'subjects'"
          >
            <IconBook :size="17" class="shrink-0" />
            <span class="flex-1 text-center">{{ $t('classes.subjectManage') }}</span>
          </button>
        </div>

        <div class="mx-3 mt-1 h-px" style="background: var(--color-border)"></div>

        <!-- 문제 은행 -->
        <div class="p-3">
          <router-link
            to="/problem-bank"
            class="flex items-center gap-2.5 w-full min-h-10 py-2 px-3 rounded-lg font-medium no-underline leading-tight"
            style="color: var(--color-text-muted)"
            active-class=""
          >
            <IconBooks :size="17" class="shrink-0" />
            <span class="flex-1 text-center">{{ $t('problems.title') }}</span>
          </router-link>
        </div>

        <!-- 스페이서 -->
        <div class="flex-1"></div>

        <div class="mx-3 h-px" style="background: var(--color-border)"></div>

        <!-- LMS 기본 설정 (하단 고정) -->
        <div class="p-3">
          <button
            class="flex items-center gap-2.5 w-full min-h-10 py-2 px-3 rounded-lg font-medium border-0 leading-tight"
            :style="activeSection === 'appSettings'
              ? { background: 'var(--color-info-bg)', color: 'var(--color-accent)', fontWeight: 600 }
              : { background: 'transparent', color: 'var(--color-text-muted)' }"
            @click="openAppSettings"
          >
            <IconAdjustments :size="17" class="shrink-0" />
            <span class="flex-1 text-center">{{ $t('admin.appSettings') }}</span>
          </button>
        </div>

      </aside>

      <!-- ── 메인 콘텐츠 ── -->
      <main class="flex-1 overflow-y-auto px-6 pt-4 pb-16">

        <!-- 수업 전체 탭 -->
        <template v-if="activeSection === 'classes'">
          <div class="flex items-center justify-between mb-6 min-h-9">
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
                  <th scope="col" class="px-5 py-3 text-left font-semibold" style="color: var(--color-text-muted)">
                    {{ $t('classes.className') }}
                  </th>
                  <th scope="col" class="px-5 py-3 text-left font-semibold hidden sm:table-cell"
                      style="color: var(--color-text-muted)">
                    {{ $t('classes.subject') }}
                  </th>
                  <th scope="col" class="px-5 py-3 text-left font-semibold hidden md:table-cell"
                      style="color: var(--color-text-muted)">
                    {{ $t('classes.teacher') }}
                  </th>
                  <th scope="col" class="px-5 py-3 text-left font-semibold"
                      style="color: var(--color-text-muted)">
                    {{ $t('classes.studentCount') }}
                  </th>
                  <th scope="col" class="px-5 py-3 w-16"></th>
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
                    {{ cls.teacher_name }}
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
          <div class="flex items-center justify-between mb-6 min-h-9">
            <h2 class="font-semibold tracking-widest uppercase"
                style="color: var(--color-text-muted)">{{ $t('admin.teachers') }}</h2>
            <div class="flex items-center gap-2">
              <button
                class="h-9 px-4 rounded-lg flex items-center gap-2 font-medium"
                style="border: 1px solid var(--color-border); color: var(--color-text-primary); background: var(--color-bg-secondary)"
                @click="showImportTeachersModal = true"
              >
                <IconUpload :size="17" />
                {{ $t('admin.importTeachers') }}
              </button>
              <button
                class="h-9 px-4 rounded-lg flex items-center gap-2 font-medium"
                style="background: var(--color-accent); color: var(--color-accent-text); border: none"
                @click="openAddTeacherModal"
              >
                <IconPlus :size="17" />
                {{ $t('admin.addTeacher') }}
              </button>
            </div>
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
                  <th scope="col" class="px-5 py-3 text-left font-semibold" style="color: var(--color-text-muted)">
                    {{ $t('admin.teacherName') }}
                  </th>
                  <th scope="col" class="px-5 py-3 text-left font-semibold hidden sm:table-cell"
                      style="color: var(--color-text-muted)">
                    {{ $t('admin.teacherUsername') }}
                  </th>
                  <th scope="col" class="px-5 py-3 text-left font-semibold" style="color: var(--color-text-muted)">
                    {{ $t('admin.teacherRole') }}
                  </th>
                  <th scope="col" class="px-5 py-3 text-left font-semibold hidden lg:table-cell"
                      style="color: var(--color-text-muted)">
                    {{ $t('admin.createdAt') }}
                  </th>
                  <th scope="col" class="px-5 py-3 w-20"></th>
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

        <!-- LMS 기본 설정 탭 -->
        <template v-else-if="activeSection === 'appSettings'">
          <div class="flex items-center justify-between mb-6 min-h-9">
            <h2 class="font-semibold tracking-widest uppercase"
                style="color: var(--color-text-muted)">{{ $t('admin.appSettings') }}</h2>
          </div>

          <div v-if="appSettingsLoading"
               class="flex items-center gap-3 py-8"
               style="color: var(--color-text-muted)">
            <IconLoader2 :size="20" class="spin" />
            <span>{{ $t('common.loading') }}</span>
          </div>

          <div v-else class="max-w-lg flex flex-col gap-6">

            <div v-if="appSettingsError"
                 class="flex items-center gap-2 rounded-xl border px-5 py-4"
                 style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
                 role="alert">
              <IconAlertCircle :size="18" class="shrink-0" />
              <span>{{ appSettingsError }}</span>
            </div>

            <div v-if="appSettingsSuccess"
                 class="flex items-center gap-2 rounded-xl border px-5 py-4"
                 style="background: var(--color-success-bg); border-color: var(--color-success); color: var(--color-success)"
                 role="status">
              <IconCheck :size="18" class="shrink-0" />
              <span>{{ $t('admin.appSettingsSaveSuccess') }}</span>
            </div>

            <!-- 학교 이름 -->
            <div class="flex flex-col gap-2">
              <label class="font-medium" style="color: var(--color-text-primary)">
                {{ $t('admin.appSettingsSchoolName') }}
              </label>
              <input
                v-model="appSettingsForm.school_name"
                type="text"
                class="h-12 w-full px-4 rounded-lg border outline-none"
                style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)"
                :disabled="isSavingAppSettings"
              />
            </div>

            <!-- 기본 언어 -->
            <div class="flex flex-col gap-2">
              <label class="font-medium" style="color: var(--color-text-primary)">
                {{ $t('admin.appSettingsDefaultLocale') }}
              </label>
              <p style="color: var(--color-text-muted)">{{ $t('admin.appSettingsLocaleNote') }}</p>
              <select
                v-model="appSettingsForm.locale"
                class="h-12 w-full px-4 rounded-lg border outline-none"
                style="background: var(--color-bg-primary); border-color: var(--color-border); color: var(--color-text-primary)"
                :disabled="isSavingAppSettings"
              >
                <option value="ko">{{ $t('admin.localeKo') }}</option>
                <option value="en">{{ $t('admin.localeEn') }}</option>
              </select>
            </div>

            <button
              class="h-12 px-6 rounded-lg font-medium flex items-center justify-center gap-2"
              style="background: var(--color-accent); color: var(--color-accent-text); border: none"
              :disabled="isSavingAppSettings"
              :class="isSavingAppSettings ? 'opacity-60 cursor-not-allowed' : ''"
              @click="onSaveAppSettings"
            >
              <IconLoader2 v-if="isSavingAppSettings" :size="18" class="spin" />
              {{ isSavingAppSettings ? $t('admin.saving') : $t('admin.save') }}
            </button>

          </div>
        </template>

        <!-- 과목 관리 탭 -->
        <template v-else-if="activeSection === 'subjects'">
          <div class="flex items-center justify-between mb-6 min-h-9">
            <h2 class="font-semibold tracking-widest uppercase"
                style="color: var(--color-text-muted)">{{ $t('admin.subjects') }}</h2>
            <div class="flex items-center gap-2">
              <button
                class="h-9 px-4 rounded-lg flex items-center gap-2 font-medium"
                style="border: 1px solid var(--color-border); color: var(--color-text-primary); background: var(--color-bg-secondary)"
                @click="showImportSubjectsModal = true"
              >
                <IconUpload :size="17" />
                {{ $t('admin.importSubjects') }}
              </button>
              <button
                class="h-9 px-4 rounded-lg flex items-center gap-2 font-medium"
                style="background: var(--color-accent); color: var(--color-accent-text); border: none"
                @click="openAddSubjectModal"
              >
                <IconPlus :size="17" />
                {{ $t('admin.addSubject') }}
              </button>
            </div>
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
           style="background: var(--color-modal-overlay)">
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
           style="background: var(--color-modal-overlay)">
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
           style="background: var(--color-modal-overlay)">
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
                    style="background: var(--color-danger); color: var(--color-accent-text); border: none"
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
           style="background: var(--color-modal-overlay)">
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
           style="background: var(--color-modal-overlay)">
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
                    style="background: var(--color-danger); color: var(--color-accent-text); border: none"
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

    <!-- ── Import Teachers Modal ── -->
    <ImportModal
      v-model:show="showImportTeachersModal"
      :title="$t('admin.importTeachers')"
      template-filename="teachers_template"
      :template-headers="teacherTemplateHeaders"
      :template-sample="[['Jonghwan Lee', 'teacher1', 'password1', 'teacher'],['Admin', 'admin', 'password1', 'admin']]"
      :synonym-map="teacherSynonymMap"
      :required-fields="['name', 'username', 'password']"
      :columns="teacherImportColumns"
      :on-import="handleImportTeachers"
    />

    <!-- ── Import Subjects Modal ── -->
    <ImportModal
      v-model:show="showImportSubjectsModal"
      :title="$t('admin.importSubjects')"
      template-filename="subjects_template"
      :template-headers="subjectTemplateHeaders"
      :template-sample="[['Programming'], ['Mathematics']]"
      :synonym-map="subjectSynonymMap"
      :required-fields="['name']"
      :columns="subjectImportColumns"
      :on-import="handleImportSubjects"
    />

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  IconMoon, IconSun, IconPlus, IconUpload, IconLoader2, IconAlertCircle, IconPencil, IconTrash,
  IconSettings, IconUsers, IconBook, IconLayoutGrid, IconChevronRight, IconBooks,
  IconAdjustments, IconCheck,
} from '@tabler/icons-vue'
import { useAuthStore } from '@/stores/auth'
import { useAdminStore } from '@/stores/admin'
import { useClassStore } from '@/stores/class'
import LanguageSelector from '@/components/LanguageSelector.vue'
import SettingsModal from '@/components/SettingsModal.vue'
import ImportModal from '@/components/ImportModal.vue'
import { api } from '@/api/client'
import type { AdminTeacher, Subject } from '@/api/client'
import type { SynonymMap } from '@/utils/excelImport'

const { t } = useI18n()
const router = useRouter()
const auth = useAuthStore()
const adminStore = useAdminStore()
const classStore = useClassStore()

const isDark = ref(document.documentElement.getAttribute('data-theme') === 'dark')
const isLoggingOut = ref(false)
const showSettings = ref(false)
const activeSection = ref<'classes' | 'teachers' | 'subjects' | 'appSettings'>('classes')

// ── Modal visibility flags ─────────────────────────────────────
const showAddTeacherModal = ref(false)
const showEditTeacherModal = ref(false)
const showDeleteTeacherModal = ref(false)
const showAddSubjectModal = ref(false)
const showDeleteSubjectModal = ref(false)
const showImportTeachersModal = ref(false)
const showImportSubjectsModal = ref(false)

// ── Import config ──────────────────────────────────────────────
const teacherSynonymMap: SynonymMap = {
  name: ['name', '이름', '성명'],
  username: ['username', 'id', '아이디'],
  password: ['password', 'pw', '비밀번호', '패스워드'],
  role: ['role', '역할', '권한'],
}
const subjectSynonymMap: SynonymMap = {
  name: ['name', 'subject', '과목명', '과목', '과목 이름', 'subject name'],
}
const teacherImportColumns = [
  { key: 'name', labelKey: 'admin.teacherName' },
  { key: 'username', labelKey: 'admin.teacherUsername' },
  { key: 'password', labelKey: 'admin.teacherPassword' },
  { key: 'role', labelKey: 'admin.teacherRole' },
]
const subjectImportColumns = [
  { key: 'name', labelKey: 'admin.subjectName' },
]
const teacherTemplateHeaders = computed(() => [
  t('admin.teacherName'),
  t('admin.teacherUsername'),
  t('admin.teacherPassword'),
  t('admin.teacherRole'),
])
const subjectTemplateHeaders = computed(() => [t('admin.subjectName')])

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

// ── App settings state ─────────────────────────────────────────
const appSettingsForm = ref({ school_name: '', locale: 'ko' })
const appSettingsLoading = ref(false)
const isSavingAppSettings = ref(false)
const appSettingsError = ref<string | null>(null)
const appSettingsSuccess = ref(false)

async function openAppSettings() {
  activeSection.value = 'appSettings'
  appSettingsError.value = null
  appSettingsSuccess.value = false
  appSettingsLoading.value = true
  try {
    const res = await api.admin.getAppSettings()
    appSettingsForm.value = { school_name: res.school_name, locale: res.locale }
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    appSettingsError.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    appSettingsLoading.value = false
  }
}

async function onSaveAppSettings() {
  if (isSavingAppSettings.value) return
  isSavingAppSettings.value = true
  appSettingsError.value = null
  appSettingsSuccess.value = false
  try {
    await api.admin.updateAppSettings(appSettingsForm.value)
    appSettingsSuccess.value = true
    // 학교 이름이 바뀌었을 수 있으므로 store 갱신
    await auth.fetchSchoolName()
  } catch (e) {
    const code = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    appSettingsError.value = t(`errors.${code}`, t('errors.ERR_UNKNOWN'))
  } finally {
    isSavingAppSettings.value = false
  }
}

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
  showImportTeachersModal.value = false
  showImportSubjectsModal.value = false
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

// ── Import handlers ────────────────────────────────────────────
async function handleImportTeachers(rows: Record<string, string>[]) {
  const data = rows.map((r) => ({
    name: r.name,
    username: r.username,
    password: r.password,
    role: r.role || undefined,
  }))
  return await adminStore.importTeachers(data)
}

async function handleImportSubjects(rows: Record<string, string>[]) {
  const data = rows.map((r) => ({ name: r.name }))
  return await adminStore.importSubjects(data)
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
