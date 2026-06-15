<template>
  <Teleport to="body">
    <div
      v-if="show"
      class="fixed inset-0 z-50 flex items-center justify-center px-4"
      style="background: var(--color-modal-overlay)"
    >
      <div
        class="w-full max-w-2xl rounded-xl p-6"
        style="background: var(--color-bg-secondary); border: 1px solid var(--color-border); box-shadow: var(--shadow-dropdown)"
      >
        <!-- Header -->
        <div class="flex items-center justify-between mb-5">
          <h2 class="font-semibold" style="color: var(--color-text-primary)">{{ title }}</h2>
          <button
            type="button"
            class="h-9 px-4 rounded-lg flex items-center gap-2 font-medium border transition-colors"
            :style="downloadDone
              ? { background: 'var(--color-success-bg)', color: 'var(--color-success)', borderColor: 'var(--color-success)' }
              : { background: 'transparent', color: 'var(--color-text-muted)', borderColor: 'var(--color-border)' }"
            @click="onDownloadTemplate"
          >
            <IconCheck v-if="downloadDone" :size="16" />
            <IconDownload v-else :size="16" />
            {{ downloadDone ? $t('common.downloaded') : $t('common.downloadTemplate') }}
          </button>
        </div>

        <!-- Drop zone -->
        <div
          class="rounded-xl border-2 border-dashed flex flex-col items-center justify-center py-10 px-6 cursor-pointer transition-colors"
          :style="{
            borderColor: isDragging ? 'var(--color-accent)' : 'var(--color-border)',
            background: isDragging ? 'var(--color-info-bg)' : 'var(--color-bg-primary)',
          }"
          @click="fileInputRef?.click()"
          @dragover.prevent="isDragging = true"
          @dragleave.prevent="isDragging = false"
          @drop.prevent="onDrop"
        >
          <IconUpload :size="32" class="mb-3" style="color: var(--color-text-tertiary)" />
          <p class="font-medium" style="color: var(--color-text-primary)">
            {{ selectedFileName || $t('common.dropOrClick') }}
          </p>
          <p class="mt-1" style="color: var(--color-text-muted)">{{ $t('common.fileFormats') }}</p>
        </div>
        <input
          ref="fileInputRef"
          type="file"
          accept=".xlsx,.xls,.csv"
          class="hidden"
          @change="onFileChange"
        />

        <!-- Parse error -->
        <div
          v-if="parseError"
          class="mt-4 flex items-center gap-2 rounded-lg border px-4 py-3"
          style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
          role="alert"
        >
          <IconAlertCircle :size="18" class="shrink-0" />
          <span>{{ $t(`errors.${parseError}`, $t('errors.ERR_UNKNOWN')) }}</span>
        </div>

        <!-- Preview -->
        <div v-if="parsedRows.length > 0" class="mt-4">
          <p class="font-medium mb-2" style="color: var(--color-text-muted)">
            {{ $t('common.rowsDetected', { count: parsedRows.length }) }}
          </p>
          <div
            class="rounded-xl border overflow-auto"
            style="border-color: var(--color-border); max-height: 220px"
          >
            <table class="w-full">
              <thead>
                <tr class="sticky top-0" style="background: var(--color-bg-tertiary)">
                  <th
                    v-for="col in columns"
                    :key="col.key"
                    class="px-4 py-2 text-left font-semibold whitespace-nowrap"
                    style="color: var(--color-text-muted)"
                  >
                    {{ $t(col.labelKey) }}
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="(row, i) in parsedRows.slice(0, 100)"
                  :key="i"
                  class="border-t"
                  style="border-color: var(--color-border)"
                >
                  <td
                    v-for="col in columns"
                    :key="col.key"
                    class="px-4 py-2 whitespace-nowrap"
                    style="color: var(--color-text-primary)"
                  >
                    {{ col.display ? col.display(row) : (row[col.key] || '—') }}
                  </td>
                </tr>
                <tr v-if="parsedRows.length > 100">
                  <td
                    :colspan="columns.length"
                    class="px-4 py-2 text-center"
                    style="color: var(--color-text-muted)"
                  >
                    ...{{ parsedRows.length - 100 }}{{ $t('common.moreRows') }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- Import error -->
        <div
          v-if="importError"
          class="mt-4 flex items-center gap-2 rounded-lg border px-4 py-3"
          style="background: var(--color-danger-bg); border-color: var(--color-danger-border); color: var(--color-danger)"
          role="alert"
        >
          <IconAlertCircle :size="18" class="shrink-0" />
          <span>{{ $t(`errors.${importError}`, $t('errors.ERR_UNKNOWN')) }}</span>
        </div>

        <!-- Actions -->
        <div class="flex justify-end gap-3 mt-5">
          <button
            type="button"
            class="h-10 px-5 rounded-lg font-medium"
            style="border: 1px solid var(--color-border); color: var(--color-text-primary); background: transparent"
            @click="emit('update:show', false)"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            :disabled="isImporting || parsedRows.length === 0"
            class="h-10 px-5 rounded-lg font-medium flex items-center gap-2"
            style="background: var(--color-accent); color: var(--color-accent-text); border: none"
            :class="(isImporting || parsedRows.length === 0) ? 'opacity-60 cursor-not-allowed' : ''"
            @click="onImportClick"
          >
            <IconLoader2 v-if="isImporting" :size="17" class="spin" />
            {{ isImporting ? $t('common.importing') : $t('common.importAction') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { IconDownload, IconUpload, IconAlertCircle, IconLoader2, IconCheck } from '@tabler/icons-vue'
import { parseExcelFile, type SynonymMap } from '@/utils/excelImport'
import { downloadExcelTemplate } from '@/utils/templateDownload'

interface Column {
  key: string
  labelKey: string
  display?: (row: Record<string, string>) => string
}

const props = defineProps<{
  show: boolean
  title: string
  templateFilename: string
  templateHeaders: string[]
  templateSample: string[][]
  synonymMap: SynonymMap
  requiredFields: string[]
  columns: Column[]
  onImport: (rows: Record<string, string>[]) => Promise<void>
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
}>()

const fileInputRef = ref<HTMLInputElement | null>(null)
const selectedFileName = ref('')
const downloadDone = ref(false)
const parsedRows = ref<Record<string, string>[]>([])
const parseError = ref<string | null>(null)
const importError = ref<string | null>(null)
const isImporting = ref(false)
const isDragging = ref(false)

watch(
  () => props.show,
  (val) => {
    if (val) {
      selectedFileName.value = ''
      parsedRows.value = []
      parseError.value = null
      importError.value = null
      isImporting.value = false
      isDragging.value = false
      downloadDone.value = false
      if (fileInputRef.value) fileInputRef.value.value = ''
    }
  },
)

async function processFile(file: File) {
  selectedFileName.value = file.name
  parsedRows.value = []
  parseError.value = null
  importError.value = null
  try {
    const result = await parseExcelFile(file, props.synonymMap, props.requiredFields)
    parsedRows.value = result.rows
  } catch (e) {
    parseError.value = e instanceof Error ? e.message : 'ERR_UNKNOWN'
  }
}

function onFileChange(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (file) processFile(file)
}

function onDrop(e: DragEvent) {
  isDragging.value = false
  const file = e.dataTransfer?.files?.[0]
  if (file) processFile(file)
}

function onDownloadTemplate() {
  downloadExcelTemplate(props.templateFilename, props.templateHeaders, props.templateSample)
  downloadDone.value = true
  setTimeout(() => { downloadDone.value = false }, 2000)
}

async function onImportClick() {
  if (isImporting.value) return
  isImporting.value = true
  importError.value = null
  try {
    await props.onImport(parsedRows.value)
    emit('update:show', false)
  } catch (e) {
    importError.value = e instanceof Error ? e.message : 'ERR_UNKNOWN'
  } finally {
    isImporting.value = false
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.show) emit('update:show', false)
}

onMounted(() => document.addEventListener('keydown', onKeydown))
onUnmounted(() => document.removeEventListener('keydown', onKeydown))
</script>
