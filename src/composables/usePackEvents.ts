import { onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { usePackStore } from '@/stores/pack'
import type { InstallOutputPayload, StateChangedPayload } from '@/types/pack'

export function usePackEvents() {
  const store = usePackStore()
  const unlisteners: UnlistenFn[] = []

  onMounted(async () => {
    unlisteners.push(
      await listen<StateChangedPayload>('pack:state-changed', ({ payload }) => {
        store.applyStateChange(payload.pack_id, payload.state)
      }),
      // 5.6 / 8.7: stream install output into per-pack log
      await listen<InstallOutputPayload>('pack:install-output', ({ payload }) => {
        store.appendInstallLog(payload)
      }),
    )
  })

  onUnmounted(() => {
    unlisteners.forEach((fn) => fn())
  })
}
