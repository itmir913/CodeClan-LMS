<template>
  <div class="min-h-screen" style="background: var(--color-bg-primary)">

    <!-- Top Nav -->
    <header class="sticky top-0 z-30 h-16 border-b"
            style="background: var(--color-bg-secondary); border-color: var(--color-border)">
      <div class="h-full max-w-full flex items-center gap-4 px-6">
        <button
          class="h-9 px-3 rounded-lg flex items-center gap-2 border bg-transparent font-medium"
          style="border-color: var(--color-border); color: var(--color-text-muted)"
          @click="router.push('/student')"
        >
          <IconArrowLeft :size="16" />
          {{ $t('classes.backToHome') }}
        </button>
        <span class="font-semibold" style="color: var(--color-text-primary)">
          {{ $t('student.classDetail') }}
        </span>
      </div>
    </header>

    <!-- Coming soon placeholder -->
    <main class="max-w-4xl mx-auto px-4 sm:px-6 py-16 flex flex-col items-center gap-4">
      <IconHourglass :size="48" style="color: var(--color-text-tertiary)" />
      <p class="font-semibold" style="color: var(--color-text-muted)">
        {{ $t('classes.tabComingSoon') }}
      </p>
    </main>

  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { IconArrowLeft, IconHourglass } from '@tabler/icons-vue'

const router = useRouter()
</script>
