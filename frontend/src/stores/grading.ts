import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api, type SubmissionRow } from '@/api/client'

export const useGradingStore = defineStore('grading', () => {
  const submissions = ref<SubmissionRow[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 학생별로 그룹화
  const byStudent = computed(() => {
    const map = new Map<number, { name: string; student_number: string; subs: SubmissionRow[] }>()
    for (const sub of submissions.value) {
      if (!map.has(sub.student_id)) {
        map.set(sub.student_id, {
          name: sub.student_name,
          student_number: sub.student_number,
          subs: [],
        })
      }
      map.get(sub.student_id)!.subs.push(sub)
    }
    return Array.from(map.entries())
      .sort((a, b) => a[1].student_number.localeCompare(b[1].student_number))
      .map(([id, val]) => ({ student_id: id, ...val }))
  })

  async function fetchSubmissions(sessionId: number) {
    loading.value = true
    error.value = null
    try {
      submissions.value = await api.submissions.forSession(sessionId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '제출물을 불러오지 못했습니다'
    } finally {
      loading.value = false
    }
  }

  async function gradeSubmission(submissionId: number, score: number) {
    await api.submissions.grade(submissionId, score)
    const idx = submissions.value.findIndex(s => s.id === submissionId)
    if (idx !== -1) {
      submissions.value[idx] = { ...submissions.value[idx], score, verdict: 'GRADED' }
    }
  }

  return { submissions, byStudent, loading, error, fetchSubmissions, gradeSubmission }
})
