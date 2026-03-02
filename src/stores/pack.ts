import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { InstallOutputPayload, PackId, PackState, PackSummary } from '@/types/pack'

export const usePackStore = defineStore('pack', () => {
  const packs = ref<PackSummary[]>([])
  const detecting = ref(false)
  const initialized = ref(false)
  // Per-pack install log lines (8.7)
  const installLogs = ref<Record<PackId, string[]>>({})
  // Track which packs are currently installing (8.5)
  const installing = ref<Set<PackId>>(new Set())

  async function init() {
    packs.value = await invoke<PackSummary[]>('load_packs')
    initialized.value = true
  }

  async function detectAll() {
    detecting.value = true
    try {
      await invoke('detect_all')
    } finally {
      detecting.value = false
    }
  }

  async function detectOne(packId: PackId) {
    await invoke('detect_pack', { packId })
  }

  // 8.5: Per-pack install
  async function installOne(packId: PackId) {
    installLogs.value[packId] = []
    installing.value = new Set([...installing.value, packId])
    try {
      await invoke('install_pack', { packId })
    } finally {
      installing.value = new Set([...installing.value].filter((id) => id !== packId))
    }
  }

  // 8.6: Install all not-installed packs
  async function installAllMissing() {
    const missing = packs.value.filter((p) => p.state.type === 'not_installed')
    for (const pack of missing) {
      await installOne(pack.pack_id)
    }
  }

  function applyStateChange(packId: PackId, state: PackState) {
    const pack = packs.value.find((p) => p.pack_id === packId)
    if (!pack) return
    pack.state = state
    pack.installed_version =
      state.type === 'installed' ? state.data.version : null
  }

  // 8.7: Append install output line to per-pack log
  function appendInstallLog(payload: InstallOutputPayload) {
    if (!installLogs.value[payload.pack_id]) {
      installLogs.value[payload.pack_id] = []
    }
    installLogs.value[payload.pack_id].push(payload.line)
  }

  return {
    packs,
    detecting,
    initialized,
    installLogs,
    installing,
    init,
    detectAll,
    detectOne,
    installOne,
    installAllMissing,
    applyStateChange,
    appendInstallLog,
  }
})
