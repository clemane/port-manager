<script setup lang="ts">
import { ref, computed } from 'vue'
import { PmButton, PmBadge, PmTable, PmModal, PmInput, PmMetricCard, PmStatusDot, PmConnectionModal, PmCredentialPicker } from '@/components/ui'
import type { ConnectionFormData, ConnectionInitialValues } from '@/components/ui'
import type { DetectedCredentials } from '@/types/k8s'
import { useForwards } from '@/composables/useForwards'
import type { ActiveForward } from '@/composables/useForwards'
import { useDbDetection } from '@/composables/useDbDetection'
import { usePgManager } from '@/composables/usePgManager'
import { useRouter } from 'vue-router'

const { forwards, favorites, killForward, restartForward, launchFavorite, deleteFavorite, saveFavorite } = useForwards()

const router = useRouter()
const { detecting, detectedCreds, detectError, detectCredentials, reset: resetDetection } = useDbDetection()
const { saveConnection } = usePgManager()

const showSaveModal = ref(false)
const saveTarget = ref<ActiveForward | null>(null)
const favoriteLabel = ref('')
const favoriteGroup = ref('')
const copiedId = ref<string | null>(null)

const showCredentialPicker = ref(false)
const showConnectionModal = ref(false)
const connectionInitialValues = ref<ConnectionInitialValues | null>(null)
const dbTargetForward = ref<ActiveForward | null>(null)

const columns = [
  { key: 'resource_name', label: 'Resource', sortable: true },
  { key: 'namespace', label: 'Namespace', sortable: true },
  { key: 'local_port', label: 'Local Port', sortable: true, width: '100px' },
  { key: 'remote_port', label: 'Remote Port', sortable: true, width: '100px' },
  { key: 'status', label: 'Status', sortable: true, width: '110px' },
  { key: 'actions', label: 'Actions', width: '200px' },
]

const activeCount = computed(() => forwards.value.filter(f => f.status === 'running').length)
const errorCount = computed(() => forwards.value.filter(f => f.status === 'error').length)

function statusVariant(status: string): 'running' | 'error' | 'stopped' {
  if (status === 'running') return 'running'
  if (status === 'error') return 'error'
  return 'stopped'
}

function copyUrl(id: string, port: number) {
  navigator.clipboard.writeText(`localhost:${port}`)
  copiedId.value = id
  setTimeout(() => { copiedId.value = null }, 1500)
}

function openSaveModal(forward: ActiveForward) {
  saveTarget.value = forward
  favoriteLabel.value = `${forward.resource_name}:${forward.remote_port}`
  favoriteGroup.value = forward.namespace
  showSaveModal.value = true
}

async function confirmSave() {
  if (saveTarget.value && favoriteLabel.value) {
    await saveFavorite(saveTarget.value, favoriteLabel.value, favoriteGroup.value || undefined)
    showSaveModal.value = false
  }
}

function groupedFavorites() {
  const groups = new Map<string, typeof favorites.value>()
  for (const fav of favorites.value) {
    const group = fav.group_name || 'Ungrouped'
    if (!groups.has(group)) groups.set(group, [])
    groups.get(group)!.push(fav)
  }
  return groups
}

async function openDbManager(forward: ActiveForward) {
  dbTargetForward.value = forward
  connectionInitialValues.value = {
    host: '127.0.0.1',
    port: forward.local_port,
    forwardId: forward.id,
  }
  await detectCredentials(forward.kubeconfig_id, forward.namespace)
  if (detectedCreds.value.length > 1) {
    showCredentialPicker.value = true
  } else if (detectedCreds.value.length === 1) {
    applyCredential(detectedCreds.value[0])
  } else {
    showConnectionModal.value = true
  }
}

function applyCredential(cred: DetectedCredentials) {
  showCredentialPicker.value = false
  const forward = dbTargetForward.value
  connectionInitialValues.value = {
    host: cred.host ?? '127.0.0.1',
    port: cred.port ?? forward?.local_port ?? 5432,
    databaseName: cred.database ?? undefined,
    username: cred.username ?? undefined,
    password: cred.password ?? undefined,
    sslMode: cred.ssl_mode ?? undefined,
    label: cred.source,
    forwardId: forward?.id ?? undefined,
  }
  showConnectionModal.value = true
}

function onCredentialPickerManual() {
  showCredentialPicker.value = false
  showConnectionModal.value = true
}

async function onConnectionSave(data: ConnectionFormData) {
  try {
    await saveConnection({
      label: data.label || undefined,
      forwardId: data.forwardId || undefined,
      favoriteId: data.favoriteId || undefined,
      host: data.host,
      port: data.port,
      databaseName: data.databaseName,
      username: data.username,
      password: data.password || undefined,
      sslMode: data.sslMode,
      color: data.color || undefined,
    })
    showConnectionModal.value = false
    resetDetection()
    router.push('/database')
  } catch (e) {
    alert(String(e))
  }
}
</script>

