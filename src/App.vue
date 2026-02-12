<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppLayout from '@/components/layout/AppLayout.vue'
import { useTheme, type Theme } from '@/composables/useTheme'

const { currentTheme, loadTheme, setTheme } = useTheme()

const forwardCount = ref(0)
const listeningPorts = ref(0)
let countInterval: ReturnType<typeof setInterval> | null = null

async function refreshCounts() {
  try {
    const forwards = await invoke<{ status: string }[]>('list_forwards')
    forwardCount.value = forwards.filter(f => f.status === 'running').length
    const ports = await invoke<{ state: string }[]>('get_system_ports')
    listeningPorts.value = ports.filter(p => p.state === 'LISTEN').length
  } catch {
    // Ignore on startup before backend is ready
  }
}

onMounted(async () => {
  await loadTheme()
  refreshCounts()
  countInterval = setInterval(refreshCounts, 5000)
})

onUnmounted(() => {
  if (countInterval) {
    clearInterval(countInterval)
    countInterval = null
  }
})

function onThemeChange(theme: string) {
  setTheme(theme as Theme)
}
</script>

<template>
  <AppLayout
    :current-theme="currentTheme"
    :forward-count="forwardCount"
    :listening-ports="listeningPorts"
    @theme-change="onThemeChange"
  >
    <router-view />
  </AppLayout>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  background: var(--pm-bg);
  color: var(--pm-text-primary);
  -webkit-font-smoothing: antialiased;
}

::-webkit-scrollbar { width: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--pm-border); border-radius: 3px; }
</style>
