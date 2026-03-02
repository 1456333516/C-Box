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

      <!-- 8.5: Install button -->
      <button
        v-if="pack.state.type === 'not_installed' || pack.state.type === 'install_failed' || pack.state.type === 'download_failed'"
        class="btn"
        :disabled="isInstalling"
        @click="emit('install')"
      >
        {{ t('action.install') }}
      </button>

      <!-- Detect / Retry button -->
      <button
        v-else-if="pack.state.type === 'undetected' || pack.state.type === 'detect_failed'"
        class="btn"
        @click="emit('detect')"
      >
        {{ pack.state.type === 'detect_failed' ? t('action.retry') : t('action.detect') }}
      </button>
    </div>

    <!-- 8.9: Reboot notification banner -->
    <div
      v-if="pack.state.type === 'installed' && pack.state.data.pending_reboot"
      class="reboot-banner"
    >
      ⚠ 需要重启系统以完成安装
    </div>

    <!-- Error messages -->
    <p v-if="pack.state.type === 'detect_failed'" class="error-msg">
      {{ pack.state.data.reason }}
    </p>
    <p v-else-if="pack.state.type === 'install_failed'" class="error-msg">
      {{ pack.state.data.reason }}
    </p>

    <!-- 8.7: Collapsible install log panel -->
    <div v-if="logs.length > 0" class="log-section">
      <button class="log-toggle" @click="logOpen = !logOpen">
        {{ logOpen ? '▾' : '▸' }} 安装日志 ({{ logs.length }} 行)
      </button>
      <div v-if="logOpen" class="log-panel">
        <div
          v-for="(line, i) in logs"
          :key="i"
          class="log-line"
          :class="{ 'log-err': isErrLine(line) }"
        >{{ line }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePackStore } from '@/stores/pack'
import type { PackSummary } from '@/types/pack'

const props = defineProps<{ pack: PackSummary }>()
const emit = defineEmits<{ detect: []; install: [] }>()
const { t } = useI18n()
const store = usePackStore()

const logOpen = ref(false)
const isInstalling = computed(() => store.installing.has(props.pack.pack_id))
const logs = computed(() => store.installLogs[props.pack.pack_id] ?? [])

// Simple heuristic: lines with "error" or "fail" words (case-insensitive) shown in red
function isErrLine(line: string) {
  return /error|fail|exception/i.test(line)
}

const dotClass = computed(() => ({
  'dot-gray': props.pack.state.type === 'undetected',
  'dot-blue dot-pulse': props.pack.state.type === 'detecting',
  'dot-yellow dot-pulse': props.pack.state.type === 'downloading' || props.pack.state.type === 'installing',
  'dot-green': props.pack.state.type === 'installed' || props.pack.state.type === 'configured',
  'dot-orange': props.pack.state.type === 'not_installed',
  'dot-red': props.pack.state.type === 'detect_failed' || props.pack.state.type === 'install_failed' || props.pack.state.type === 'download_failed',
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
.dot-yellow { background: #eab308; }
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
.btn:hover:not(:disabled) {
  background: var(--c-badge, #f1f5f9);
}
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.reboot-banner {
  font-size: 0.78rem;
  color: #92400e;
  background: #fef3c7;
  border: 1px solid #fbbf24;
  padding: 6px 10px;
  border-radius: 4px;
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
.log-section {
  border-top: 1px solid var(--c-border, #e2e8f0);
  padding-top: 6px;
}
.log-toggle {
  all: unset;
  cursor: pointer;
  font-size: 0.75rem;
  color: var(--c-muted, #64748b);
}
.log-toggle:hover {
  color: #334155;
}
.log-panel {
  margin-top: 6px;
  max-height: 160px;
  overflow-y: auto;
  background: #0f172a;
  border-radius: 4px;
  padding: 8px;
  font-family: monospace;
  font-size: 0.72rem;
  line-height: 1.5;
}
.log-line {
  color: #cbd5e1;
  white-space: pre-wrap;
  word-break: break-all;
}
.log-err {
  color: #f87171;
}
</style>
