<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { PmButton, PmInput, PmSelect, PmTable, PmModal, PmBadge } from '@/components/ui'
import { useVault } from '@/composables/useVault'
import type { VaultSecret } from '@/composables/useVault'

const { secrets, loading, error, loadSecrets, addSecret, updateSecret, deleteSecret, activateSecret, deactivateSecret, deactivateAll } = useVault()

const searchQuery = ref('')
const categoryFilter = ref('')

const categoryOptions = [
  { value: '', label: 'All Categories' },
  { value: 'kubeconfig', label: 'Kubeconfig' },
  { value: 'ssh_key', label: 'SSH Key' },
  { value: 'token', label: 'Token' },
  { value: 'certificate', label: 'Certificate' },
  { value: 'password', label: 'Password' },
  { value: 'other', label: 'Other' },
]

const formCategoryOptions = categoryOptions.filter(o => o.value !== '')

const columns = [
  { key: 'name', label: 'Name', sortable: true },
  { key: 'category', label: 'Category', sortable: true, width: '130px' },
  { key: 'file_path', label: 'File Path', sortable: true },
  { key: 'is_active', label: 'Status', sortable: true, width: '100px' },
  { key: 'actions', label: 'Actions', width: '140px' },
]

const filteredSecrets = computed(() => {
  let result = [...secrets.value]
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(s =>
      s.name.toLowerCase().includes(q) ||
      (s.file_path && s.file_path.toLowerCase().includes(q)) ||
      (s.notes && s.notes.toLowerCase().includes(q))
    )
  }
  if (categoryFilter.value) {
    result = result.filter(s => s.category === categoryFilter.value)
  }
  return result
})

// Add/Edit modal state
const showModal = ref(false)
const editingSecret = ref<VaultSecret | null>(null)
const formName = ref('')
const formCategory = ref('other')
const formContent = ref('')
const formFilePath = ref('')
const formNotes = ref('')

// Delete confirmation
const showDeleteConfirm = ref(false)
const deleteTarget = ref<VaultSecret | null>(null)

function categoryLabel(cat: string): string {
  const found = categoryOptions.find(o => o.value === cat)
  return found?.label ?? cat
}

function categoryVariant(cat: string): 'running' | 'info' | 'stopped' | 'error' {
  switch (cat) {
    case 'kubeconfig': return 'info'
    case 'ssh_key': return 'running'
    case 'token': return 'error'
    case 'certificate': return 'running'
    case 'password': return 'error'
    default: return 'stopped'
  }
}

function openAddModal() {
  editingSecret.value = null
  formName.value = ''
  formCategory.value = 'other'
  formContent.value = ''
  formFilePath.value = ''
  formNotes.value = ''
  showModal.value = true
}

function openEditModal(secret: VaultSecret) {
  editingSecret.value = secret
  formName.value = secret.name
  formCategory.value = secret.category
  formContent.value = ''
  formFilePath.value = secret.file_path ?? ''
  formNotes.value = secret.notes ?? ''
  showModal.value = true
}

async function handleSave() {
  if (!formName.value.trim()) return
  if (editingSecret.value) {
    await updateSecret(editingSecret.value.id, {
      name: formName.value,
      category: formCategory.value,
      content: formContent.value || undefined,
      filePath: formFilePath.value || undefined,
      notes: formNotes.value || undefined,
    })
  } else {
    if (!formContent.value.trim()) return
    await addSecret(formName.value, formCategory.value, formContent.value, formFilePath.value || undefined, formNotes.value || undefined)
  }
  showModal.value = false
}

function openDeleteConfirm(secret: VaultSecret) {
  deleteTarget.value = secret
  showDeleteConfirm.value = true
}

async function confirmDelete() {
  if (deleteTarget.value) {
    await deleteSecret(deleteTarget.value.id)
    showDeleteConfirm.value = false
    deleteTarget.value = null
  }
}

async function toggleActive(secret: VaultSecret) {
  if (secret.is_active) {
    await deactivateSecret(secret.id)
  } else {
    await activateSecret(secret.id)
  }
}

onMounted(() => {
  loadSecrets()
})
</script>

