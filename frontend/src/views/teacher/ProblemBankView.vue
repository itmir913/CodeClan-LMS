<template>
  <div class="min-h-screen flex flex-col" style="background: var(--color-bg-secondary)">

    <!-- ── 헤더 ── -->
    <header class="h-15 flex items-center justify-between px-7 flex-shrink-0"
            style="background: var(--color-bg-primary); border-bottom: 1px solid var(--color-border)">
      <div class="flex items-center gap-4">
        <button class="flex items-center gap-2 h-9 px-3 rounded-lg border"
                style="background: transparent; color: var(--color-text-muted); border-color: var(--color-border)"
                @click="goBack">
          <IconArrowLeft :size="15" />
          <span>{{ auth.teacher?.role === 'admin' ? $t('admin.adminBadge') : $t('problems.myClasses') }}</span>
        </button>
        <div class="w-px h-5" style="background: var(--color-border)"></div>
        <div class="flex items-center gap-2">
          <IconBooks :size="20" style="color: var(--color-accent)" />
          <span class="text-xl font-bold" style="color: var(--color-text-primary)">{{ $t('problems.title') }}</span>
        </div>
      </div>
      <button class="flex items-center gap-2 h-9 px-4 rounded-lg font-semibold"
              style="background: var(--color-accent); color: #fff; border: none"
              @click="openCreate">
        <IconPlus :size="16" />
        <span>{{ $t('problems.newProblem') }}</span>
      </button>
    </header>

    <!-- ── 툴바 ── -->
    <div class="flex items-center gap-3 px-7 py-4 flex-wrap flex-shrink-0"
         style="background: var(--color-bg-primary); border-bottom: 1px solid var(--color-border)">
      <!-- 검색 -->
      <div class="relative flex-1" style="min-width: 180px; max-width: 340px">
        <IconSearch :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none"
                    style="color: var(--color-text-tertiary)" />
        <input
          v-model="searchQuery"
          type="text"
          class="w-full h-10 rounded-xl pl-10 pr-4 border"
          style="background: var(--color-bg-secondary); color: var(--color-text-primary); border-color: var(--color-border); font-size: 1rem"
          :placeholder="$t('problems.searchPlaceholder')"
        />
      </div>
      <!-- 유형 필터 -->
      <div class="flex items-center gap-2 flex-wrap">
        <button
          v-for="f in typeFilters"
          :key="f.value"
          class="h-8 px-4 rounded-full border font-medium transition-colors"
          :style="activeFilter === f.value
            ? 'background: var(--color-text-primary); color: var(--color-bg-primary); border-color: var(--color-text-primary)'
            : 'background: var(--color-bg-primary); color: var(--color-text-muted); border-color: var(--color-border)'"
          @click="activeFilter = f.value"
        >
          {{ f.label }}
        </button>
      </div>
      <!-- 문제 수 -->
      <span class="ml-auto font-medium" style="color: var(--color-text-tertiary); white-space: nowrap">
        {{ $t('problems.problemCount', { count: filteredProblems.length }) }}
      </span>
    </div>

    <!-- ── 문제 목록 ── -->
    <main class="flex-1 px-7 py-5">

      <!-- 로딩 -->
      <div v-if="store.loading" class="flex items-center justify-center py-16">
        <IconLoader2 :size="32" class="spin" style="color: var(--color-accent)" />
      </div>

      <!-- 에러 -->
      <div v-else-if="store.error"
           class="flex items-center gap-3 p-4 rounded-xl mb-4"
           style="background: var(--color-danger-bg); color: var(--color-danger); border: 1px solid var(--color-danger-border)">
        <IconAlertCircle :size="20" class="shrink-0" />
        <span>{{ $t(`errors.${store.error}`, $t('errors.ERR_UNKNOWN')) }}</span>
      </div>

      <!-- 빈 상태 -->
      <div v-else-if="store.problems.length === 0"
           class="flex flex-col items-center justify-center py-20 gap-3">
        <IconBooks :size="48" class="opacity-30" style="color: var(--color-text-muted)" />
        <p class="font-semibold" style="color: var(--color-text-muted)">{{ $t('problems.noProblemsFull') }}</p>
        <p style="color: var(--color-text-tertiary)">{{ $t('problems.noProblemHint') }}</p>
      </div>

      <!-- 검색 결과 없음 -->
      <div v-else-if="filteredProblems.length === 0"
           class="flex flex-col items-center justify-center py-20">
        <p style="color: var(--color-text-tertiary)">{{ $t('problems.noProblemsFiltered') }}</p>
      </div>

      <!-- 목록 -->
      <div v-else class="flex flex-col gap-2">
        <div
          v-for="p in filteredProblems"
          :key="p.id"
          class="flex items-center gap-4 px-5 py-4 rounded-xl border cursor-default transition-colors problem-row"
          style="background: var(--color-bg-primary); border-color: var(--color-border)"
        >
          <!-- 유형 뱃지 -->
          <span
            class="inline-flex items-center h-7 px-3 rounded-full font-semibold flex-shrink-0"
            :style="typeBadgeStyle(p.type)"
          >
            {{ typeLabel(p.type) }}
          </span>

          <!-- 제목 -->
          <span class="flex-1 font-semibold truncate" style="color: var(--color-text-primary)">
            {{ p.title }}
          </span>

          <!-- 임시저장 뱃지 -->
          <span v-if="p.is_draft"
                class="h-6 px-3 rounded-full font-medium flex-shrink-0"
                style="background: var(--color-warning-bg); color: var(--color-warning); display: inline-flex; align-items: center">
            {{ $t('problems.draft') }}
          </span>

          <!-- 과목 -->
          <span v-if="p.subject_name"
                class="flex-shrink-0" style="color: var(--color-text-tertiary)">
            {{ p.subject_name }}
          </span>

          <!-- 날짜 -->
          <span class="flex-shrink-0 font-mono" style="color: var(--color-text-tertiary)">
            {{ p.created_at.slice(0, 10) }}
          </span>

          <!-- 액션 -->
          <div class="flex gap-2 flex-shrink-0">
            <button
              class="h-8 px-3 rounded-lg border font-medium transition-colors"
              style="background: var(--color-bg-primary); color: var(--color-text-muted); border-color: var(--color-border)"
              @click="openEdit(p.id)"
            >
              {{ $t('problems.edit') }}
            </button>
            <button
              class="h-8 px-3 rounded-lg border font-medium transition-colors"
              style="background: var(--color-bg-primary); color: var(--color-danger); border-color: var(--color-danger-border)"
              @click="confirmDelete(p)"
            >
              {{ $t('problems.delete') }}
            </button>
          </div>
        </div>
      </div>
    </main>

    <!-- ══ 생성/편집 모달 ══ -->
    <div v-if="showFormModal"
         class="fixed inset-0 z-50 flex items-center justify-center p-4"
         style="background: rgba(0,0,0,0.5)">
      <div class="w-full flex flex-col rounded-2xl overflow-hidden"
           style="max-width: 900px; max-height: 90vh; background: var(--color-bg-primary)">

        <!-- 모달 헤더 -->
        <div class="flex items-center justify-between px-6 py-4 flex-shrink-0"
             style="background: var(--color-bg-primary); border-bottom: 1px solid var(--color-border)">
          <div class="flex items-center gap-2">
            <button
              class="flex items-center gap-2 h-8 px-3 rounded-lg border font-medium"
              style="background: transparent; color: var(--color-text-muted); border-color: var(--color-border)"
              @click="closeForm"
            >
              <IconArrowLeft :size="14" />
              <span>{{ $t('problems.backToProblemBank') }}</span>
            </button>
            <span style="color: var(--color-border)">/</span>
            <span class="font-semibold" style="color: var(--color-text-primary)">
              {{ editingId ? $t('problems.editProblemTitle') : $t('problems.newProblemTitle') }}
            </span>
          </div>
          <div class="flex items-center gap-2">
            <button
              :disabled="isSaving"
              class="h-9 px-4 rounded-lg border font-medium"
              style="background: var(--color-bg-primary); color: var(--color-text-muted); border-color: var(--color-border)"
              @click="submitForm(true)"
            >
              <IconLoader2 v-if="isSaving && savingAsDraft" :size="15" class="spin inline-block mr-1" />
              {{ isSaving && savingAsDraft ? $t('problems.saving') : $t('problems.saveDraft') }}
            </button>
            <button
              :disabled="isSaving"
              class="h-9 px-5 rounded-lg font-semibold"
              style="background: var(--color-accent); color: #fff; border: none"
              @click="submitForm(false)"
            >
              <IconLoader2 v-if="isSaving && !savingAsDraft" :size="15" class="spin inline-block mr-1" />
              {{ isSaving && !savingAsDraft ? $t('problems.saving') : $t('problems.save') }}
            </button>
          </div>
        </div>

        <!-- 에러 배너 -->
        <div v-if="formError"
             class="flex items-center gap-3 px-6 py-3 flex-shrink-0"
             style="background: var(--color-danger-bg); color: var(--color-danger); border-bottom: 1px solid var(--color-danger-border)">
          <IconAlertCircle :size="18" class="shrink-0" />
          <span>{{ $t(`errors.${formError}`, $t('errors.ERR_UNKNOWN')) }}</span>
        </div>

        <!-- 모달 본문 (MCQ는 2-column, 나머지는 1-column) -->
        <div class="flex flex-1 overflow-hidden">

          <!-- 폼 영역 -->
          <div class="flex-1 overflow-y-auto p-6 flex flex-col gap-5">

            <!-- 유형 선택 (편집 시 비활성) -->
            <div>
              <label class="block font-semibold mb-3" style="color: var(--color-text-primary)">
                {{ $t('problems.problemType') }}
              </label>
              <div class="flex gap-2 flex-wrap">
                <button
                  v-for="pt in problemTypes"
                  :key="pt.value"
                  :disabled="!!editingId"
                  class="h-9 px-4 rounded-lg font-medium transition-colors border"
                  :style="formType === pt.value
                    ? 'border-width: 2px; border-color: var(--color-accent); background: rgba(37,99,235,0.08); color: var(--color-accent); font-weight: 600'
                    : 'border-color: var(--color-border); background: var(--color-bg-primary); color: var(--color-text-muted)'"
                  @click="formType = pt.value"
                >
                  {{ pt.label }}
                </button>
              </div>
            </div>

            <!-- 제목 -->
            <div>
              <label class="block font-semibold mb-2" style="color: var(--color-text-primary)">
                {{ $t('problems.problemTitle') }}
              </label>
              <input
                v-model="formTitle"
                :disabled="isSaving"
                type="text"
                class="w-full h-11 rounded-xl px-4 border"
                style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border)"
                :placeholder="$t('problems.titlePlaceholder')"
              />
            </div>

            <!-- 설명 -->
            <div>
              <label class="block font-semibold mb-2" style="color: var(--color-text-primary)">
                {{ $t('problems.description') }}
              </label>
              <textarea
                v-model="formDescription"
                :disabled="isSaving"
                class="w-full rounded-xl px-4 py-3 border resize-y"
                style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border); min-height: 80px; line-height: 1.6"
                :placeholder="$t('problems.descriptionPlaceholder')"
              ></textarea>
            </div>

            <!-- 과목 + 교사 메모 -->
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block font-semibold mb-2" style="color: var(--color-text-primary)">
                  {{ $t('problems.subject') }}
                </label>
                <select
                  v-model="formSubjectId"
                  :disabled="isSaving"
                  class="w-full h-11 rounded-xl px-4 border"
                  style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border)"
                >
                  <option :value="null">{{ $t('problems.subjectNone') }}</option>
                  <option v-for="s in subjects" :key="s.id" :value="s.id">{{ s.name }}</option>
                </select>
              </div>
              <div>
                <label class="block font-semibold mb-2" style="color: var(--color-text-primary)">
                  {{ $t('problems.comment') }}
                </label>
                <input
                  v-model="formComment"
                  :disabled="isSaving"
                  type="text"
                  class="w-full h-11 rounded-xl px-4 border"
                  style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border)"
                  :placeholder="$t('problems.commentPlaceholder')"
                />
              </div>
            </div>

            <!-- ── 단답형 전용 ── -->
            <template v-if="formType === 'short_answer'">
              <div>
                <label class="block font-semibold mb-2" style="color: var(--color-text-primary)">
                  {{ $t('problems.answerLabel') }}
                </label>
                <input
                  v-model="formAnswer"
                  :disabled="isSaving"
                  type="text"
                  class="w-full h-11 rounded-xl px-4 border"
                  style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border)"
                  :placeholder="$t('problems.answerPlaceholder')"
                />
              </div>
              <label class="flex items-center gap-3 cursor-pointer">
                <input v-model="formCaseSensitive" type="checkbox"
                       class="w-5 h-5 rounded" style="accent-color: var(--color-accent)" />
                <span style="color: var(--color-text-primary)">{{ $t('problems.caseSensitive') }}</span>
              </label>
            </template>

            <!-- ── 객관식 전용 ── -->
            <template v-if="formType === 'multiple_choice'">
              <label class="flex items-center gap-3 cursor-pointer">
                <input v-model="formAllowMultiple" type="checkbox"
                       class="w-5 h-5 rounded" style="accent-color: var(--color-accent)" />
                <span style="color: var(--color-text-primary)">{{ $t('problems.allowMultiple') }}</span>
              </label>

              <div>
                <label class="block font-semibold mb-3" style="color: var(--color-text-primary)">
                  {{ $t('problems.choicesLabel') }}
                </label>
                <div class="flex flex-col gap-2 mb-3">
                  <div
                    v-for="(choice, idx) in formChoices"
                    :key="idx"
                    class="flex items-center gap-3"
                  >
                    <label
                      class="flex flex-1 items-center gap-3 px-4 py-3 rounded-xl border cursor-pointer transition-colors"
                      :style="choice.is_correct
                        ? 'border-color: var(--color-success); background: var(--color-success-bg)'
                        : 'border-color: var(--color-border); background: var(--color-bg-primary)'"
                    >
                      <input
                        type="radio"
                        name="correctChoice"
                        class="w-5 h-5 flex-shrink-0"
                        style="accent-color: var(--color-success)"
                        :checked="choice.is_correct"
                        @change="setCorrectChoice(idx)"
                      />
                      <input
                        v-model="choice.content"
                        :disabled="isSaving"
                        type="text"
                        class="flex-1 border-0 bg-transparent outline-none"
                        style="color: var(--color-text-primary)"
                        :placeholder="$t('problems.choicePlaceholder')"
                        @input="syncPreview"
                      />
                      <span
                        v-if="choice.is_correct"
                        class="font-semibold flex-shrink-0"
                        style="color: var(--color-success)"
                      >
                        {{ $t('problems.correctAnswer') }}
                      </span>
                    </label>
                    <button
                      v-if="formChoices.length > 2"
                      class="w-8 h-8 rounded-lg border flex items-center justify-center flex-shrink-0 transition-colors"
                      style="background: var(--color-bg-primary); color: var(--color-danger); border-color: var(--color-danger-border)"
                      @click="removeChoice(idx)"
                    >
                      <IconX :size="14" />
                    </button>
                  </div>
                </div>
                <button
                  class="flex items-center gap-2 h-10 px-4 rounded-xl border font-medium transition-colors add-choice-btn"
                  style="background: transparent; color: var(--color-text-tertiary); border: 1.5px dashed var(--color-border)"
                  @click="addChoice"
                >
                  <IconPlus :size="15" />
                  <span>{{ $t('problems.addChoice') }}</span>
                </button>
              </div>
            </template>

            <!-- ── 코딩 전용 ── -->
            <template v-if="formType === 'code_submit'">
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block font-semibold mb-2" style="color: var(--color-text-primary)">
                    {{ $t('problems.inputFormat') }}
                  </label>
                  <textarea
                    v-model="formInputFormat"
                    :disabled="isSaving"
                    class="w-full rounded-xl px-4 py-3 border resize-y"
                    style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border); min-height: 72px"
                  ></textarea>
                </div>
                <div>
                  <label class="block font-semibold mb-2" style="color: var(--color-text-primary)">
                    {{ $t('problems.outputFormat') }}
                  </label>
                  <textarea
                    v-model="formOutputFormat"
                    :disabled="isSaving"
                    class="w-full rounded-xl px-4 py-3 border resize-y"
                    style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border); min-height: 72px"
                  ></textarea>
                </div>
              </div>

              <div>
                <label class="block font-semibold mb-2" style="color: var(--color-text-primary)">
                  {{ $t('problems.constraints') }}
                </label>
                <textarea
                  v-model="formConstraints"
                  :disabled="isSaving"
                  class="w-full rounded-xl px-4 py-3 border resize-y"
                  style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border); min-height: 56px"
                ></textarea>
              </div>

              <div class="grid grid-cols-3 gap-4">
                <div>
                  <label class="block font-semibold mb-2" style="color: var(--color-text-primary)">
                    {{ $t('problems.timeLimitMs') }}
                  </label>
                  <input v-model.number="formTimeLimitMs" :disabled="isSaving" type="number" min="100" max="30000"
                         class="w-full h-11 rounded-xl px-4 border"
                         style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border)" />
                </div>
                <div>
                  <label class="block font-semibold mb-2" style="color: var(--color-text-primary)">
                    {{ $t('problems.memoryLimitMb') }}
                  </label>
                  <input v-model.number="formMemoryLimitMb" :disabled="isSaving" type="number" min="16" max="1024"
                         class="w-full h-11 rounded-xl px-4 border"
                         style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border)" />
                </div>
                <div class="flex items-end pb-2">
                  <label class="flex items-center gap-3 cursor-pointer">
                    <input v-model="formShowIoOnFail" type="checkbox"
                           class="w-5 h-5 rounded" style="accent-color: var(--color-accent)" />
                    <span style="color: var(--color-text-primary)">{{ $t('problems.showIoOnFail') }}</span>
                  </label>
                </div>
              </div>

              <!-- 테스트케이스 -->
              <div>
                <div class="flex items-center justify-between mb-3">
                  <label class="font-semibold" style="color: var(--color-text-primary)">
                    {{ $t('problems.testCasesLabel') }}
                  </label>
                  <span style="color: var(--color-text-tertiary)">{{ $t('problems.testCaseHint') }}</span>
                </div>

                <!-- 헤더 -->
                <div class="grid gap-2 mb-2 px-2" style="grid-template-columns: 1fr 1fr auto auto">
                  <span class="font-semibold" style="color: var(--color-text-muted)">{{ $t('problems.inputStdin') }}</span>
                  <span class="font-semibold" style="color: var(--color-text-muted)">{{ $t('problems.expectedOutput') }}</span>
                  <span class="font-semibold" style="color: var(--color-text-muted)">{{ $t('problems.isSample') }}</span>
                  <span></span>
                </div>

                <div class="flex flex-col gap-2 mb-3">
                  <div
                    v-for="(tc, idx) in formTestCases"
                    :key="idx"
                    class="grid gap-2 items-center"
                    style="grid-template-columns: 1fr 1fr auto auto"
                  >
                    <input
                      v-model="tc.input"
                      :disabled="isSaving"
                      type="text"
                      class="h-11 rounded-xl px-3 border font-mono"
                      style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border)"
                    />
                    <input
                      v-model="tc.expected_output"
                      :disabled="isSaving"
                      type="text"
                      class="h-11 rounded-xl px-3 border font-mono"
                      style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border)"
                    />
                    <input
                      v-model="tc.is_sample"
                      type="checkbox"
                      class="w-5 h-5 mx-auto"
                      style="accent-color: var(--color-accent)"
                    />
                    <button
                      class="w-9 h-9 rounded-lg border flex items-center justify-center transition-colors"
                      style="background: var(--color-bg-primary); color: var(--color-danger); border-color: var(--color-danger-border)"
                      @click="removeTestCase(idx)"
                    >
                      <IconTrash :size="14" />
                    </button>
                  </div>
                </div>

                <button
                  class="flex items-center gap-2 h-10 px-4 rounded-xl border font-medium add-choice-btn"
                  style="background: transparent; color: var(--color-text-tertiary); border: 1.5px dashed var(--color-border)"
                  @click="addTestCase"
                >
                  <IconPlus :size="15" />
                  <span>{{ $t('problems.addTestCase') }}</span>
                </button>
              </div>
            </template>

          </div>

          <!-- MCQ 학생 미리보기 패널 -->
          <div
            v-if="formType === 'multiple_choice'"
            class="flex-shrink-0 flex flex-col overflow-y-auto"
            style="width: 320px; border-left: 1px solid var(--color-border); background: var(--color-bg-secondary)"
          >
            <div class="flex items-center gap-2 px-5 py-4 flex-shrink-0"
                 style="border-bottom: 1px solid var(--color-border)">
              <IconEye :size="15" style="color: var(--color-accent)" />
              <span class="font-semibold" style="color: var(--color-text-primary)">{{ $t('problems.studentPreview') }}</span>
              <span style="color: var(--color-text-tertiary)">{{ $t('problems.previewUpdates') }}</span>
            </div>

            <div class="flex-1 p-5 overflow-y-auto">
              <div class="rounded-xl p-5" style="background: var(--color-bg-primary); border: 1px solid var(--color-border)">
                <div class="flex items-center gap-2 mb-3">
                  <span class="inline-flex items-center h-7 px-3 rounded-lg font-mono font-bold"
                        style="background: var(--color-bg-secondary); color: var(--color-text-muted)">P ??</span>
                  <span class="inline-flex items-center h-6 px-3 rounded-full font-semibold"
                        :style="typeBadgeStyle('multiple_choice')">
                    {{ typeLabel('multiple_choice') }}
                  </span>
                </div>
                <h3 class="font-bold mb-4" style="color: var(--color-text-primary); line-height: 1.5; font-size: 1.05rem">
                  {{ formTitle || '...' }}
                </h3>
                <div class="flex flex-col gap-2">
                  <label
                    v-for="(choice, idx) in formChoices"
                    :key="idx"
                    class="flex items-center gap-3 px-4 py-3 rounded-xl border cursor-pointer"
                    style="border-color: var(--color-border); background: var(--color-bg-primary)"
                  >
                    <input type="radio" name="preview-ans" class="w-4 h-4 flex-shrink-0"
                           style="accent-color: var(--color-accent)" />
                    <span style="color: var(--color-text-muted)">
                      {{ String.fromCharCode(9312 + idx) }}  {{ choice.content || '...' }}
                    </span>
                  </label>
                </div>
              </div>
            </div>
          </div>

        </div>
      </div>
    </div>

    <!-- ══ 삭제 확인 모달 ══ -->
    <div v-if="deleteTarget"
         class="fixed inset-0 z-50 flex items-center justify-center p-4"
         style="background: rgba(0,0,0,0.5)">
      <div class="w-full max-w-md rounded-2xl p-7 flex flex-col gap-5"
           style="background: var(--color-bg-primary)">
        <div class="flex items-center gap-3">
          <IconAlertTriangle :size="22" style="color: var(--color-danger); flex-shrink: 0" />
          <h2 class="text-xl font-bold" style="color: var(--color-text-primary)">
            {{ $t('problems.deleteConfirmTitle') }}
          </h2>
        </div>
        <p style="color: var(--color-text-muted)">
          {{ $t('problems.deleteConfirm', { title: deleteTarget.title }) }}
        </p>
        <p style="color: var(--color-text-tertiary)">{{ $t('problems.deleteConfirmHint') }}</p>
        <div v-if="deleteError"
             class="flex items-center gap-2 p-3 rounded-lg"
             style="background: var(--color-danger-bg); color: var(--color-danger)">
          <IconAlertCircle :size="16" class="shrink-0" />
          <span>{{ $t(`errors.${deleteError}`, $t('errors.ERR_UNKNOWN')) }}</span>
        </div>
        <div class="flex justify-end gap-3">
          <button
            :disabled="isDeleting"
            class="h-10 px-5 rounded-xl border font-medium"
            style="background: var(--color-bg-primary); color: var(--color-text-muted); border-color: var(--color-border)"
            @click="deleteTarget = null; deleteError = null"
          >
            {{ $t('problems.cancel') }}
          </button>
          <button
            :disabled="isDeleting"
            class="h-10 px-5 rounded-xl font-semibold"
            style="background: var(--color-danger); color: #fff; border: none"
            @click="doDelete"
          >
            <IconLoader2 v-if="isDeleting" :size="15" class="spin inline-block mr-1" />
            {{ isDeleting ? $t('problems.deleting') : $t('problems.delete') }}
          </button>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  IconArrowLeft, IconPlus, IconSearch, IconLoader2,
  IconAlertCircle, IconAlertTriangle, IconX, IconTrash,
  IconEye, IconBooks,
} from '@tabler/icons-vue'
import { useProblemStore } from '@/stores/problem'
import { useClassStore } from '@/stores/class'
import { useAuthStore } from '@/stores/auth'
import type { ProblemListItem, ChoiceInput, TestCaseInput } from '@/api/client'

