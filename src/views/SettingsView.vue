<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { PmButton, PmInput, PmThemeSwitcher } from '@/components/ui'
import { useTheme, type Theme } from '@/composables/useTheme'

const { currentTheme, setTheme } = useTheme()

interface KubeconfigInfo {
  id: string
  name: string
  created_at: string
  last_used: string | null
}

const kubeconfigs = ref<KubeconfigInfo[]>([])
const portRangeStart = ref('3000')
const portRangeEnd = ref('4000')
const ngrokAuthtoken = ref('')
const ngrokApiKey = ref('')
const ngrokSaved = ref(false)
const ngrokApiKeySaved = ref(false)
const importName = ref('')
const importContent = ref('')
const importFilePath = ref('')
const showImportForm = ref(false)
const importError = ref('')
const loadingFile = ref(false)

const themes: { value: string; label: string }[] = [
  { value: 'dark', label: 'Dark' },
  { value: 'light', label: 'Light' },
  { value: 'cyberpunk', label: 'Cyber' },
  { value: 'matrix', label: 'Matrix' },
]

onMounted(async () => {
  await loadKubeconfigs()
  await loadSettings()
})

async function loadKubeconfigs() {
  try {
    kubeconfigs.value = await invoke<KubeconfigInfo[]>('list_kubeconfigs')
  } catch {
    kubeconfigs.value = []
  }
}

async function loadSettings() {
  try {
    const start = await invoke<string | null>('get_setting', { key: 'port_range_start' })
    const end = await invoke<string | null>('get_setting', { key: 'port_range_end' })
    const token = await invoke<string | null>('get_setting', { key: 'ngrok_authtoken' })
    const apiKey = await invoke<string | null>('get_setting', { key: 'ngrok_api_key' })
    if (start) portRangeStart.value = start
    if (end) portRangeEnd.value = end
    if (token) ngrokAuthtoken.value = token
    if (apiKey) ngrokApiKey.value = apiKey
  } catch {
    // Use defaults
  }
}

async function loadFromFile() {
  if (!importFilePath.value.trim()) {
    importError.value = 'Please enter a file path'
    return
  }
  loadingFile.value = true
  importError.value = ''
  try {
    const content = await invoke<string>('read_file_content', { path: importFilePath.value.trim() })
    importContent.value = content
    importFilePath.value = ''
  } catch (e) {
    importError.value = `Failed to read file: ${e}`
  } finally {
    loadingFile.value = false
  }
}

async function importKubeconfig() {
  importError.value = ''

  if (!importName.value.trim()) {
    importError.value = 'Please enter a cluster name'
    return
  }
  if (!importContent.value.trim()) {
    importError.value = 'Please paste or load your kubeconfig YAML content'
    return
  }

  try {
    await invoke('import_kubeconfig', {
      name: importName.value.trim(),
      content: importContent.value,
    })
    importName.value = ''
    importContent.value = ''
    importError.value = ''
    showImportForm.value = false
    await loadKubeconfigs()
  } catch (e) {
    importError.value = `Failed to import: ${e}`
  }
}

function cancelImport() {
  importName.value = ''
  importContent.value = ''
  importFilePath.value = ''
  importError.value = ''
  showImportForm.value = false
}

async function deleteKubeconfig(id: string) {
  if (confirm('Delete this kubeconfig?')) {
    try {
      await invoke('delete_kubeconfig', { id })
      await loadKubeconfigs()
    } catch {
      // Silently fail
    }
  }
}

async function savePortRange() {
  try {
    await invoke('set_setting', { key: 'port_range_start', value: portRangeStart.value })
    await invoke('set_setting', { key: 'port_range_end', value: portRangeEnd.value })
  } catch {
    // Silently fail
  }
}

async function saveNgrokAuthtoken() {
  try {
    await invoke('set_setting', { key: 'ngrok_authtoken', value: ngrokAuthtoken.value })
    ngrokSaved.value = true
    setTimeout(() => { ngrokSaved.value = false }, 2000)
  } catch {
    // Silently fail
  }
}

async function saveNgrokApiKey() {
  try {
    await invoke('set_setting', { key: 'ngrok_api_key', value: ngrokApiKey.value })
    ngrokApiKeySaved.value = true
    setTimeout(() => { ngrokApiKeySaved.value = false }, 2000)
  } catch {
    // Silently fail
  }
}
</script>

