import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface NgrokDomain {
  id: string
  domain: string
  created_at: string
}

export interface NgrokTunnel {
  id: string
  domain_id: string | null
  domain: string
  local_port: number
  pid: number | null
  status: string
  tunnel_url: string | null
  started_at: string | null
  error_msg: string | null
}

export function useNgrok() {
  const tunnels = ref<NgrokTunnel[]>([])
  const domains = ref<NgrokDomain[]>([])
  const loading = ref(false)
  let interval: ReturnType<typeof setInterval> | null = null

  async function loadTunnels() {
    tunnels.value = await invoke<NgrokTunnel[]>('list_tunnels')
  }

  async function loadDomains() {
    domains.value = await invoke<NgrokDomain[]>('list_ngrok_domains')
  }

  async function addDomain(domain: string) {
    await invoke('add_ngrok_domain', { domain })
    await loadDomains()
  }

  async function deleteDomain(id: string) {
    await invoke('delete_ngrok_domain', { id })
    await loadDomains()
  }

  async function createTunnel(domainId: string, localPort: number) {
    await invoke('create_tunnel', { domainId, localPort })
    await loadTunnels()
  }

  async function killTunnel(id: string) {
    await invoke('kill_tunnel', { id })
    await loadTunnels()
  }

  async function restartTunnel(id: string) {
    await invoke('restart_tunnel', { id })
    await loadTunnels()
  }

  async function syncDomains() {
    domains.value = await invoke<NgrokDomain[]>('sync_ngrok_domains')
  }

  async function detectRunningTunnels() {
    tunnels.value = await invoke<NgrokTunnel[]>('detect_running_tunnels')
  }

  function startPolling() {
    loadTunnels()
    loadDomains()
    interval = setInterval(loadTunnels, 3000)
  }

  function stopPolling() {
    if (interval) {
      clearInterval(interval)
      interval = null
    }
  }

  onMounted(() => startPolling())
  onUnmounted(() => stopPolling())

  return {
    tunnels,
    domains,
    loading,
    loadTunnels,
    loadDomains,
    addDomain,
    deleteDomain,
    createTunnel,
    killTunnel,
    restartTunnel,
    syncDomains,
    detectRunningTunnels,
  }
}
