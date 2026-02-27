<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import AppLayout from '@/components/layout/AppLayout.vue'
import LoginView from '@/views/LoginView.vue'
import { PmMatrixRain, PmModal, PmButton } from '@/components/ui'
import { useTheme, type Theme } from '@/composables/useTheme'
import { useAuth } from '@/composables/useAuth'
import { useUpdater } from '@/composables/useUpdater'

const { currentTheme, loadTheme, setTheme } = useTheme()
const { isUnlocked, checkStatus, lock, resetActivity } = useAuth()
const {
  updateAvailable, newVersion, releaseNotes,
  downloading, progress, error: updateError,
  checkForUpdate, downloadAndInstall, dismiss
} = useUpdater()

const forwardCount = ref(0)
const listeningPorts = ref(0)
const tunnelCount = ref(0)
const showUpdateModal = ref(false)
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

function openUpdateModal() {
  showUpdateModal.value = true
}

function closeUpdateModal() {
  showUpdateModal.value = false
}

function dismissUpdate() {
  dismiss()
  closeUpdateModal()
}

onMounted(async () => {
  await loadTheme()
  await checkStatus()
  refreshCounts()
  countInterval = setInterval(refreshCounts, 5000)
  checkForUpdate()

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
    :update-available="updateAvailable"
    :new-version="newVersion"
    @theme-change="onThemeChange"
    @show-update="openUpdateModal"
  />
  <LoginView v-else />

  <PmModal :open="showUpdateModal" title="Update Available" @close="closeUpdateModal">
    <p class="update-version">Version {{ newVersion }} is available</p>
    <div v-if="releaseNotes" class="update-notes">
      <p class="update-notes__label">Release notes:</p>
      <pre class="update-notes__content">{{ releaseNotes }}</pre>
    </div>
    <div v-if="downloading" class="update-progress">
      <div class="update-progress__bar">
        <div class="update-progress__fill" :style="{ width: progress + '%' }" />
      </div>
      <span class="update-progress__text">{{ progress }}%</span>
    </div>
    <p v-if="updateError" class="update-error">{{ updateError }}</p>
    <template #footer>
      <PmButton variant="ghost" :disabled="downloading" @click="dismissUpdate">Later</PmButton>
      <PmButton :loading="downloading" @click="downloadAndInstall">Install &amp; Restart</PmButton>
    </template>
  </PmModal>
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

.update-version {
  font-size: 14px;
  color: var(--pm-text-primary);
  margin-bottom: 12px;
}
.update-notes__label {
  font-size: 12px;
  color: var(--pm-text-muted);
  margin-bottom: 6px;
}
.update-notes__content {
  font-family: var(--pm-font-mono, monospace);
  font-size: 12px;
  color: var(--pm-text-secondary);
  background: var(--pm-bg);
  padding: 12px;
  border-radius: var(--pm-radius-sm);
  border: 1px solid var(--pm-border-subtle);
  max-height: 200px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-word;
}
.update-progress {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 12px;
}
.update-progress__bar {
  flex: 1;
  height: 6px;
  background: var(--pm-bg);
  border-radius: 3px;
  overflow: hidden;
}
.update-progress__fill {
  height: 100%;
  background: var(--pm-accent);
  transition: width 0.3s ease;
}
.update-progress__text {
  font-size: 12px;
  color: var(--pm-text-muted);
  min-width: 36px;
  text-align: right;
}
.update-error {
  color: var(--pm-danger);
  font-size: 12px;
  margin-top: 8px;
}
</style>
