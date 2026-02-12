import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface ActiveForward {
  id: string
  favorite_id: string | null
  kubeconfig_id: string
  namespace: string
  resource_type: string
  resource_name: string
  remote_port: number
  local_port: number
  pid: number | null
  status: string
  started_at: string | null
  error_msg: string | null
}

export interface Favorite {
  id: string
  kubeconfig_id: string
  namespace: string
  resource_type: string
  resource_name: string
  remote_port: number
  local_port: number | null
  label: string
  group_name: string | null
}

export function useForwards() {
  const forwards = ref<ActiveForward[]>([])
  const favorites = ref<Favorite[]>([])
  const loading = ref(false)
  let interval: ReturnType<typeof setInterval> | null = null

  async function loadForwards() {
    forwards.value = await invoke<ActiveForward[]>('list_forwards')
  }

  async function loadFavorites() {
    favorites.value = await invoke<Favorite[]>('list_favorites')
  }

  async function killForward(id: string) {
    await invoke('kill_forward', { id })
    await loadForwards()
  }

  async function restartForward(id: string) {
    await invoke('restart_forward', { id })
    await loadForwards()
  }

  async function launchFavorite(fav: Favorite) {
    await invoke('create_forward', {
      kubeconfigId: fav.kubeconfig_id,
      namespace: fav.namespace,
      resourceType: fav.resource_type,
      resourceName: fav.resource_name,
      remotePort: fav.remote_port,
      localPort: fav.local_port,
      favoriteId: fav.id,
    })
    await loadForwards()
  }

  async function deleteFavorite(id: string) {
    await invoke('delete_favorite', { id })
    await loadFavorites()
  }

  async function saveFavorite(forward: ActiveForward, label: string, groupName?: string) {
    await invoke('save_favorite', {
      req: {
        kubeconfig_id: forward.kubeconfig_id,
        namespace: forward.namespace,
        resource_type: forward.resource_type,
        resource_name: forward.resource_name,
        remote_port: forward.remote_port,
        local_port: forward.local_port,
        label,
        group_name: groupName ?? null,
      }
    })
    await loadFavorites()
  }

  function startPolling() {
    loadForwards()
    loadFavorites()
    interval = setInterval(loadForwards, 3000)
  }

  function stopPolling() {
    if (interval) {
      clearInterval(interval)
      interval = null
    }
  }

  onMounted(() => startPolling())
  onUnmounted(() => stopPolling())

  return { forwards, favorites, loading, killForward, restartForward, launchFavorite, deleteFavorite, saveFavorite, loadForwards, loadFavorites }
}
