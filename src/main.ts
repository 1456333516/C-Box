import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import App from './App.vue'
import zhCN from './locales/zh-CN.json'
import './styles/global.css'

const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  messages: { 'zh-CN': zhCN },
})

createApp(App).use(createPinia()).use(i18n).mount('#app')