const router = useRouter()
const { t } = useI18n()
const store = useProblemStore()
const classStore = useClassStore()
const auth = useAuthStore()

// ── 목록 상태 ────────────────────────────────────────────────────────────────

const searchQuery = ref('')
const activeFilter = ref('all')

const TYPE_SLUGS = ['short_answer', 'multiple_choice', 'code_submit'] as const
type TypeSlug = typeof TYPE_SLUGS[number]

const typeFilters = computed(() => [
  { value: 'all', label: t('problems.all') },
  { value: 'short_answer', label: t('problems.type_short_answer') },
  { value: 'multiple_choice', label: t('problems.type_multiple_choice') },
  { value: 'code_submit', label: t('problems.type_code_submit') },
])

const problemTypes = computed(() => [
  { value: 'short_answer', label: t('problems.type_short_answer') },
  { value: 'multiple_choice', label: t('problems.type_multiple_choice') },
  { value: 'code_submit', label: t('problems.type_code_submit') },
])

const subjects = computed(() => classStore.subjects)

const filteredProblems = computed(() => {
  let list = store.problems
  if (activeFilter.value !== 'all') {
    list = list.filter((p) => p.type === activeFilter.value)
  }
  const q = searchQuery.value.trim().toLowerCase()
  if (q) {
    list = list.filter((p) => p.title.toLowerCase().includes(q))
  }
  return list
})

