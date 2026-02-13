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
const showAuthtoken = ref(false)
const showApiKey = ref(false)
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
  try { kubeconfigs.value = await invoke<KubeconfigInfo[]>('list_kubeconfigs') } catch { kubeconfigs.value = [] }
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
  } catch { /* defaults */ }
}

async function loadFromFile() {
  if (!importFilePath.value.trim()) { importError.value = 'Please enter a file path'; return }
  loadingFile.value = true; importError.value = ''
  try {
    const content = await invoke<string>('read_file_content', { path: importFilePath.value.trim() })
    importContent.value = content; importFilePath.value = ''
  } catch (e) { importError.value = `Failed to read file: ${e}` }
  finally { loadingFile.value = false }
}

async function importKubeconfig() {
  importError.value = ''
  if (!importName.value.trim()) { importError.value = 'Please enter a cluster name'; return }
  if (!importContent.value.trim()) { importError.value = 'Please paste or load kubeconfig YAML'; return }
  try {
    await invoke('import_kubeconfig', { name: importName.value.trim(), content: importContent.value })
    importName.value = ''; importContent.value = ''; importError.value = ''; showImportForm.value = false
    await loadKubeconfigs()
  } catch (e) { importError.value = `Failed to import: ${e}` }
}

function cancelImport() {
  importName.value = ''; importContent.value = ''; importFilePath.value = ''; importError.value = ''; showImportForm.value = false
}

async function deleteKubeconfig(id: string) {
  if (confirm('Delete this kubeconfig?')) {
    try { await invoke('delete_kubeconfig', { id }); await loadKubeconfigs() } catch { /* */ }
  }
}

async function savePortRange() {
  try {
    await invoke('set_setting', { key: 'port_range_start', value: portRangeStart.value })
    await invoke('set_setting', { key: 'port_range_end', value: portRangeEnd.value })
  } catch { /* */ }
}

async function saveNgrokAuthtoken() {
  try {
    await invoke('set_setting', { key: 'ngrok_authtoken', value: ngrokAuthtoken.value })
    ngrokSaved.value = true; setTimeout(() => { ngrokSaved.value = false }, 2000)
  } catch { /* */ }
}

async function saveNgrokApiKey() {
  try {
    await invoke('set_setting', { key: 'ngrok_api_key', value: ngrokApiKey.value })
    ngrokApiKeySaved.value = true; setTimeout(() => { ngrokApiKeySaved.value = false }, 2000)
  } catch { /* */ }
}
</script>

