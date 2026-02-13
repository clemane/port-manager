import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface SystemPort {
  protocol: string
  local_port: number
  remote_port: number
  state: string
  pid: number | null
  process_name: string | null
}

export function usePorts() {
  const ports = ref<SystemPort[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  let interval: ReturnType<typeof setInterval> | null = null

  async function refresh() {
    if (ports.value.length === 0) loading.value = true
    error.value = null
    try {
      ports.value = await invoke<SystemPort[]>('get_system_ports')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  function startPolling(ms = 5000) {
    refresh()
    interval = setInterval(refresh, ms)
  }

  function stopPolling() {
    if (interval) {
      clearInterval(interval)
      interval = null
    }
  }

  onMounted(() => startPolling())
  onUnmounted(() => stopPolling())

  return { ports, loading, error, refresh }
}