function typeLabel(slug: string): string {
  const map: Record<string, string> = {
    short_answer: t('problems.type_short_answer'),
    multiple_choice: t('problems.type_multiple_choice'),
    code_submit: t('problems.type_code_submit'),
  }
  return map[slug] ?? slug
}

function typeBadgeStyle(slug: string): string {
  const styles: Record<string, string> = {
    short_answer:
      'background: var(--color-type-short-bg); color: var(--color-type-short)',
    multiple_choice:
      'background: var(--color-type-mcq-bg); color: var(--color-type-mcq)',
    code_submit:
      'background: var(--color-type-coding-bg); color: var(--color-type-coding)',
  }
  return styles[slug] ?? 'background: var(--color-bg-secondary); color: var(--color-text-muted)'
}

// ── 폼 상태 ──────────────────────────────────────────────────────────────────

const showFormModal = ref(false)
const editingId = ref<number | null>(null)
const formError = ref<string | null>(null)
const isSaving = ref(false)
const savingAsDraft = ref(false)

const formType = ref<TypeSlug>('multiple_choice')
const formTitle = ref('')
const formDescription = ref('')
const formComment = ref('')
const formSubjectId = ref<number | null>(null)

// short_answer
const formAnswer = ref('')
const formCaseSensitive = ref(false)

