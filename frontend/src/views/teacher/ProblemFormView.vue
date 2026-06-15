<template>
  <div class="min-h-screen flex flex-col" style="background: var(--color-bg-secondary)">

    <!-- ── 헤더 ── -->
    <header class="h-15 flex items-center justify-between px-7 flex-shrink-0"
            style="background: var(--color-bg-primary); border-bottom: 1px solid var(--color-border)">
      <div class="flex items-center gap-3">
        <button
          class="flex items-center gap-2 h-9 px-3 rounded-lg border"
          style="background: transparent; color: var(--color-text-muted); border-color: var(--color-border)"
          @click="goBack"
        >
          <IconArrowLeft :size="16" />
          <span>{{ $t('problems.backToProblemBank') }}</span>
        </button>
        <div class="w-px h-5" style="background: var(--color-border)"></div>
        <span class="font-semibold" style="color: var(--color-text-primary)">
          {{ editingId ? $t('problems.editProblemTitle') : $t('problems.newProblemTitle') }}
        </span>
      </div>
      <div class="flex items-center gap-2">
        <button
          :disabled="isSaving || isLoading"
          class="h-9 px-4 rounded-lg border font-medium"
          style="background: var(--color-bg-primary); color: var(--color-text-muted); border-color: var(--color-border)"
          @click="submitForm(true)"
        >
          <IconLoader2 v-if="isSaving && savingAsDraft" :size="15" class="spin inline-block mr-1" />
          {{ isSaving && savingAsDraft ? $t('problems.saving') : $t('problems.saveDraft') }}
        </button>
        <button
          :disabled="isSaving || isLoading"
          class="h-9 px-5 rounded-lg font-semibold"
          style="background: var(--color-accent); color: #fff; border: none"
          @click="submitForm(false)"
        >
          <IconLoader2 v-if="isSaving && !savingAsDraft" :size="15" class="spin inline-block mr-1" />
          {{ isSaving && !savingAsDraft ? $t('problems.saving') : $t('problems.save') }}
        </button>
      </div>
    </header>

    <!-- ── 에러 배너 ── -->
    <div v-if="formError"
         class="flex items-center gap-3 px-7 py-3 flex-shrink-0"
         style="background: var(--color-danger-bg); color: var(--color-danger); border-bottom: 1px solid var(--color-danger-border)">
      <IconAlertCircle :size="18" class="shrink-0" />
      <span>{{ $t(`errors.${formError}`, $t('errors.ERR_UNKNOWN')) }}</span>
    </div>

    <!-- ── 로딩 (편집 모드 초기) ── -->
    <div v-if="isLoading" class="flex-1 flex items-center justify-center">
      <IconLoader2 :size="36" class="spin" style="color: var(--color-accent)" />
    </div>

    <!-- ── 로드 에러 ── -->
    <div v-else-if="loadError" class="flex-1 flex items-center justify-center p-8">
      <div class="flex items-center gap-3 p-5 rounded-2xl"
           style="background: var(--color-danger-bg); color: var(--color-danger)">
        <IconAlertCircle :size="22" />
        <span>{{ $t(`errors.${loadError}`, $t('errors.ERR_UNKNOWN')) }}</span>
      </div>
    </div>

    <!-- ── 본문: 좌(미리보기) + 우(폼), 소형화면은 1열 ── -->
    <div v-else class="flex-1 flex flex-col lg:flex-row">

      <!-- ══ 좌: 미리보기 패널 (모바일: 하단, 데스크톱: 좌측) ══ -->
      <aside
        class="order-2 lg:order-1 flex-shrink-0 flex flex-col lg:w-96 xl:w-[420px]"
        style="background: var(--color-bg-primary); border-top: 1px solid var(--color-border)"
      >
        <!-- 패널 헤더 -->
        <div class="flex items-center gap-2 px-5 py-4 flex-shrink-0"
             style="border-bottom: 1px solid var(--color-border)">
          <IconEye :size="16" style="color: var(--color-accent)" />
          <span class="font-semibold" style="color: var(--color-text-primary)">{{ $t('problems.studentPreview') }}</span>
          <span class="ml-1" style="color: var(--color-text-tertiary)">{{ $t('problems.previewUpdates') }}</span>
        </div>

        <!-- 미리보기 콘텐츠 -->
        <div class="p-5">
          <div class="rounded-xl p-5" style="background: var(--color-bg-secondary); border: 1px solid var(--color-border)">

            <!-- 유형 뱃지 -->
            <div class="flex items-center gap-2 mb-3">
              <span class="inline-flex items-center h-6 px-3 rounded-full font-semibold"
                    :style="typeBadgeStyle(formType)">
                {{ typeLabel(formType) }}
              </span>
            </div>

            <!-- 제목 -->
            <h3 class="font-bold mb-3"
                style="color: var(--color-text-primary); line-height: 1.5; font-size: 1.05rem; white-space: pre-wrap; word-break: break-word">
              {{ formTitle || '...' }}
            </h3>

            <!-- 설명 -->
            <p v-if="formDescription"
               class="mb-4"
               style="color: var(--color-text-muted); line-height: 1.6; white-space: pre-wrap; word-break: break-word">
              {{ formDescription }}
            </p>

            <!-- 단답형 -->
            <template v-if="formType === 'short_answer'">
              <input
                type="text"
                disabled
                class="w-full h-11 rounded-xl px-4 border"
                style="background: var(--color-bg-primary); color: var(--color-text-tertiary); border-color: var(--color-border)"
                :placeholder="$t('problems.answerPlaceholder')"
              />
            </template>

            <!-- 객관식 -->
            <template v-else-if="formType === 'multiple_choice'">
              <div class="flex flex-col gap-2">
                <label
                  v-for="(choice, idx) in formChoices"
                  :key="idx"
                  class="flex items-center gap-3 px-4 py-3 rounded-xl border"
                  style="border-color: var(--color-border); background: var(--color-bg-primary)"
                >
                  <input
                    :type="formAllowMultiple ? 'checkbox' : 'radio'"
                    name="preview-ans"
                    disabled
                    class="w-4 h-4 flex-shrink-0"
                    style="accent-color: var(--color-accent)"
                  />
                  <span style="color: var(--color-text-muted)">
                    {{ String.fromCharCode(9312 + idx) }} {{ choice.content || '...' }}
                  </span>
                </label>
              </div>
            </template>

            <!-- 코딩 -->
            <template v-else-if="formType === 'code_submit'">
              <div v-if="formInputFormat || formOutputFormat || formConstraints" class="flex flex-col gap-3 mb-4">
                <div v-if="formInputFormat">
                  <p class="font-semibold mb-1" style="color: var(--color-text-muted)">{{ $t('problems.inputFormat') }}</p>
                  <div class="rounded-lg px-3 py-2" style="background: var(--color-bg-primary); border: 1px solid var(--color-border)">
                    <pre class="font-mono" style="color: var(--color-text-primary); white-space: pre-wrap; word-break: break-word">{{ formInputFormat }}</pre>
                  </div>
                </div>
                <div v-if="formOutputFormat">
                  <p class="font-semibold mb-1" style="color: var(--color-text-muted)">{{ $t('problems.outputFormat') }}</p>
                  <div class="rounded-lg px-3 py-2" style="background: var(--color-bg-primary); border: 1px solid var(--color-border)">
                    <pre class="font-mono" style="color: var(--color-text-primary); white-space: pre-wrap; word-break: break-word">{{ formOutputFormat }}</pre>
                  </div>
                </div>
                <div v-if="formConstraints">
                  <p class="font-semibold mb-1" style="color: var(--color-text-muted)">{{ $t('problems.constraints') }}</p>
                  <p style="color: var(--color-text-muted); white-space: pre-wrap; word-break: break-word">{{ formConstraints }}</p>
                </div>
              </div>

              <!-- 샘플 테스트케이스 -->
              <div v-if="sampleTestCases.length > 0">
                <p class="font-semibold mb-2" style="color: var(--color-text-muted)">{{ $t('problems.isSample') }}</p>
                <div class="flex flex-col gap-3">
                  <div
                    v-for="(tc, idx) in sampleTestCases"
                    :key="idx"
                    class="grid grid-cols-2 gap-2"
                  >
                    <div>
                      <p class="mb-1" style="color: var(--color-text-tertiary)">{{ $t('problems.inputStdin') }}</p>
                      <div class="rounded-lg px-3 py-2" style="background: var(--color-bg-primary); border: 1px solid var(--color-border)">
                        <pre class="font-mono" style="color: var(--color-text-primary); white-space: pre-wrap">{{ tc.input || '-' }}</pre>
                      </div>
                    </div>
                    <div>
                      <p class="mb-1" style="color: var(--color-text-tertiary)">{{ $t('problems.expectedOutput') }}</p>
                      <div class="rounded-lg px-3 py-2" style="background: var(--color-bg-primary); border: 1px solid var(--color-border)">
                        <pre class="font-mono" style="color: var(--color-text-primary); white-space: pre-wrap">{{ tc.expected_output || '-' }}</pre>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              <p v-else style="color: var(--color-text-tertiary)">{{ $t('problems.testCaseHint') }}</p>
            </template>

          </div>
        </div>
      </aside>

      <!-- ══ 우: 폼 영역 (모바일: 상단, 데스크톱: 우측) ══ -->
      <div
        class="order-1 lg:order-2 flex-1 p-6 flex flex-col gap-5"
        style="border-left: 1px solid var(--color-border)"
      >

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
              @click="formType = pt.value as TypeSlug"
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
            style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border); min-height: 100px; line-height: 1.6"
            :placeholder="$t('problems.descriptionPlaceholder')"
          ></textarea>
        </div>

        <!-- 과목 + 교사 메모 -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
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

        <div class="h-px" style="background: var(--color-border)"></div>

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
            <input
              v-model="formCaseSensitive"
              type="checkbox"
              class="w-5 h-5 rounded"
              style="accent-color: var(--color-accent)"
            />
            <span style="color: var(--color-text-primary)">{{ $t('problems.caseSensitive') }}</span>
          </label>
        </template>

        <!-- ── 객관식 전용 ── -->
        <template v-else-if="formType === 'multiple_choice'">
          <label class="flex items-center gap-3 cursor-pointer">
            <input
              v-model="formAllowMultiple"
              type="checkbox"
              class="w-5 h-5 rounded"
              style="accent-color: var(--color-accent)"
            />
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
              class="flex items-center gap-2 h-10 px-4 rounded-xl border font-medium transition-colors add-item-btn"
              style="background: transparent; color: var(--color-text-tertiary); border: 1.5px dashed var(--color-border)"
              @click="addChoice"
            >
              <IconPlus :size="15" />
              <span>{{ $t('problems.addChoice') }}</span>
            </button>
          </div>
        </template>

        <!-- ── 코딩 전용 ── -->
        <template v-else-if="formType === 'code_submit'">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label class="block font-semibold mb-2" style="color: var(--color-text-primary)">
                {{ $t('problems.inputFormat') }}
              </label>
              <textarea
                v-model="formInputFormat"
                :disabled="isSaving"
                class="w-full rounded-xl px-4 py-3 border resize-y"
                style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border); min-height: 88px"
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
                style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border); min-height: 88px"
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
              style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border); min-height: 64px"
            ></textarea>
          </div>

          <div class="grid grid-cols-2 sm:grid-cols-3 gap-4">
            <div>
              <label class="block font-semibold mb-2" style="color: var(--color-text-primary)">
                {{ $t('problems.timeLimitMs') }}
              </label>
              <input
                v-model.number="formTimeLimitMs"
                :disabled="isSaving"
                type="number"
                min="100"
                max="30000"
                class="w-full h-11 rounded-xl px-4 border"
                style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border)"
              />
            </div>
            <div>
              <label class="block font-semibold mb-2" style="color: var(--color-text-primary)">
                {{ $t('problems.memoryLimitMb') }}
              </label>
              <input
                v-model.number="formMemoryLimitMb"
                :disabled="isSaving"
                type="number"
                min="16"
                max="1024"
                class="w-full h-11 rounded-xl px-4 border"
                style="background: var(--color-bg-primary); color: var(--color-text-primary); border-color: var(--color-border)"
              />
            </div>
            <div class="flex items-end pb-2 col-span-2 sm:col-span-1">
              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  v-model="formShowIoOnFail"
                  type="checkbox"
                  class="w-5 h-5 rounded"
                  style="accent-color: var(--color-accent)"
                />
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

            <!-- 헤더 행 -->
            <div class="grid gap-3 mb-2 px-1" style="grid-template-columns: 1fr 1fr 80px 36px">
              <span class="font-semibold" style="color: var(--color-text-muted)">{{ $t('problems.inputStdin') }}</span>
              <span class="font-semibold" style="color: var(--color-text-muted)">{{ $t('problems.expectedOutput') }}</span>
              <span class="font-semibold text-center" style="color: var(--color-text-muted)">{{ $t('problems.isSample') }}</span>
              <span></span>
            </div>

            <div class="flex flex-col gap-2 mb-3">
              <div
                v-for="(tc, idx) in formTestCases"
                :key="idx"
                class="grid gap-3 items-center"
                style="grid-template-columns: 1fr 1fr 80px 36px"
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
                <div class="flex justify-center">
                  <input
                    v-model="tc.is_sample"
                    type="checkbox"
                    class="w-5 h-5"
                    style="accent-color: var(--color-accent)"
                  />
                </div>
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
              class="flex items-center gap-2 h-10 px-4 rounded-xl border font-medium transition-colors add-item-btn"
              style="background: transparent; color: var(--color-text-tertiary); border: 1.5px dashed var(--color-border)"
              @click="addTestCase"
            >
              <IconPlus :size="15" />
              <span>{{ $t('problems.addTestCase') }}</span>
            </button>
          </div>
        </template>

        <!-- 하단 여백 -->
        <div class="h-6"></div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  IconArrowLeft, IconPlus, IconLoader2,
  IconAlertCircle, IconX, IconTrash, IconEye,
} from '@tabler/icons-vue'
import { useProblemStore } from '@/stores/problem'
import { useClassStore } from '@/stores/class'
import type { ChoiceInput, TestCaseInput } from '@/api/client'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const store = useProblemStore()
const classStore = useClassStore()