<template>
  <div class="settings">
    <!-- Theme -->
    <div class="settings-card">
      <div class="settings-card__header">
        <h2 class="settings-card__title">Theme</h2>
        <p class="settings-card__desc">Choose your preferred visual theme</p>
      </div>
      <PmThemeSwitcher
        :current="currentTheme"
        :themes="themes"
        @change="setTheme($event as Theme)"
      />
    </div>

    <!-- Kubeconfigs -->
    <div class="settings-card">
      <div class="settings-card__header">
        <h2 class="settings-card__title">Kubeconfigs</h2>
        <p class="settings-card__desc">Manage Kubernetes cluster configurations</p>
      </div>

      <div v-if="kubeconfigs.length > 0" class="kubeconfig-list">
        <div v-for="kc in kubeconfigs" :key="kc.id" class="kubeconfig-card">
          <div class="kubeconfig-card__info">
            <span class="kubeconfig-card__name">{{ kc.name }}</span>
            <span class="kubeconfig-card__date">Added {{ kc.created_at }}</span>
          </div>
          <PmButton size="sm" variant="danger" @click="deleteKubeconfig(kc.id)">Delete</PmButton>
        </div>
      </div>
      <p v-else class="empty-text">No kubeconfigs imported yet</p>

      <div v-if="!showImportForm">
        <PmButton variant="ghost" @click="showImportForm = true">+ Import Kubeconfig</PmButton>
      </div>

      <div v-else class="import-form">
        <PmInput v-model="importName" placeholder="Cluster name (e.g., production)" />
        <div class="file-load-row">
          <PmInput v-model="importFilePath" placeholder="File path (e.g., ~/.kube/config)" />
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
        <p v-if="importError" class="form-error">{{ importError }}</p>
        <div class="import-actions">
          <PmButton @click="importKubeconfig">Import</PmButton>
          <PmButton variant="ghost" @click="cancelImport">Cancel</PmButton>
        </div>
      </div>
    </div>

    <!-- Ngrok Credentials -->
    <div class="settings-card">
      <div class="settings-card__header">
        <h2 class="settings-card__title">Ngrok Credentials</h2>
        <p class="settings-card__desc">Authentication for ngrok tunnels and API</p>
      </div>

      <div class="credential-row">
        <label class="credential-label">Authtoken</label>
        <div class="credential-input">
          <PmInput
            v-model="ngrokAuthtoken"
            :type="showAuthtoken ? 'text' : 'password'"
            placeholder="Paste your ngrok authtoken"
          />
          <button class="eye-toggle" @click="showAuthtoken = !showAuthtoken" title="Toggle visibility">
            <svg v-if="!showAuthtoken" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="16" height="16"><path d="M1 8s3-5 7-5 7 5 7 5-3 5-7 5-7-5-7-5z"/><circle cx="8" cy="8" r="2"/></svg>
            <svg v-else viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="16" height="16"><path d="M1 8s3-5 7-5 7 5 7 5-3 5-7 5-7-5-7-5z"/><circle cx="8" cy="8" r="2"/><path d="M2 14L14 2"/></svg>
          </button>
          <PmButton variant="ghost" @click="saveNgrokAuthtoken">
            {{ ngrokSaved ? 'Saved!' : 'Save' }}
          </PmButton>
        </div>
      </div>

      <div class="credential-row">
        <label class="credential-label">API Key</label>
        <div class="credential-input">
          <PmInput
            v-model="ngrokApiKey"
            :type="showApiKey ? 'text' : 'password'"
            placeholder="Paste your ngrok API key"
          />
          <button class="eye-toggle" @click="showApiKey = !showApiKey" title="Toggle visibility">
            <svg v-if="!showApiKey" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="16" height="16"><path d="M1 8s3-5 7-5 7 5 7 5-3 5-7 5-7-5-7-5z"/><circle cx="8" cy="8" r="2"/></svg>
            <svg v-else viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="16" height="16"><path d="M1 8s3-5 7-5 7 5 7 5-3 5-7 5-7-5-7-5z"/><circle cx="8" cy="8" r="2"/><path d="M2 14L14 2"/></svg>
          </button>
          <PmButton variant="ghost" @click="saveNgrokApiKey">
            {{ ngrokApiKeySaved ? 'Saved!' : 'Save' }}
          </PmButton>
        </div>
      </div>
    </div>

    <!-- Port Range -->
    <div class="settings-card">
      <div class="settings-card__header">
        <h2 class="settings-card__title">Port Range</h2>
        <p class="settings-card__desc">Preferred range for auto-assigned local ports</p>
      </div>
      <div class="port-range">
        <PmInput v-model="portRangeStart" type="number" placeholder="Start" />
        <span class="port-range__separator">to</span>
        <PmInput v-model="portRangeEnd" type="number" placeholder="End" />
        <PmButton variant="ghost" @click="savePortRange">Save</PmButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings {
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 700px;
}

.settings-card {
  background: var(--pm-surface);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius);
  padding: 20px;
}

.settings-card__header {
  margin-bottom: 16px;
}

.settings-card__title {
  font-family: var(--pm-font-display);
  font-size: 15px;
  font-weight: 600;
  color: var(--pm-text-primary);
  margin: 0 0 4px;
}

.settings-card__desc {
  font-family: var(--pm-font-body);
  font-size: 13px;
  color: var(--pm-text-muted);
  margin: 0;
}

.kubeconfig-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.kubeconfig-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--pm-surface-elevated);
  border: 1px solid var(--pm-border-subtle);
  border-radius: var(--pm-radius-sm);
  padding: 12px 16px;
}

.kubeconfig-card__name {
  font-family: var(--pm-font-body);
  font-weight: 500;
  font-size: 13px;
  color: var(--pm-text-primary);
  display: block;
}

.kubeconfig-card__date {
  font-family: var(--pm-font-body);
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
  margin-top: 12px;
}

.file-load-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.import-textarea {
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

.import-textarea:focus {
  border-color: var(--pm-accent);
  box-shadow: 0 0 0 3px var(--pm-accent-glow);
}

.import-textarea::placeholder { color: var(--pm-text-muted); }

.form-error {
  color: var(--pm-danger);
  font-size: 12px;
  margin: 0;
}

.import-actions { display: flex; gap: 8px; }

.credential-row {
  margin-bottom: 16px;
}

.credential-row:last-child { margin-bottom: 0; }

.credential-label {
  font-family: var(--pm-font-body);
  font-size: 12px;
  color: var(--pm-text-secondary);
  display: block;
  margin-bottom: 6px;
}

.credential-input {
  display: flex;
  align-items: center;
  gap: 8px;
}

.eye-toggle {
  background: none;
  border: none;
  color: var(--pm-text-muted);
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  transition: color 0.15s;
  flex-shrink: 0;
}

.eye-toggle:hover { color: var(--pm-text-primary); }

.port-range {
  display: flex;
  align-items: center;
  gap: 8px;
  max-width: 400px;
}

.port-range__separator {
  color: var(--pm-text-muted);
  font-family: var(--pm-font-body);
  font-size: 13px;
}
</style>