// multiple_choice
const formAllowMultiple = ref(false)
const formChoices = ref<ChoiceInput[]>([
  { content: '', is_correct: false },
  { content: '', is_correct: false },
])

// code_submit
const formInputFormat = ref('')
const formOutputFormat = ref('')
const formConstraints = ref('')
const formTimeLimitMs = ref(1000)
const formMemoryLimitMb = ref(128)
const formShowIoOnFail = ref(true)
const formTestCases = ref<TestCaseInput[]>([])

function resetForm() {
  formType.value = 'multiple_choice'
  formTitle.value = ''
  formDescription.value = ''
  formComment.value = ''
  formSubjectId.value = null
  formAnswer.value = ''
  formCaseSensitive.value = false
  formAllowMultiple.value = false
  formChoices.value = [
    { content: '', is_correct: false },
    { content: '', is_correct: false },
  ]
  formInputFormat.value = ''
  formOutputFormat.value = ''
  formConstraints.value = ''
  formTimeLimitMs.value = 1000
  formMemoryLimitMb.value = 128
  formShowIoOnFail.value = true
  formTestCases.value = []
  formError.value = null
}

// ── 생성 / 편집 열기 ──────────────────────────────────────────────────────────

function openCreate() {
  resetForm()
  editingId.value = null
  showFormModal.value = true
}

