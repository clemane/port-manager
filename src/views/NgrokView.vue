<script setup lang="ts">
import { ref } from 'vue'
import { PmButton, PmInput, PmSelect, PmTable, PmBadge } from '@/components/ui'
import { useNgrok } from '@/composables/useNgrok'

const {
  tunnels,
  domains,
  addDomain,
  deleteDomain,
  createTunnel,
  killTunnel,
  restartTunnel,
  syncDomains,
  detectRunningTunnels,
} = useNgrok()

// Launch form
const selectedDomainId = ref('')
const localPort = ref('')
const launchError = ref('')

// Domain form
const newDomain = ref('')
const domainError = ref('')
const syncing = ref(false)
const syncError = ref('')
const detecting = ref(false)
const detectError = ref('')

const columns = [
  { key: 'domain', label: 'Domain', sortable: true },
  { key: 'local_port', label: 'Local Port', sortable: true, width: '100px' },
  { key: 'tunnel_url', label: 'Tunnel URL', sortable: false },
  { key: 'status', label: 'Status', sortable: true, width: '100px' },
  { key: 'actions', label: 'Actions', width: '140px' },
]

function statusVariant(status: string): 'running' | 'error' | 'stopped' {
  if (status === 'running') return 'running'
  if (status === 'error') return 'error'
  return 'stopped'
}

function domainOptions() {
  return domains.value.map((d) => ({ value: d.id, label: d.domain }))
}

async function handleLaunch() {
  launchError.value = ''
  if (!selectedDomainId.value) {
    launchError.value = 'Select a domain'
    return
  }
  const port = parseInt(localPort.value, 10)
  if (!port || port < 1 || port > 65535) {
    launchError.value = 'Enter a valid port (1-65535)'
    return
  }
  try {
    await createTunnel(selectedDomainId.value, port)
    localPort.value = ''
  } catch (e) {
    launchError.value = `${e}`
  }
}

async function handleAddDomain() {
  domainError.value = ''
  const d = newDomain.value.trim()
  if (!d) {
    domainError.value = 'Enter a domain'
    return
  }
  try {
    await addDomain(d)
    newDomain.value = ''
  } catch (e) {
    domainError.value = `${e}`
  }
}

function copyUrl(url: string) {
  navigator.clipboard.writeText(url)
}

async function handleSync() {
  syncing.value = true
  syncError.value = ''
  try {
    await syncDomains()
  } catch (e) {
    syncError.value = `${e}`
  } finally {
    syncing.value = false
  }
}

async function handleDetect() {
  detecting.value = true
  detectError.value = ''
  try {
    await detectRunningTunnels()
  } catch (e) {
    detectError.value = `${e}`
  } finally {
    detecting.value = false
  }
}
</script>

