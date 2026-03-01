import { onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { usePackStore } from '@/stores/pack'
import type { StateChangedPayload } from '@/types/pack'

export function usePackEvents() {
  const store = usePackStore()
  let unlisten: UnlistenFn | null = null

  onMounted(async () => {
    unlisten = await listen<StateChangedPayload>('pack:state-changed', ({ payload }) => {
      store.applyStateChange(payload.pack_id, payload.state)
    })
  })

  onUnmounted(() => {
    unlisten?.()
  })
}
