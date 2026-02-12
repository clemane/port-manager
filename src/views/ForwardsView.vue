<script setup lang="ts">
import { ref } from 'vue'
import { PmButton, PmBadge, PmTable, PmModal, PmInput } from '@/components/ui'
import { useForwards } from '@/composables/useForwards'
import type { ActiveForward } from '@/composables/useForwards'

const { forwards, favorites, killForward, restartForward, launchFavorite, deleteFavorite, saveFavorite } = useForwards()

// Save favorite modal
const showSaveModal = ref(false)
const saveTarget = ref<ActiveForward | null>(null)
const favoriteLabel = ref('')
const favoriteGroup = ref('')

const columns = [
  { key: 'resource_name', label: 'Resource', sortable: true },
  { key: 'namespace', label: 'Namespace', sortable: true },
  { key: 'local_port', label: 'Local Port', sortable: true, width: '100px' },
  { key: 'remote_port', label: 'Remote Port', sortable: true, width: '100px' },
  { key: 'status', label: 'Status', sortable: true, width: '100px' },
  { key: 'actions', label: 'Actions', width: '180px' },
]

function statusVariant(status: string): 'running' | 'error' | 'stopped' {
  if (status === 'running') return 'running'
  if (status === 'error') return 'error'
  return 'stopped'
}

function copyUrl(port: number) {
  navigator.clipboard.writeText(`localhost:${port}`)
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

// Group favorites by group_name
function groupedFavorites() {
  const groups = new Map<string, typeof favorites.value>()
  for (const fav of favorites.value) {
    const group = fav.group_name || 'Ungrouped'
    if (!groups.has(group)) groups.set(group, [])
    groups.get(group)!.push(fav)
  }
  return groups
}
</script>

<template>
  <div class="forwards">
    <div class="forwards__header">
      <h1 class="view-title">Port Forwards</h1>
      <p class="view-subtitle">Manage active forwards and favorites</p>
    </div>

    <!-- Favorites Section -->
    <section v-if="favorites.length > 0" class="forwards__favorites">
      <h2 class="section-title">Favorites</h2>
      <div v-for="[group, favs] in groupedFavorites()" :key="group" class="favorite-group">
        <h3 class="group-title">{{ group }}</h3>
        <div class="favorite-cards">
          <div v-for="fav in favs" :key="fav.id" class="favorite-card">
            <div class="favorite-card__info">
              <span class="favorite-card__label">{{ fav.label }}</span>
              <span class="favorite-card__detail">
                {{ fav.resource_type }}/{{ fav.resource_name }} : {{ fav.remote_port }}
              </span>
            </div>
            <div class="favorite-card__actions">
              <PmButton size="sm" @click="launchFavorite(fav)">Launch</PmButton>
              <PmButton size="sm" variant="danger" @click="deleteFavorite(fav.id)">Delete</PmButton>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Active Forwards Table -->
    <section class="forwards__active">
      <h2 class="section-title">Active Forwards</h2>
      <PmTable :data="forwards" :columns="columns">
        <template #cell-resource_name="{ row }">
          <span class="resource-name">{{ row.resource_type }}/{{ row.resource_name }}</span>
        </template>
        <template #cell-local_port="{ value }">
          <span class="port-mono">{{ value }}</span>
        </template>
        <template #cell-remote_port="{ value }">
          <span class="port-mono">{{ value }}</span>
        </template>
        <template #cell-status="{ value }">
          <PmBadge :variant="statusVariant(value)">{{ value }}</PmBadge>
        </template>
        <template #cell-actions="{ row }">
          <div class="action-btns">
            <PmButton v-if="row.status === 'running'" size="sm" variant="danger" @click="killForward(row.id)">Kill</PmButton>
            <PmButton v-if="row.status !== 'running'" size="sm" variant="ghost" @click="restartForward(row.id)">Restart</PmButton>
            <PmButton size="sm" variant="icon" @click="copyUrl(row.local_port)" title="Copy URL">📋</PmButton>
            <PmButton size="sm" variant="icon" @click="openSaveModal(row)" title="Save as favorite">⭐</PmButton>
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
  </div>
</template>

<style scoped>
.forwards__header { margin-bottom: 20px; }
.view-title { font-size: 20px; font-weight: 600; color: var(--pm-text-primary); margin: 0 0 4px; }
.view-subtitle { font-size: 13px; color: var(--pm-text-secondary); margin: 0; }

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--pm-text-primary);
  margin: 0 0 12px;
}

.forwards__favorites { margin-bottom: 32px; }
.group-title {
  font-size: 12px;
  color: var(--pm-text-muted);
  text-transform: uppercase;
  margin: 0 0 8px;
}
.favorite-cards { display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 16px; }
.favorite-card {
  background: var(--pm-surface);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius);
  padding: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  min-width: 280px;
}
.favorite-card__label { font-weight: 500; font-size: 13px; color: var(--pm-text-primary); display: block; }
.favorite-card__detail { font-size: 11px; color: var(--pm-text-muted); font-family: monospace; }
.favorite-card__actions { display: flex; gap: 4px; }

.resource-name { font-family: monospace; font-size: 13px; }
.port-mono { font-family: monospace; }
.action-btns { display: flex; gap: 4px; }

.save-form { display: flex; flex-direction: column; gap: 12px; }
.form-label {
  font-size: 13px;
  color: var(--pm-text-secondary);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
</style>