async function openEdit(id: number) {
  resetForm()
  editingId.value = id
  showFormModal.value = true

  try {
    const p = await store.getProblem(id)
    formType.value = p.type as TypeSlug
    formTitle.value = p.title
    formDescription.value = p.description
    formComment.value = p.comment
    formSubjectId.value = p.subject_id

    if (p.type === 'short_answer') {
      formAnswer.value = p.answer ?? ''
      formCaseSensitive.value = p.case_sensitive ?? false
    } else if (p.type === 'multiple_choice') {
      formAllowMultiple.value = p.allow_multiple ?? false
      formChoices.value = (p.choices ?? []).map((c) => ({
        content: c.content,
        is_correct: c.is_correct,
      }))
      if (formChoices.value.length < 2) {
        formChoices.value = [
          { content: '', is_correct: false },
          { content: '', is_correct: false },
        ]
      }
    } else if (p.type === 'code_submit') {
      formInputFormat.value = p.input_format ?? ''
      formOutputFormat.value = p.output_format ?? ''
      formConstraints.value = p.constraints ?? ''
      formTimeLimitMs.value = p.time_limit_ms ?? 1000
      formMemoryLimitMb.value = p.memory_limit_mb ?? 128
      formShowIoOnFail.value = p.show_io_on_fail ?? true
      formTestCases.value = (p.test_cases ?? []).map((tc) => ({
        input: tc.input,
        expected_output: tc.expected_output,
        is_sample: tc.is_sample,
        explanation: tc.explanation,
      }))
    }
  } catch (e) {
    formError.value = e instanceof Error ? e.message : 'ERR_UNKNOWN'
  }
}