const editingId = computed(() => {
  const id = route.params.id
  return id ? Number(id) : null
})

// ── 로딩 상태 (편집 모드) ─────────────────────────────────────────────────────
const isLoading = ref(false)
const loadError = ref<string | null>(null)

// ── 폼 상태 ──────────────────────────────────────────────────────────────────

const TYPE_SLUGS = ['short_answer', 'multiple_choice', 'code_submit'] as const
type TypeSlug = typeof TYPE_SLUGS[number]

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

// ── computed ──────────────────────────────────────────────────────────────────

const subjects = computed(() => classStore.subjects)

const problemTypes = computed(() => [
  { value: 'short_answer', label: t('problems.type_short_answer') },
  { value: 'multiple_choice', label: t('problems.type_multiple_choice') },
  { value: 'code_submit', label: t('problems.type_code_submit') },
])

const sampleTestCases = computed(() =>
  formTestCases.value.filter((tc) => tc.is_sample)
)

// ── helpers ───────────────────────────────────────────────────────────────────

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
    short_answer: 'background: var(--color-type-short-bg); color: var(--color-type-short)',
    multiple_choice: 'background: var(--color-type-mcq-bg); color: var(--color-type-mcq)',
    code_submit: 'background: var(--color-type-coding-bg); color: var(--color-type-coding)',
  }
  return styles[slug] ?? 'background: var(--color-bg-secondary); color: var(--color-text-muted)'
}