<template>
  <div class="forwards">
    <div class="forwards__metrics">
      <PmMetricCard label="Active" :value="activeCount" color="success" />
      <PmMetricCard label="Errored" :value="errorCount" color="danger" />
      <PmMetricCard label="Favorites" :value="favorites.length" color="accent" />
    </div>

    <!-- Favorites -->
    <section v-if="favorites.length > 0" class="forwards__favorites">
      <h2 class="section-title">Favorites</h2>
      <div v-for="[group, favs] in groupedFavorites()" :key="group" class="favorite-group">
        <h3 class="group-title">{{ group }}</h3>
        <div class="favorite-grid">
          <div v-for="fav in favs" :key="fav.id" class="favorite-card">
            <div class="favorite-card__info">
              <span class="favorite-card__label">{{ fav.label }}</span>
              <span class="favorite-card__detail">
                {{ fav.resource_type }}/{{ fav.resource_name }}:{{ fav.remote_port }}
              </span>
            </div>
            <div class="favorite-card__actions">
              <PmButton size="sm" @click="launchFavorite(fav)">Launch</PmButton>
              <PmButton size="sm" variant="icon" @click="deleteFavorite(fav.id)" title="Delete">
                <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="14" height="14"><path d="M4 4l8 8M12 4l-8 8" stroke-linecap="round"/></svg>
              </PmButton>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Active Forwards -->
    <section class="forwards__active">
      <h2 class="section-title">Active Forwards</h2>
      <PmTable :data="forwards" :columns="columns">
        <template #cell-resource_name="{ row }">
          <span class="mono-data">{{ row.resource_type }}/{{ row.resource_name }}</span>
        </template>
        <template #cell-local_port="{ value }">
          <span class="mono-data">{{ value }}</span>
        </template>
        <template #cell-remote_port="{ value }">
          <span class="mono-data">{{ value }}</span>
        </template>
        <template #cell-status="{ row }">
          <span class="status-cell">
            <PmStatusDot :status="statusVariant(row.status)" />
            <span>{{ row.status }}</span>
          </span>
        </template>
        <template #cell-actions="{ row }">
          <div class="action-btns">
            <PmButton v-if="row.status === 'running'" size="sm" variant="danger" @click="killForward(row.id)">Kill</PmButton>
            <PmButton v-else size="sm" variant="ghost" @click="restartForward(row.id)">Restart</PmButton>
            <PmButton v-if="row.status === 'running'" size="sm" variant="ghost" @click="openDbManager(row)" title="Open Database Manager">
              <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="14" height="14">
                <ellipse cx="8" cy="4" rx="6" ry="2.5"/>
                <path d="M2 4v8c0 1.38 2.69 2.5 6 2.5s6-1.12 6-2.5V4"/>
                <path d="M2 8c0 1.38 2.69 2.5 6 2.5s6-1.12 6-2.5"/>
              </svg>
            </PmButton>
            <PmButton size="sm" variant="icon" @click="copyUrl(row.id, row.local_port)" :title="copiedId === row.id ? 'Copied!' : 'Copy URL'">
              <svg v-if="copiedId !== row.id" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="14" height="14"><rect x="5" y="5" width="8" height="8" rx="1"/><path d="M3 11V3h8" stroke-linecap="round"/></svg>
              <svg v-else viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="14" height="14"><path d="M4 8l3 3 5-5" stroke-linecap="round" stroke-linejoin="round"/></svg>
            </PmButton>
            <PmButton size="sm" variant="icon" @click="openSaveModal(row)" title="Save as favorite">
              <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="14" height="14"><path d="M8 2l2 4 4.5.7-3.2 3.1.8 4.5L8 12l-4.1 2.3.8-4.5L1.5 6.7 6 6z" stroke-linejoin="round"/></svg>
            </PmButton>
          </div>
        </template>
      </PmTable>
    </section>

    <!-- Save Favorite Modal -->
    <PmModal :open="showSaveModal" title="Save as Favorite" @close="showSaveModal = false">
      <div class="save-form">
        <label class="form-label">
          Label
          <PmInput v-model="favoriteLabel" placeholder="e.g., API Staging" />
        </label>
        <label class="form-label">
          Group
          <PmInput v-model="favoriteGroup" placeholder="e.g., production" />
        </label>
      </div>
      <template #footer>
        <PmButton variant="ghost" @click="showSaveModal = false">Cancel</PmButton>
        <PmButton @click="confirmSave">Save</PmButton>
      </template>
    </PmModal>

    <PmCredentialPicker
      :open="showCredentialPicker"
      :credentials="detectedCreds"
      :loading="detecting"
      :error="detectError"
      @close="showCredentialPicker = false"
      @select="applyCredential"
      @manual="onCredentialPickerManual"
    />

    <PmConnectionModal
      :open="showConnectionModal"
      :connection="null"
      :forwards="[]"
      :favorites="[]"
      :initial-values="connectionInitialValues"
      @close="showConnectionModal = false"
      @save="onConnectionSave"
    />
  </div>
</template>

<style scoped>
.forwards__metrics {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.section-title {
  font-family: var(--pm-font-display);
  font-size: 14px;
  font-weight: 600;
  color: var(--pm-text-primary);
  margin: 0 0 12px;
}

.forwards__favorites {
  margin-bottom: 32px;
}

.group-title {
  font-family: var(--pm-font-body);
  font-size: 11px;
  color: var(--pm-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0 0 8px;
}

.favorite-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 10px;
  margin-bottom: 16px;
}

.favorite-card {
  background: var(--pm-surface);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius);
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  transition: border-color 0.15s;
}

.favorite-card:hover {
  border-color: var(--pm-accent);
}

.favorite-card__label {
  font-family: var(--pm-font-body);
  font-weight: 500;
  font-size: 13px;
  color: var(--pm-text-primary);
  display: block;
}

.favorite-card__detail {
  font-family: var(--pm-font-mono);
  font-size: 11px;
  color: var(--pm-text-muted);
}

.favorite-card__actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.forwards__active {
  margin-bottom: 24px;
}

.mono-data {
  font-family: var(--pm-font-mono);
  font-size: 12px;
}

.status-cell {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.action-btns {
  display: flex;
  gap: 4px;
}

.save-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-label {
  font-family: var(--pm-font-body);
  font-size: 13px;
  color: var(--pm-text-secondary);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
</style>
