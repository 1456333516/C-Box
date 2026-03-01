<template>
  <div class="pack-card">
    <div class="pack-header">
      <span class="pack-name">{{ pack.name }}</span>
      <span class="category-badge">{{ t(`pack.category.${pack.category}`, pack.category) }}</span>
    </div>

    <p class="pack-desc">{{ pack.description }}</p>

    <div class="pack-footer">
      <div class="status-row">
        <span class="status-dot" :class="dotClass" />
        <span class="status-text">{{ t(`pack.state.${pack.state.type}`) }}</span>
        <span v-if="pack.installed_version" class="version">v{{ pack.installed_version }}</span>
      </div>
      <button
        v-if="pack.state.type === 'undetected' || pack.state.type === 'detect_failed'"
        class="btn"
        @click="emit('detect')"
      >
        {{ t('action.detect') }}
      </button>
    </div>

    <p
      v-if="pack.state.type === 'detect_failed'"
      class="error-msg"
    >
      {{ pack.state.data.reason }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { PackSummary } from '@/types/pack'

const props = defineProps<{ pack: PackSummary }>()
const emit = defineEmits<{ detect: [] }>()
const { t } = useI18n()

const dotClass = computed(() => ({
  'dot-gray': props.pack.state.type === 'undetected',
  'dot-blue dot-pulse': props.pack.state.type === 'detecting',
  'dot-green': props.pack.state.type === 'installed',
  'dot-orange': props.pack.state.type === 'not_installed',
  'dot-red': props.pack.state.type === 'detect_failed',
}))
</script>

<style scoped>
.pack-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
  border: 1px solid var(--c-border, #e2e8f0);
  border-radius: 8px;
  background: var(--c-surface, #fff);
  transition: box-shadow 0.15s;
}
.pack-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}
.pack-header {
  display: flex;
  align-items: center;
  gap: 8px;
}
.pack-name {
  flex: 1;
  font-weight: 600;
  font-size: 0.95rem;
}
.category-badge {
  font-size: 0.7rem;
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--c-badge, #f1f5f9);
  color: var(--c-muted, #64748b);
}
.pack-desc {
  margin: 0;
  font-size: 0.85rem;
  color: var(--c-muted, #64748b);
  line-height: 1.4;
}
.pack-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 4px;
}
.status-row {
  display: flex;
  align-items: center;
  gap: 6px;
}
.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dot-gray   { background: #94a3b8; }
.dot-blue   { background: #3b82f6; }
.dot-green  { background: #22c55e; }
.dot-orange { background: #f97316; }
.dot-red    { background: #ef4444; }
.dot-pulse  { animation: pulse 1.2s ease-in-out infinite; }
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.35; }
}
.status-text {
  font-size: 0.8rem;
  color: var(--c-muted, #64748b);
}
.version {
  font-size: 0.75rem;
  color: #22c55e;
  font-variant-numeric: tabular-nums;
}
.btn {
  padding: 4px 14px;
  border: 1px solid var(--c-border, #e2e8f0);
  border-radius: 6px;
  background: transparent;
  font-size: 0.8rem;
  cursor: pointer;
  transition: background 0.1s;
}
.btn:hover {
  background: var(--c-badge, #f1f5f9);
}
.error-msg {
  margin: 0;
  font-size: 0.78rem;
  color: #ef4444;
  background: #fef2f2;
  padding: 6px 10px;
  border-radius: 4px;
  word-break: break-word;
}
</style>