function closeForm() {
  if (isSaving.value) return
  showFormModal.value = false
  editingId.value = null
}

// ── MCQ 선지 조작 ────────────────────────────────────────────────────────────

function setCorrectChoice(idx: number) {
  formChoices.value.forEach((c, i) => {
    c.is_correct = i === idx
  })
}

function addChoice() {
  formChoices.value.push({ content: '', is_correct: false })
}

function removeChoice(idx: number) {
  if (formChoices.value.length <= 2) return
  const wasCorrect = formChoices.value[idx].is_correct
  formChoices.value.splice(idx, 1)
  if (wasCorrect && formChoices.value.length > 0) {
    formChoices.value[0].is_correct = true
  }
}

function syncPreview() {
  // 반응형으로 자동 처리됨 — 명시적 동기화 불필요
}

// ── 테스트케이스 조작 ────────────────────────────────────────────────────────

function addTestCase() {
  formTestCases.value.push({ input: '', expected_output: '', is_sample: false, explanation: '' })
}

function removeTestCase(idx: number) {
  formTestCases.value.splice(idx, 1)
}

// ── 저장 ─────────────────────────────────────────────────────────────────────

async function submitForm(asDraft: boolean) {
  if (isSaving.value) return           // Layer 1
  isSaving.value = true                // Layer 2
  savingAsDraft.value = asDraft
  formError.value = null
  try {                                // Layer 3
    const body = buildBody(asDraft)
    if (editingId.value !== null) {
      await store.updateProblem(editingId.value, body)
    } else {
      await store.createProblem(body)
    }
    showFormModal.value = false
    editingId.value = null
  } catch (e) {
    formError.value = e instanceof Error ? e.message : 'ERR_UNKNOWN'
  } finally {
    isSaving.value = false             // Layer 4
  }
}

