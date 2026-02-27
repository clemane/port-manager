import { ref, readonly } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface VaultSecret {
  id: string
  name: string
  category: 'kubeconfig' | 'ssh_key' | 'token' | 'certificate' | 'password' | 'other'
  file_path: string | null
  notes: string | null
  is_active: boolean
  created_at: string
  updated_at: string
}

const secrets = ref<VaultSecret[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

async function loadSecrets() {
  loading.value = true
  error.value = null
  try {
    secrets.value = await invoke<VaultSecret[]>('list_vault_secrets')
  } catch (e) {
    error.value = `Failed to load secrets: ${e}`
  } finally {
    loading.value = false
  }
}

async function addSecret(name: string, category: string, content: string, filePath?: string, notes?: string) {
  error.value = null
  try {
    await invoke('add_vault_secret', {
      name,
      category,
      content: Array.from(new TextEncoder().encode(content)),
      filePath: filePath ?? null,
      notes: notes ?? null,
    })
    await loadSecrets()
  } catch (e) {
    error.value = `Failed to add secret: ${e}`
  }
}

async function updateSecret(id: string, updates: { name?: string; category?: string; content?: string; filePath?: string; notes?: string }) {
  error.value = null
  try {
    await invoke('update_vault_secret', {
      id,
      name: updates.name ?? null,
      category: updates.category ?? null,
      content: updates.content ? Array.from(new TextEncoder().encode(updates.content)) : null,
      filePath: updates.filePath ?? null,
      notes: updates.notes ?? null,
    })
    await loadSecrets()
  } catch (e) {
    error.value = `Failed to update secret: ${e}`
  }
}

async function deleteSecret(id: string) {
  error.value = null
  try {
    await invoke('delete_vault_secret', { id })
    await loadSecrets()
  } catch (e) {
    error.value = `Failed to delete secret: ${e}`
  }
}

async function activateSecret(id: string) {
  error.value = null
  try {
    await invoke('activate_secret', { id })
    await loadSecrets()
  } catch (e) {
    error.value = `Failed to activate secret: ${e}`
  }
}

async function deactivateSecret(id: string) {
  error.value = null
  try {
    await invoke('deactivate_secret', { id })
    await loadSecrets()
  } catch (e) {
    error.value = `Failed to deactivate secret: ${e}`
  }
}

async function deactivateAll() {
  error.value = null
  try {
    await invoke('deactivate_all_secrets')
    await loadSecrets()
  } catch (e) {
    error.value = `Failed to deactivate secrets: ${e}`
  }
}

export function useVault() {
  return {
    secrets: readonly(secrets),
    loading: readonly(loading),
    error: readonly(error),
    loadSecrets,
    addSecret,
    updateSecret,
    deleteSecret,
    activateSecret,
    deactivateSecret,
    deactivateAll,
  }
}