<template>
  <div class="vault">
    <!-- Header -->
    <div class="vault__header">
      <div class="vault__filters">
        <PmInput v-model="searchQuery" type="search" placeholder="Search secrets..." />
        <PmSelect v-model="categoryFilter" :options="categoryOptions" placeholder="All Categories" />
      </div>
      <div class="vault__actions">
        <PmButton variant="ghost" @click="deactivateAll">Deactivate All</PmButton>
        <PmButton @click="openAddModal">Add Secret</PmButton>
      </div>
    </div>

    <!-- Error -->
    <p v-if="error" class="vault__error">{{ error }}</p>

    <!-- Table -->
    <section class="vault__list">
      <PmTable :data="filteredSecrets" :columns="columns" :loading="loading">
        <template #cell-name="{ row }">
          <span class="secret-name">{{ row.name }}</span>
        </template>
        <template #cell-category="{ row }">
          <PmBadge :variant="categoryVariant(row.category)">{{ categoryLabel(row.category) }}</PmBadge>
        </template>
        <template #cell-file_path="{ row }">
          <span v-if="row.file_path" class="mono-data">{{ row.file_path }}</span>
          <span v-else class="text-muted">&mdash;</span>
        </template>
        <template #cell-is_active="{ row }">
          <button class="toggle-switch" :class="{ 'toggle-switch--on': row.is_active }" @click="toggleActive(row)" :title="row.is_active ? 'Deactivate' : 'Activate'">
            <span class="toggle-switch__knob" />
          </button>
        </template>
        <template #cell-actions="{ row }">
          <div class="action-btns">
            <PmButton size="sm" variant="ghost" @click="openEditModal(row)">Edit</PmButton>
            <PmButton size="sm" variant="danger" @click="openDeleteConfirm(row)">Delete</PmButton>
          </div>
        </template>
      </PmTable>
    </section>

    <!-- Add/Edit Modal -->
    <PmModal :open="showModal" :title="editingSecret ? 'Edit Secret' : 'Add Secret'" @close="showModal = false">
      <div class="secret-form">
        <label class="form-label">
          Name
          <PmInput v-model="formName" placeholder="e.g., Production Kubeconfig" />
        </label>
        <label class="form-label">
          Category
          <PmSelect v-model="formCategory" :options="formCategoryOptions" />
        </label>
        <label class="form-label">
          Content
          <textarea
            v-model="formContent"
            class="form-textarea"
            :placeholder="editingSecret ? 'Leave empty to keep existing content' : 'Secret content (key, token, config...)'"
            rows="6"
          />
        </label>
        <label class="form-label">
          File Path
          <PmInput v-model="formFilePath" placeholder="e.g., ~/.kube/config (optional)" />
        </label>
        <label class="form-label">
          Notes
          <textarea
            v-model="formNotes"
            class="form-textarea"
            placeholder="Optional notes..."
            rows="3"
          />
        </label>
      </div>
      <template #footer>
        <PmButton variant="ghost" @click="showModal = false">Cancel</PmButton>
        <PmButton @click="handleSave">{{ editingSecret ? 'Update' : 'Add' }}</PmButton>
      </template>
    </PmModal>

    <!-- Delete Confirmation -->
    <PmModal :open="showDeleteConfirm" title="Delete Secret" @close="showDeleteConfirm = false">
      <p class="delete-confirm-text">
        Are you sure you want to delete <strong>{{ deleteTarget?.name }}</strong>? This action cannot be undone.
      </p>
      <template #footer>
        <PmButton variant="ghost" @click="showDeleteConfirm = false">Cancel</PmButton>
        <PmButton variant="danger" @click="confirmDelete">Delete</PmButton>
      </template>
    </PmModal>
  </div>
</template>

<style scoped>
.vault {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.vault__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
}

.vault__filters {
  display: flex;
  gap: 10px;
  flex: 1;
  min-width: 0;
  max-width: 500px;
}

.vault__actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.vault__error {
  color: var(--pm-danger);
  font-size: 13px;
  margin: 0;
  padding: 8px 12px;
  background: var(--pm-badge-error-bg);
  border: 1px solid color-mix(in srgb, var(--pm-danger) 20%, transparent);
  border-radius: var(--pm-radius-sm);
}

.vault__list {
  margin-bottom: 24px;
}

.secret-name {
  font-family: var(--pm-font-body);
  font-weight: 500;
  font-size: 13px;
  color: var(--pm-text-primary);
}

.mono-data {
  font-family: var(--pm-font-mono);
  font-size: 12px;
}

.text-muted {
  color: var(--pm-text-muted);
  font-size: 13px;
}

.action-btns {
  display: flex;
  gap: 4px;
}

/* Toggle Switch */
.toggle-switch {
  position: relative;
  width: 36px;
  height: 20px;
  background: var(--pm-border);
  border: none;
  border-radius: 10px;
  cursor: pointer;
  padding: 0;
  transition: background 0.2s;
}

.toggle-switch--on {
  background: var(--pm-success);
}

.toggle-switch__knob {
  display: block;
  width: 16px;
  height: 16px;
  background: white;
  border-radius: 50%;
  position: absolute;
  top: 2px;
  left: 2px;
  transition: transform 0.2s;
}

.toggle-switch--on .toggle-switch__knob {
  transform: translateX(16px);
}

/* Form */
.secret-form {
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

.form-textarea {
  background: var(--pm-surface-elevated);
  color: var(--pm-text-primary);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  padding: 8px 12px;
  font-size: 12px;
  font-family: var(--pm-font-mono);
  resize: vertical;
  outline: none;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.form-textarea:focus {
  border-color: var(--pm-accent);
  box-shadow: 0 0 0 3px var(--pm-accent-glow);
}

.form-textarea::placeholder {
  color: var(--pm-text-muted);
}

.delete-confirm-text {
  font-family: var(--pm-font-body);
  font-size: 13px;
  color: var(--pm-text-secondary);
  margin: 0;
  line-height: 1.5;
}

.delete-confirm-text strong {
  color: var(--pm-text-primary);
}
</style>
