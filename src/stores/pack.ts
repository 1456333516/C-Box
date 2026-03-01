import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { PackId, PackState, PackSummary } from '@/types/pack'

export const usePackStore = defineStore('pack', () => {
  const packs = ref<PackSummary[]>([])
  const detecting = ref(false)
  const initialized = ref(false)

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

  function applyStateChange(packId: PackId, state: PackState) {
    const pack = packs.value.find((p) => p.pack_id === packId)
    if (!pack) return
    pack.state = state
    pack.installed_version =
      state.type === 'installed' ? state.data.version : null
  }

  return { packs, detecting, initialized, init, detectAll, detectOne, applyStateChange }
})
