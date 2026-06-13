<template>
  <div class="editor-form">
    <!-- 유형 선택 (신규 생성 시만) -->
    <div v-if="!initial" class="field">
      <label>문항 유형</label>
      <div class="type-grid">
        <button
          v-for="(label, type) in PROBLEM_TYPE_LABELS"
          :key="type"
          type="button"
          class="type-btn"
          :class="{ active: form.problem_type === Number(type), [`type-${type}`]: true }"
          @click="form.problem_type = Number(type)"
        >
          {{ label }}
        </button>
      </div>
    </div>
    <div v-else class="field">
      <label>문항 유형</label>
      <span class="type-label">{{ PROBLEM_TYPE_LABELS[initial.problem_type as keyof typeof PROBLEM_TYPE_LABELS] }}</span>
    </div>

    <div class="field">
      <label>제목 <span class="required">*</span></label>
      <input v-model="form.title" type="text" placeholder="문제 제목 입력" />
    </div>

    <div class="field">
      <label>설명 (선택)</label>
      <textarea v-model="form.description" placeholder="학생에게 보여줄 문제 설명"></textarea>
    </div>

    <!-- 구조검사 플래그 (유형 2, 4만) -->
    <div v-if="form.problem_type === 2 || form.problem_type === 4" class="field checkbox-field">
      <input type="checkbox" id="isc" v-model="form.is_structure_check" />
      <label for="isc">⑤구조검사 게이트 활성화 (채점 전 사전 검사)</label>
    </div>

    <!-- type_config 편집 -->
    <div class="field">
      <label>유형별 설정 (type_config JSON)</label>
      <div class="config-helper">
        <button type="button" class="helper-btn" @click="fillTemplate">템플릿 채우기</button>
        <span class="config-hint">유효한 JSON이어야 합니다</span>
      </div>
      <textarea v-model="form.type_config" class="json-textarea" spellcheck="false"></textarea>
      <div v-if="jsonError" class="json-error">{{ jsonError }}</div>
    </div>

    <div v-if="saveError" class="save-error">{{ saveError }}</div>

    <div class="editor-actions">
      <button type="button" @click="$emit('cancel')">취소</button>
      <button type="button" class="btn-primary" @click="submit" :disabled="saving">
        {{ saving ? '저장 중...' : '저장' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, computed, watch, ref } from 'vue'
import { PROBLEM_TYPE_LABELS, type ProblemRow, type CreateProblemInput, type UpdateProblemInput } from '@/api/client'

const props = defineProps<{
  initial: ProblemRow | null
  saving: boolean
  saveError: string | null
}>()

const emit = defineEmits<{
  (e: 'save', data: CreateProblemInput | UpdateProblemInput): void
  (e: 'cancel'): void
}>()

const form = reactive({
  problem_type: props.initial?.problem_type ?? 1,
  title: props.initial?.title ?? '',
  description: props.initial?.description ?? '',
  type_config: props.initial?.type_config ?? '{}',
  is_structure_check: props.initial?.is_structure_check ?? false,
})

const jsonError = ref<string | null>(null)

watch(() => form.type_config, (v) => {
  try { JSON.parse(v); jsonError.value = null }
  catch (e) { jsonError.value = `JSON 오류: ${e instanceof Error ? e.message : '잘못된 형식'}` }
})

// 유형 변경 시 템플릿 자동 채우기 (신규 생성 시만)
watch(() => form.problem_type, () => {
  if (!props.initial) fillTemplate()
})

const TEMPLATES: Record<number, object> = {
  1: { code: "", language: "python", expected_output: "" },
  2: { languages: ["python"], time_limit_sec: 2, memory_limit_mb: 256, test_cases: [] },
  3: { template_path: "", has_code: false },
  4: { base_code: "", language: "python", markers: [] },
}

function fillTemplate() {
  form.type_config = JSON.stringify(TEMPLATES[form.problem_type] ?? {}, null, 2)
}

function submit() {
  if (!form.title.trim()) return
  try { JSON.parse(form.type_config) } catch { return }

  if (props.initial) {
    const data: UpdateProblemInput = {
      title: form.title.trim(),
      description: form.description,
      type_config: form.type_config,
      is_structure_check: form.is_structure_check,
    }
    emit('save', data)
  } else {
    const data: CreateProblemInput = {
      problem_type: form.problem_type,
      title: form.title.trim(),
      description: form.description,
      type_config: form.type_config,
      is_structure_check: form.is_structure_check,
    }
    emit('save', data)
  }
}

// 초기 진입 시 템플릿 채우기
if (!props.initial) fillTemplate()
</script>

<style scoped>
.editor-form {
  background: var(--color-background-primary);
  border: 1px solid var(--color-border-secondary);
  border-radius: var(--border-radius-lg);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.field { display: flex; flex-direction: column; gap: 6px; }
.field label { font-size: 12px; font-weight: 600; color: var(--color-text-secondary); }
.required { color: var(--color-text-danger); }

.type-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 6px;
}

.type-btn {
  padding: 7px 10px;
  font-size: 12px;
  font-weight: 500;
  border-radius: var(--border-radius-sm);
  cursor: pointer;
  border: 1px solid var(--color-border-primary);
  background: var(--color-background-secondary);
  color: var(--color-text-secondary);
  transition: all 0.1s;
  text-align: left;
}

.type-btn.active.type-1 { background: #7c3aed; color: #fff; border-color: transparent; }
.type-btn.active.type-2 { background: #0369a1; color: #fff; border-color: transparent; }
.type-btn.active.type-3 { background: #065f46; color: #fff; border-color: transparent; }
.type-btn.active.type-4 { background: #92400e; color: #fff; border-color: transparent; }

.type-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.field textarea {
  width: 100%;
  min-height: 80px;
  padding: 7px 10px;
  border: 1px solid var(--color-border-primary);
  border-radius: var(--border-radius-sm);
  font-size: 13px;
  font-family: inherit;
  resize: vertical;
}

.checkbox-field {
  flex-direction: row;
  align-items: center;
  gap: 8px;
}

.checkbox-field label {
  font-size: 13px;
  font-weight: normal;
  color: var(--color-text-primary);
  cursor: pointer;
}

.config-helper {
  display: flex;
  align-items: center;
  gap: 8px;
}

.helper-btn {
  font-size: 11px;
  padding: 3px 8px;
  border: 1px solid var(--color-border-primary);
  border-radius: var(--border-radius-sm);
  background: var(--color-background-secondary);
  cursor: pointer;
  color: var(--color-text-secondary);
}

.helper-btn:hover { background: var(--color-background-tertiary); }
.config-hint { font-size: 11px; color: var(--color-text-tertiary); }

.json-textarea {
  width: 100%;
  min-height: 140px;
  padding: 8px 10px;
  border: 1px solid var(--color-border-primary);
  border-radius: var(--border-radius-sm);
  font-family: 'Courier New', monospace;
  font-size: 12px;
  resize: vertical;
  background: var(--color-background-secondary);
}

.json-error {
  font-size: 11px;
  color: var(--color-text-danger);
  background: var(--color-background-danger);
  padding: 6px 8px;
  border-radius: var(--border-radius-sm);
}

.save-error {
  font-size: 12px;
  color: var(--color-text-danger);
  background: var(--color-background-danger);
  padding: 8px 10px;
  border-radius: var(--border-radius-sm);
}

.editor-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
}

.btn-primary {
  background: var(--color-accent);
  color: #fff;
  border-color: transparent;
  padding: 7px 16px;
  font-size: 13px;
}

.btn-primary:hover:not(:disabled) { background: var(--color-accent-hover); }
</style>
