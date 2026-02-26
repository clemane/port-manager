<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import AppLayout from '@/components/layout/AppLayout.vue'
import LoginView from '@/views/LoginView.vue'
import { PmMatrixRain } from '@/components/ui'
import { useTheme, type Theme } from '@/composables/useTheme'
import { useAuth } from '@/composables/useAuth'

const { currentTheme, loadTheme, setTheme } = useTheme()
const { isUnlocked, checkStatus, lock, resetActivity } = useAuth()

const forwardCount = ref(0)
const listeningPorts = ref(0)
const tunnelCount = ref(0)
let countInterval: ReturnType<typeof setInterval> | null = null
let unlistenClose: (() => void) | null = null

async function refreshCounts() {
  try {
    const forwards = await invoke<{ status: string }[]>('list_forwards')
    forwardCount.value = forwards.filter(f => f.status === 'running').length
    const ports = await invoke<{ state: string }[]>('get_system_ports')
    listeningPorts.value = ports.filter(p => p.state === 'LISTEN').length
    const tunnels = await invoke<{ status: string }[]>('list_tunnels')
    tunnelCount.value = tunnels.filter(t => t.status === 'running').length
  } catch {
    // Ignore on startup before backend is ready
  }
}

function onActivity() {
  resetActivity()
}

onMounted(async () => {
  await loadTheme()
  await checkStatus()
  refreshCounts()
  countInterval = setInterval(refreshCounts, 5000)

  document.addEventListener('mousemove', onActivity)
  document.addEventListener('keydown', onActivity)

  try {
    unlistenClose = await listen('tauri://close-requested', () => {
      lock()
    })
  } catch {
    // Silently fail if event not available
  }
})

onUnmounted(() => {
  if (countInterval) {
    clearInterval(countInterval)
    countInterval = null
  }
  document.removeEventListener('mousemove', onActivity)
  document.removeEventListener('keydown', onActivity)
  if (unlistenClose) {
    unlistenClose()
    unlistenClose = null
  }
})

function onThemeChange(theme: string) {
  setTheme(theme as Theme)
}
</script>

<template>
  <PmMatrixRain :active="currentTheme === 'matrix'" />
  <AppLayout
    v-if="isUnlocked"
    :current-theme="currentTheme"
    :forward-count="forwardCount"
    :listening-ports="listeningPorts"
    :tunnel-count="tunnelCount"
    @theme-change="onThemeChange"
  />
  <LoginView v-else />
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: var(--pm-font-body), sans-serif;
  background: var(--pm-bg);
  color: var(--pm-text-primary);
  -webkit-font-smoothing: antialiased;
}

::-webkit-scrollbar { width: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--pm-border); border-radius: 3px; }
</style>