<template>
  <div class="settings">
    <div class="settings__header">
      <h1 class="view-title">Settings</h1>
      <p class="view-subtitle">Kubeconfigs, themes, and preferences</p>
    </div>

    <!-- Theme -->
    <section class="settings__section">
      <h2 class="section-title">Theme</h2>
      <PmThemeSwitcher
        :current="currentTheme"
        :themes="themes"
        @change="setTheme($event as Theme)"
      />
    </section>

    <!-- Kubeconfigs -->
    <section class="settings__section">
      <h2 class="section-title">Kubeconfigs</h2>

      <div v-if="kubeconfigs.length > 0" class="kubeconfig-list">
        <div v-for="kc in kubeconfigs" :key="kc.id" class="kubeconfig-item">
          <div class="kubeconfig-info">
            <span class="kubeconfig-name">{{ kc.name }}</span>
            <span class="kubeconfig-date">Added {{ kc.created_at }}</span>
          </div>
          <PmButton size="sm" variant="danger" @click="deleteKubeconfig(kc.id)">
            Delete
          </PmButton>
        </div>
      </div>
      <p v-else class="empty-text">No kubeconfigs imported yet</p>

      <div v-if="!showImportForm" class="import-trigger">
        <PmButton variant="ghost" @click="showImportForm = true">+ Import Kubeconfig</PmButton>
      </div>

      <div v-else class="import-form">
        <PmInput v-model="importName" placeholder="Cluster name (e.g., production)" />
        <div class="file-load-row">
          <PmInput
            v-model="importFilePath"
            placeholder="File path (e.g., ~/.kube/config)"
            class="file-load-input"
          />
          <PmButton variant="ghost" :disabled="loadingFile" @click="loadFromFile">
            {{ loadingFile ? 'Loading...' : 'Load file' }}
          </PmButton>
        </div>
        <textarea
          v-model="importContent"
          class="import-textarea"
          placeholder="Paste your kubeconfig YAML content or load from file above..."
          rows="8"
        />
        <p v-if="importError" class="import-error">{{ importError }}</p>
        <div class="import-actions">
          <PmButton @click="importKubeconfig">Import</PmButton>
          <PmButton variant="ghost" @click="cancelImport">Cancel</PmButton>
        </div>
      </div>
    </section>

    <!-- Ngrok Authtoken -->
    <section class="settings__section">
      <h2 class="section-title">Ngrok Authtoken</h2>
      <p class="section-desc">Your ngrok authentication token for launching tunnels</p>
      <div class="ngrok-token-row">
        <PmInput v-model="ngrokAuthtoken" type="password" placeholder="Paste your ngrok authtoken" />
        <PmButton variant="ghost" @click="saveNgrokAuthtoken">
          {{ ngrokSaved ? 'Saved!' : 'Save' }}
        </PmButton>
      </div>
      <p class="section-desc" style="margin-top: 16px;">API key for syncing reserved domains from your ngrok account</p>
      <div class="ngrok-token-row">
        <PmInput v-model="ngrokApiKey" type="password" placeholder="Paste your ngrok API key" />
        <PmButton variant="ghost" @click="saveNgrokApiKey">
          {{ ngrokApiKeySaved ? 'Saved!' : 'Save' }}
        </PmButton>
      </div>
    </section>

    <!-- Port Range -->
    <section class="settings__section">
      <h2 class="section-title">Port Range</h2>
      <p class="section-desc">Preferred port range for auto-assigned local ports</p>
      <div class="port-range">
        <PmInput v-model="portRangeStart" type="number" placeholder="Start" />
        <span class="port-range__separator">to</span>
        <PmInput v-model="portRangeEnd" type="number" placeholder="End" />
        <PmButton variant="ghost" @click="savePortRange">Save</PmButton>
      </div>
    </section>
  </div>
</template>

<style scoped>
.settings__header {
  margin-bottom: 24px;
}

.view-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--pm-text-primary);
  margin: 0 0 4px;
}

.view-subtitle {
  font-size: 13px;
  color: var(--pm-text-secondary);
  margin: 0;
}

.settings__section {
  margin-bottom: 32px;
  padding-bottom: 24px;
  border-bottom: 1px solid var(--pm-border-subtle);
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--pm-text-primary);
  margin: 0 0 12px;
}

.section-desc {
  font-size: 13px;
  color: var(--pm-text-muted);
  margin: 0 0 12px;
}

.kubeconfig-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.kubeconfig-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--pm-surface);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius);
  padding: 12px;
}

.kubeconfig-name {
  font-weight: 500;
  font-size: 13px;
  color: var(--pm-text-primary);
  display: block;
}

.kubeconfig-date {
  font-size: 11px;
  color: var(--pm-text-muted);
}

.empty-text {
  color: var(--pm-text-muted);
  font-size: 13px;
  margin-bottom: 12px;
}

.import-form {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 500px;
}

.file-load-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.file-load-input {
  flex: 1;
}

.import-textarea {
  background: var(--pm-surface);
  color: var(--pm-text-primary);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  padding: 8px 12px;
  font-size: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  resize: vertical;
  outline: none;
  transition: border-color 0.15s;
}

.import-textarea:focus {
  border-color: var(--pm-accent);
}

.import-textarea::placeholder {
  color: var(--pm-text-muted);
}

.import-error {
  color: var(--pm-danger);
  font-size: 12px;
  margin: 0;
}

.import-actions {
  display: flex;
  gap: 8px;
}

.ngrok-token-row {
  display: flex;
  align-items: center;
  gap: 8px;
  max-width: 500px;
}

.port-range {
  display: flex;
  align-items: center;
  gap: 8px;
  max-width: 400px;
}

.port-range__separator {
  color: var(--pm-text-muted);
  font-size: 13px;
}
</style>
