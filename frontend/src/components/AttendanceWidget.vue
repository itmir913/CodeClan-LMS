<template>
  <div class="attendance-widget">
    <div class="widget-header">
      <span class="widget-title">실시간 접속 현황</span>
      <span class="widget-summary">
        <span class="dot dot--online" />{{ onlineCount }} 접속
        <span class="dot dot--offline" />{{ offlineCount }} 미접속
      </span>
    </div>

    <div v-if="loading && list.length === 0" class="widget-empty">로딩 중...</div>
    <div v-else-if="error" class="widget-error">{{ error }}</div>
    <div v-else-if="list.length === 0" class="widget-empty">학생이 없습니다</div>
    <div v-else class="student-grid">
      <div
        v-for="s in list"
        :key="s.student_id"
        class="student-row"
        :title="s.is_late ? '지각 입장' : ''"
      >
        <span class="dot" :class="s.is_online ? 'dot--online' : 'dot--offline'" />
        <span class="student-name" :class="{ 'name--late': s.is_late }">{{ s.name }}</span>
        <span class="student-no">{{ s.student_number }}</span>
      </div>
    </div>

    <div class="widget-note">
      15초마다 갱신 · {{ onlineCount }}/{{ list.length }}명 접속
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { api, type AttendanceRow } from '@/api/client'

const props = defineProps<{ sessionId: number }>()

const list = ref<AttendanceRow[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
let timer: ReturnType<typeof setInterval> | null = null

const onlineCount = computed(() => list.value.filter(s => s.is_online).length)
const offlineCount = computed(() => list.value.filter(s => !s.is_online).length)

async function refresh() {
  loading.value = true
  try {
    list.value = await api.attendance.forSession(props.sessionId)
    error.value = null
  } catch (e) {
    error.value = e instanceof Error ? e.message : '출결 조회 실패'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  refresh()
  timer = setInterval(refresh, 15000)
})

onUnmounted(() => {
  if (timer !== null) clearInterval(timer)
})
</script>

<style scoped>
.attendance-widget {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-lg);
  padding: 1rem 1.25rem;
}

.widget-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.widget-title {
  font-size: 14px;
  font-weight: 500;
}

.widget-summary {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.widget-summary .dot--offline {
  margin-left: 8px;
}

.dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.dot--online  { background: var(--color-text-success); }
.dot--offline { background: var(--color-text-tertiary); }

.widget-empty, .widget-error {
  font-size: 12px;
  color: var(--color-text-tertiary);
  text-align: center;
  padding: 8px 0;
}

.widget-error { color: var(--color-text-danger); }

.student-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 4px 8px;
  max-height: 260px;
  overflow-y: auto;
  margin-bottom: 8px;
}

.student-row {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 2px 0;
}

.student-name {
  flex: 1;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.name--late { color: var(--color-text-warning); }

.student-no {
  font-size: 11px;
  color: var(--color-text-tertiary);
  flex-shrink: 0;
}

.widget-note {
  font-size: 11px;
  color: var(--color-text-tertiary);
  text-align: right;
  margin-top: 4px;
}
</style>