// ── MCQ 선지 조작 ─────────────────────────────────────────────────────────────

function setCorrectChoice(idx: number) {
  formChoices.value.forEach((c, i) => { c.is_correct = i === idx })
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

// ── 테스트케이스 조작 ─────────────────────────────────────────────────────────

function addTestCase() {
  formTestCases.value.push({ input: '', expected_output: '', is_sample: false, explanation: '' })
}

function removeTestCase(idx: number) {
  formTestCases.value.splice(idx, 1)
}

// ── 저장 ─────────────────────────────────────────────────────────────────────

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

async function submitForm(asDraft: boolean) {
  if (isSaving.value) return
  isSaving.value = true
  savingAsDraft.value = asDraft
  formError.value = null
  try {
    const body = buildBody(asDraft)
    if (editingId.value !== null) {
      await store.updateProblem(editingId.value, body)
    } else {
      await store.createProblem(body)
    }
    router.push({ name: 'problem-bank' })
  } catch (e) {
    formError.value = e instanceof Error ? e.message : 'ERR_UNKNOWN'
  } finally {
    isSaving.value = false
  }
}

// ── 네비게이션 ────────────────────────────────────────────────────────────────

function goBack() {
  router.push({ name: 'problem-bank' })
}

// ── 초기화 ────────────────────────────────────────────────────────────────────

onMounted(async () => {
  await classStore.fetchSubjects()

  if (editingId.value !== null) {
    isLoading.value = true
    loadError.value = null
    try {
      const p = await store.getProblem(editingId.value)
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
      loadError.value = e instanceof Error ? e.message : 'ERR_UNKNOWN'
    } finally {
      isLoading.value = false
    }
  }
})
</script>

<style scoped>
.add-item-btn:hover {
  border-color: var(--color-accent) !important;
  color: var(--color-accent) !important;
}
</style>