function buildBody(isDraft: boolean) {
  const base = {
    type: formType.value,
    title: formTitle.value,
    description: formDescription.value,
    comment: formComment.value,
    is_draft: isDraft,
    subject_id: formSubjectId.value,
  }

  if (formType.value === 'short_answer') {
    return { ...base, answer: formAnswer.value, case_sensitive: formCaseSensitive.value }
  }
  if (formType.value === 'multiple_choice') {
    return { ...base, allow_multiple: formAllowMultiple.value, choices: formChoices.value }
  }
  // code_submit
  return {
    ...base,
    input_format: formInputFormat.value,
    output_format: formOutputFormat.value,
    constraints: formConstraints.value,
    time_limit_ms: formTimeLimitMs.value,
    memory_limit_mb: formMemoryLimitMb.value,
    show_io_on_fail: formShowIoOnFail.value,
    test_cases: formTestCases.value,
  }
}

// ── 삭제 ─────────────────────────────────────────────────────────────────────

const deleteTarget = ref<ProblemListItem | null>(null)
const deleteError = ref<string | null>(null)
const isDeleting = ref(false)

function confirmDelete(p: ProblemListItem) {
  deleteTarget.value = p
  deleteError.value = null
}

async function doDelete() {
  if (isDeleting.value) return          // Layer 1
  if (!deleteTarget.value) return
  isDeleting.value = true               // Layer 2
  deleteError.value = null
  try {                                 // Layer 3
    await store.deleteProblem(deleteTarget.value.id)
    deleteTarget.value = null
  } catch (e) {
    deleteError.value = e instanceof Error ? e.message : 'ERR_UNKNOWN'
  } finally {
    isDeleting.value = false            // Layer 4
  }
}

// ── 네비게이션 ───────────────────────────────────────────────────────────────

function goBack() {
  router.push(auth.teacher?.role === 'admin' ? '/admin' : '/teacher')
}

// ── ESC 키 ───────────────────────────────────────────────────────────────────

function handleKeydown(e: KeyboardEvent) {
  if (e.key !== 'Escape') return
  if (deleteTarget.value) {
    deleteTarget.value = null
    deleteError.value = null
  } else if (showFormModal.value && !isSaving.value) {
    closeForm()
  }
}

// ── 초기화 ───────────────────────────────────────────────────────────────────

onMounted(async () => {
  window.addEventListener('keydown', handleKeydown)
  await Promise.all([store.fetchProblems(), classStore.fetchSubjects()])
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.problem-row:hover {
  border-color: var(--color-border-strong) !important;
  background: var(--color-bg-tertiary) !important;
}
.add-choice-btn:hover {
  border-color: var(--color-accent) !important;
  color: var(--color-accent) !important;
}
</style>