<template>
  <div class="ngrok">
    <div class="ngrok__header">
      <h1 class="view-title">Ngrok Tunnels</h1>
      <p class="view-subtitle">Manage ngrok HTTP tunnels with custom domains</p>
    </div>

    <!-- Launch Tunnel -->
    <section class="ngrok__section">
      <h2 class="section-title">Launch Tunnel</h2>
      <div class="launch-form">
        <PmSelect
          v-model="selectedDomainId"
          :options="domainOptions()"
          placeholder="Select domain"
        />
        <PmInput v-model="localPort" type="number" placeholder="Local port" />
        <PmButton @click="handleLaunch">Launch</PmButton>
      </div>
      <p v-if="launchError" class="form-error">{{ launchError }}</p>
    </section>

    <!-- Active Tunnels -->
    <section class="ngrok__section">
      <div class="section-header">
        <h2 class="section-title">Active Tunnels</h2>
        <PmButton size="sm" variant="ghost" :disabled="detecting" @click="handleDetect">
          {{ detecting ? 'Detecting...' : 'Detect running' }}
        </PmButton>
      </div>
      <p v-if="detectError" class="form-error">{{ detectError }}</p>
      <PmTable :data="tunnels" :columns="columns">
        <template #cell-domain="{ value }">
          <span class="mono">{{ value }}</span>
        </template>
        <template #cell-local_port="{ value }">
          <span class="mono">{{ value }}</span>
        </template>
        <template #cell-tunnel_url="{ value }">
          <span v-if="value" class="mono tunnel-url">{{ value }}</span>
          <span v-else class="text-muted">--</span>
        </template>
        <template #cell-status="{ value }">
          <PmBadge :variant="statusVariant(value)">{{ value }}</PmBadge>
        </template>
        <template #cell-actions="{ row }">
          <div class="action-btns">
            <PmButton
              v-if="row.status === 'running'"
              size="sm"
              variant="danger"
              @click="killTunnel(row.id)"
            >Kill</PmButton>
            <PmButton
              v-if="row.status !== 'running'"
              size="sm"
              variant="ghost"
              @click="restartTunnel(row.id)"
            >Restart</PmButton>
            <PmButton
              v-if="row.tunnel_url"
              size="sm"
              variant="icon"
              title="Copy URL"
              @click="copyUrl(row.tunnel_url)"
            >&#x1F4CB;</PmButton>
          </div>
        </template>
      </PmTable>
    </section>

    <!-- Domains -->
    <section class="ngrok__section">
      <div class="section-header">
        <h2 class="section-title">Reserved Domains</h2>
        <PmButton size="sm" variant="ghost" :disabled="syncing" @click="handleSync">
          {{ syncing ? 'Syncing...' : 'Sync from ngrok' }}
        </PmButton>
      </div>
      <p v-if="syncError" class="form-error">{{ syncError }}</p>
      <div v-if="domains.length > 0" class="domain-list">
        <div v-for="d in domains" :key="d.id" class="domain-item">
          <span class="domain-name">{{ d.domain }}</span>
          <PmButton size="sm" variant="danger" @click="deleteDomain(d.id)">Delete</PmButton>
        </div>
      </div>
      <p v-else class="empty-text">No domains added yet</p>
      <div class="domain-form">
        <PmInput v-model="newDomain" placeholder="e.g. myapp.ngrok-free.app" />
        <PmButton variant="ghost" @click="handleAddDomain">+ Add Domain</PmButton>
      </div>
      <p v-if="domainError" class="form-error">{{ domainError }}</p>
    </section>
  </div>
</template>

<style scoped>
.ngrok__header { margin-bottom: 20px; }
.view-title { font-size: 20px; font-weight: 600; color: var(--pm-text-primary); margin: 0 0 4px; }
.view-subtitle { font-size: 13px; color: var(--pm-text-secondary); margin: 0; }

.ngrok__section {
  margin-bottom: 32px;
  padding-bottom: 24px;
  border-bottom: 1px solid var(--pm-border-subtle);
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--pm-text-primary);
  margin: 0;
}

.section-header + .form-error {
  margin-top: -6px;
  margin-bottom: 8px;
}

.launch-form {
  display: flex;
  align-items: center;
  gap: 8px;
  max-width: 600px;
}

.form-error {
  color: var(--pm-danger);
  font-size: 12px;
  margin: 6px 0 0;
}

.mono { font-family: 'JetBrains Mono', 'Fira Code', monospace; font-size: 13px; }
.tunnel-url { color: var(--pm-accent); }
.text-muted { color: var(--pm-text-muted); font-size: 13px; }
.action-btns { display: flex; gap: 4px; }

.domain-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.domain-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--pm-surface);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius);
  padding: 10px 12px;
  max-width: 500px;
}

.domain-name {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 13px;
  color: var(--pm-text-primary);
}

.empty-text {
  color: var(--pm-text-muted);
  font-size: 13px;
  margin-bottom: 12px;
}

.domain-form {
  display: flex;
  gap: 8px;
  align-items: center;
  max-width: 500px;
}
</style>
