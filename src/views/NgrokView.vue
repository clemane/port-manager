<script setup lang="ts">
import { ref } from 'vue'
import { PmButton, PmInput, PmSelect, PmTable, PmStatusDot } from '@/components/ui'
import { useNgrok } from '@/composables/useNgrok'

const {
  tunnels, domains, addDomain, deleteDomain,
  createTunnel, killTunnel, restartTunnel, syncDomains, detectRunningTunnels,
} = useNgrok()

const selectedDomainId = ref('')
const localPort = ref('')
const launchError = ref('')
const newDomain = ref('')
const domainError = ref('')
const syncing = ref(false)
const syncError = ref('')
const detecting = ref(false)
const detectError = ref('')
const copiedId = ref<string | null>(null)

const columns = [
  { key: 'domain', label: 'Domain', sortable: true },
  { key: 'local_port', label: 'Local Port', sortable: true, width: '100px' },
  { key: 'tunnel_url', label: 'Tunnel URL', sortable: false },
  { key: 'status', label: 'Status', sortable: true, width: '110px' },
  { key: 'actions', label: 'Actions', width: '140px' },
]

function domainOptions() {
  return domains.value.map((d) => ({ value: d.id, label: d.domain }))
}

async function handleLaunch() {
  launchError.value = ''
  if (!selectedDomainId.value) { launchError.value = 'Select a domain'; return }
  const port = parseInt(localPort.value, 10)
  if (!port || port < 1 || port > 65535) { launchError.value = 'Enter a valid port (1-65535)'; return }
  try {
    await createTunnel(selectedDomainId.value, port)
    localPort.value = ''
  } catch (e) { launchError.value = `${e}` }
}

async function handleAddDomain() {
  domainError.value = ''
  const d = newDomain.value.trim()
  if (!d) { domainError.value = 'Enter a domain'; return }
  try {
    await addDomain(d)
    newDomain.value = ''
  } catch (e) { domainError.value = `${e}` }
}

function copyUrl(id: string, url: string) {
  navigator.clipboard.writeText(url)
  copiedId.value = id
  setTimeout(() => { copiedId.value = null }, 1500)
}

async function handleSync() {
  syncing.value = true; syncError.value = ''
  try { await syncDomains() } catch (e) { syncError.value = `${e}` } finally { syncing.value = false }
}

async function handleDetect() {
  detecting.value = true; detectError.value = ''
  try { await detectRunningTunnels() } catch (e) { detectError.value = `${e}` } finally { detecting.value = false }
}
</script>

<template>
  <div class="ngrok">
    <!-- Launch Tunnel Card -->
    <div class="ngrok__launch-card">
      <div class="launch-card__content">
        <PmSelect v-model="selectedDomainId" :options="domainOptions()" placeholder="Select domain" />
        <PmInput v-model="localPort" type="number" placeholder="Local port" />
        <PmButton @click="handleLaunch">Launch</PmButton>
      </div>
      <p v-if="launchError" class="form-error">{{ launchError }}</p>
    </div>

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
          <span class="mono-data">{{ value }}</span>
        </template>
        <template #cell-local_port="{ value }">
          <span class="mono-data">{{ value }}</span>
        </template>
        <template #cell-tunnel_url="{ value }">
          <span v-if="value" class="tunnel-url">{{ value }}</span>
          <span v-else class="text-muted">--</span>
        </template>
        <template #cell-status="{ row }">
          <span class="status-cell">
            <PmStatusDot :status="row.status === 'running' ? 'running' : row.status === 'error' ? 'error' : 'stopped'" />
            <span>{{ row.status }}</span>
          </span>
        </template>
        <template #cell-actions="{ row }">
          <div class="action-btns">
            <PmButton v-if="row.status === 'running'" size="sm" variant="danger" @click="killTunnel(row.id)">Kill</PmButton>
            <PmButton v-else size="sm" variant="ghost" @click="restartTunnel(row.id)">Restart</PmButton>
            <PmButton v-if="row.tunnel_url" size="sm" variant="icon" :title="copiedId === row.id ? 'Copied!' : 'Copy URL'" @click="copyUrl(row.id, row.tunnel_url)">
              <svg v-if="copiedId !== row.id" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="14" height="14"><rect x="5" y="5" width="8" height="8" rx="1"/><path d="M3 11V3h8" stroke-linecap="round"/></svg>
              <svg v-else viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="14" height="14"><path d="M4 8l3 3 5-5" stroke-linecap="round" stroke-linejoin="round"/></svg>
            </PmButton>
          </div>
        </template>
      </PmTable>
    </section>

    <!-- Reserved Domains -->
    <section class="ngrok__section">
      <div class="section-header">
        <h2 class="section-title">Reserved Domains</h2>
        <PmButton size="sm" variant="ghost" :disabled="syncing" @click="handleSync">
          {{ syncing ? 'Syncing...' : 'Sync from ngrok' }}
        </PmButton>
      </div>
      <p v-if="syncError" class="form-error">{{ syncError }}</p>
      <div class="domain-chips">
        <div v-for="d in domains" :key="d.id" class="domain-chip">
          <span class="domain-chip__text">{{ d.domain }}</span>
          <button class="domain-chip__delete" @click="deleteDomain(d.id)" title="Remove">
            <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5" width="10" height="10"><path d="M3 3l6 6M9 3l-6 6" stroke-linecap="round"/></svg>
          </button>
        </div>
        <div class="domain-add">
          <PmInput v-model="newDomain" placeholder="myapp.ngrok-free.app" />
          <PmButton size="sm" variant="ghost" @click="handleAddDomain">Add</PmButton>
        </div>
      </div>
      <p v-if="domainError" class="form-error">{{ domainError }}</p>
    </section>
  </div>
</template>

<style scoped>
.ngrok__launch-card {
  background: var(--pm-surface-elevated);
  border: 1px solid color-mix(in srgb, var(--pm-accent) 40%, var(--pm-border));
  border-radius: var(--pm-radius);
  padding: 20px;
  margin-bottom: 24px;
}

.launch-card__content {
  display: flex;
  align-items: center;
  gap: 10px;
}

.ngrok__section {
  margin-bottom: 28px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.section-title {
  font-family: var(--pm-font-display);
  font-size: 14px;
  font-weight: 600;
  color: var(--pm-text-primary);
  margin: 0;
}

.form-error {
  color: var(--pm-danger);
  font-size: 12px;
  font-family: var(--pm-font-body);
  margin: 6px 0 0;
}

.mono-data {
  font-family: var(--pm-font-mono);
  font-size: 12px;
}

.tunnel-url {
  font-family: var(--pm-font-mono);
  font-size: 12px;
  color: var(--pm-accent);
}

.text-muted { color: var(--pm-text-muted); font-size: 13px; }

.status-cell {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.action-btns { display: flex; gap: 4px; }

.domain-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.domain-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: var(--pm-surface);
  border: 1px solid var(--pm-border);
  border-radius: 9999px;
  padding: 4px 10px 4px 12px;
}

.domain-chip__text {
  font-family: var(--pm-font-mono);
  font-size: 12px;
  color: var(--pm-text-primary);
}

.domain-chip__delete {
  background: none;
  border: none;
  color: var(--pm-text-muted);
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
  border-radius: 50%;
  transition: color 0.15s, background 0.15s;
}

.domain-chip__delete:hover {
  color: var(--pm-danger);
  background: var(--pm-badge-error-bg);
}

.domain-add {
  display: flex;
  gap: 6px;
  align-items: center;
  max-width: 320px;
}
</style>
