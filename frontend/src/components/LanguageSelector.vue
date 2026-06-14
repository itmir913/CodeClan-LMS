<template>
  <div class="relative" ref="containerRef">
    <button
      type="button"
      class="flex items-center gap-1.5 rounded-md"
      style="height: 36px; padding: 0 11px; background: transparent; border: 1px solid var(--color-border); color: var(--color-text-muted);"
      @click="open = !open"
    >
      <IconLanguage :size="16" />
      <span class="font-medium">{{ currentLabel }}</span>
      <IconChevronDown :size="14" :style="{ transform: open ? 'rotate(180deg)' : '', transition: 'transform 0.15s' }" />
    </button>

    <Transition name="dropdown">
      <div
        v-if="open"
        class="absolute right-0 mt-1 rounded-md overflow-hidden z-50"
        style="min-width: 120px; background: var(--color-bg-secondary); border: 1px solid var(--color-border); box-shadow: var(--shadow-dropdown);"
      >
        <button
          v-for="lang in languages"
          :key="lang.code"
          type="button"
          class="w-full flex items-center gap-2 px-4 py-2.5 text-left"
          :style="{
            background: locale === lang.code ? 'var(--color-info-bg)' : 'transparent',
            color: locale === lang.code ? 'var(--color-accent)' : 'var(--color-text-primary)',
            border: 'none',
            fontWeight: locale === lang.code ? '600' : '400',
          }"
          @click="select(lang.code)"
        >
          <span>{{ lang.flag }}</span>
          <span>{{ lang.label }}</span>
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { IconLanguage, IconChevronDown } from '@tabler/icons-vue'

const { locale } = useI18n()

const open = ref(false)
const containerRef = ref<HTMLElement | null>(null)

const languages = [
  { code: 'ko', label: '한국어', flag: '🇰🇷' },
  { code: 'en', label: 'English', flag: '🇺🇸' },
]

const currentLabel = computed(() =>
  languages.find(l => l.code === locale.value)?.label ?? locale.value
)

function select(code: string) {
  locale.value = code
  open.value = false
}

function onOutsideClick(e: MouseEvent) {
  if (containerRef.value && !containerRef.value.contains(e.target as Node)) {
    open.value = false
  }
}

onMounted(() => document.addEventListener('mousedown', onOutsideClick))
onUnmounted(() => document.removeEventListener('mousedown', onOutsideClick))
</script>

<style scoped>
.dropdown-enter-active,
.dropdown-leave-active {
  transition: opacity 0.1s, transform 0.1s;
}
.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
