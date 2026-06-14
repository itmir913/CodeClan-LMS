import { createI18n } from 'vue-i18n'

type LocaleModules = Record<string, { default: Record<string, string> }>

function buildMessages(modules: LocaleModules): Record<string, Record<string, string>> {
  const result: Record<string, Record<string, string>> = {}
  for (const [path, mod] of Object.entries(modules)) {
    const namespace = path.replace(/^\.\/locales\/\w+\//, '').replace(/\.json$/, '')
    result[namespace] = mod.default
  }
  return result
}

const koModules = import.meta.glob<{ default: Record<string, string> }>(
  './locales/ko/*.json',
  { eager: true },
)
const enModules = import.meta.glob<{ default: Record<string, string> }>(
  './locales/en/*.json',
  { eager: true },
)

export const i18n = createI18n({
  legacy: false,
  locale: 'ko',
  fallbackLocale: 'ko',
  messages: {
    ko: buildMessages(koModules as LocaleModules),
    en: buildMessages(enModules as LocaleModules),
  },
})
