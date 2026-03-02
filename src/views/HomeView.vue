<template>
  <main class="home">
    <header class="home-header">
      <h1>{{ t('app.title') }}</h1>
      <div class="header-actions">
        <button class="btn-primary" :disabled="store.detecting" @click="store.detectAll()">
          <span v-if="store.detecting" class="spinner" />
          {{ t('action.detect_all') }}
        </button>
        <!-- 8.6: Install all missing -->
        <button class="btn-secondary" @click="store.installAllMissing()">
          {{ t('action.install_all') }}
        </button>
      </div>
    </header>

    <div v-if="!store.initialized" class="loading">
      <span class="spinner" /> 加载中...
    </div>

    <div v-else-if="store.packs.length === 0" class="empty">
      未找到任何 Pack，请检查 packs/ 目录。
    </div>

    <div v-else class="pack-grid">
      <PackCard
        v-for="pack in store.packs"
        :key="pack.pack_id"
        :pack="pack"
        @detect="store.detectOne(pack.pack_id)"
        @install="store.installOne(pack.pack_id)"
      />
    </div>
  </main>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import PackCard from '@/components/PackCard.vue'
import { usePackStore } from '@/stores/pack'
import { usePackEvents } from '@/composables/usePackEvents'

const { t } = useI18n()
const store = usePackStore()

usePackEvents()

onMounted(() => store.init())
</script>

<style scoped>
.home {
  padding: 24px;
  max-width: 960px;
  margin: 0 auto;
}
.home-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
}
.header-actions {
  display: flex;
  gap: 8px;
}
.home-header h1 {
  font-size: 1.4rem;
  font-weight: 700;
  margin: 0;
}
.pack-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}
.btn-primary {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 20px;
  background: #3b82f6;
  color: #fff;
  border: none;
  border-radius: 8px;
  font-size: 0.9rem;
  cursor: pointer;
  transition: background 0.15s;
}
.btn-primary:hover:not(:disabled) {
  background: #2563eb;
}
.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.btn-secondary {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 20px;
  background: transparent;
  color: #3b82f6;
  border: 1px solid #3b82f6;
  border-radius: 8px;
  font-size: 0.9rem;
  cursor: pointer;
  transition: background 0.15s;
}
.btn-secondary:hover {
  background: #eff6ff;
}
.spinner {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
.loading,
.empty {
  text-align: center;
  padding: 60px 20px;
  color: #94a3b8;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
}
</style>
