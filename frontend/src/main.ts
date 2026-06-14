import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import App from './App.vue'
import router from './router'
import './assets/main.css'

// locales/{lang}/*.json 파일을 자동으로 수집해 { [namespace]: messages } 형태로 병합.
// 새 기능 파일(auth.json, dashboard.json 등)을 추가해도 이 파일은 수정 불필요.
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

const i18n = createI18n({
  legacy: false,
  locale: 'ko',
  fallbackLocale: 'ko',
  messages: {
    ko: buildMessages(koModules as LocaleModules),
    en: buildMessages(enModules as LocaleModules),
  },
})

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.use(i18n)
app.mount('#app')
