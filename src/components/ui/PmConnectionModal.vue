<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import PmModal from './PmModal.vue'
import PmInput from './PmInput.vue'
import PmSelect from './PmSelect.vue'
import PmButton from './PmButton.vue'
import type { PgConnection } from '@/types/pgmanager'

export interface ConnectionFormData {
  id?: string
  label: string
  forwardId: string | null
  favoriteId: string | null
  host: string
  port: number
  databaseName: string
  username: string
  password: string
  sslMode: string
  color: string | null
}

export interface ConnectionInitialValues {
  label?: string
  host?: string
  port?: number
  databaseName?: string
  username?: string
  password?: string
  sslMode?: string
  forwardId?: string
  favoriteId?: string
}

const props = defineProps<{
  open: boolean
  connection: PgConnection | null
  forwards: { id: string; label: string; localPort: number }[]
  favorites: { id: string; label: string }[]
  testResult?: string | null
  testLoading?: boolean
  initialValues?: ConnectionInitialValues | null
}>()

const emit = defineEmits<{
  close: []
  save: [data: ConnectionFormData]
  test: [data: ConnectionFormData]
}>()

const label = ref('')
const host = ref('127.0.0.1')
const port = ref<string | number>(5432)
const databaseName = ref('postgres')
const username = ref('postgres')
const password = ref('')
const sslMode = ref('prefer')
const forwardId = ref('')
const favoriteId = ref('')
const color = ref<string | null>(null)

const sslOptions = [
  { value: 'disable', label: 'Disable' },
  { value: 'allow', label: 'Allow' },
  { value: 'prefer', label: 'Prefer' },
  { value: 'require', label: 'Require' },
  { value: 'verify-ca', label: 'Verify CA' },
  { value: 'verify-full', label: 'Verify Full' },
]

const forwardOptions = computed(() => [
  { value: '', label: 'None (direct connection)' },
  ...props.forwards.map(f => ({
    value: f.id,
    label: `${f.label} (:${f.localPort})`,
  })),
])

const favoriteOptions = computed(() => [
  { value: '', label: 'None' },
  ...props.favorites.map(f => ({
    value: f.id,
    label: f.label,
  })),
])

const isEditing = computed(() => props.connection !== null)

const modalTitle = computed(() =>
  isEditing.value ? 'Edit Connection' : 'New Connection'
)

const testResultType = computed<'success' | 'error' | null>(() => {
  if (!props.testResult) return null
  const lower = props.testResult.toLowerCase()
  if (lower.includes('success') || lower.includes('ok') || lower.includes('connected')) {
    return 'success'
  }
  return 'error'
})

function resetForm() {
  label.value = ''
  host.value = '127.0.0.1'
  port.value = 5432
  databaseName.value = 'postgres'
  username.value = 'postgres'
  password.value = ''
  sslMode.value = 'prefer'
  forwardId.value = ''
  favoriteId.value = ''
  color.value = null
}

function populateFromConnection(conn: PgConnection) {
  label.value = conn.label ?? ''
  host.value = conn.host
  port.value = conn.port
  databaseName.value = conn.database_name
  username.value = conn.username
  password.value = ''
  sslMode.value = conn.ssl_mode
  forwardId.value = conn.forward_id ?? ''
  favoriteId.value = conn.favorite_id ?? ''
  color.value = conn.color
}

// Watch open to reset or populate form
watch(() => props.open, (open) => {
  if (open) {
    if (props.connection) {
      populateFromConnection(props.connection)
    } else {
      resetForm()
      if (props.initialValues) {
        if (props.initialValues.label !== undefined) label.value = props.initialValues.label
        if (props.initialValues.host !== undefined) host.value = props.initialValues.host
        if (props.initialValues.port !== undefined) port.value = props.initialValues.port
        if (props.initialValues.databaseName !== undefined) databaseName.value = props.initialValues.databaseName
        if (props.initialValues.username !== undefined) username.value = props.initialValues.username
        if (props.initialValues.password !== undefined) password.value = props.initialValues.password
        if (props.initialValues.sslMode !== undefined) sslMode.value = props.initialValues.sslMode
        if (props.initialValues.forwardId !== undefined) forwardId.value = props.initialValues.forwardId
        if (props.initialValues.favoriteId !== undefined) favoriteId.value = props.initialValues.favoriteId
      }
    }
  }
})

// When forward is selected, auto-fill host and port
watch(forwardId, (id) => {
  if (!id) return
  const forward = props.forwards.find(f => f.id === id)
  if (forward) {
    host.value = '127.0.0.1'
    port.value = forward.localPort
  }
})

function buildFormData(): ConnectionFormData {
  return {
    ...(isEditing.value && props.connection ? { id: props.connection.id } : {}),
    label: label.value,
    forwardId: forwardId.value || null,
    favoriteId: favoriteId.value || null,
    host: host.value,
    port: Number(port.value),
    databaseName: databaseName.value,
    username: username.value,
    password: password.value,
    sslMode: sslMode.value,
    color: color.value,
  }
}

function onSave() {
  emit('save', buildFormData())
}

function onTest() {
  emit('test', buildFormData())
}
</script>

<template>
  <PmModal :open="open" :title="modalTitle" @close="$emit('close')">
    <div class="pm-conn-form">
      <!-- Label -->
      <div class="pm-conn-form__group">
        <label class="pm-conn-form__label">Label</label>
        <PmInput v-model="label" placeholder="My Database" />
      </div>

      <!-- Forward link -->
      <div class="pm-conn-form__group">
        <label class="pm-conn-form__label">Link to Forward</label>
        <PmSelect
          v-model="forwardId"
          :options="forwardOptions"
          placeholder="None (direct connection)"
        />
        <span class="pm-conn-form__hint">
          Selecting a forward auto-fills host and port
        </span>
      </div>

      <!-- Favorite link -->
      <div class="pm-conn-form__group">
        <label class="pm-conn-form__label">Link to Favorite</label>
        <PmSelect
          v-model="favoriteId"
          :options="favoriteOptions"
          placeholder="None"
        />
      </div>

      <!-- Host + Port row -->
      <div class="pm-conn-form__row">
        <div class="pm-conn-form__group pm-conn-form__group--grow">
          <label class="pm-conn-form__label">Host</label>
          <PmInput v-model="host" placeholder="127.0.0.1" />
        </div>
        <div class="pm-conn-form__group pm-conn-form__group--port">
          <label class="pm-conn-form__label">Port</label>
          <PmInput v-model="port" type="number" placeholder="5432" />
        </div>
      </div>

      <!-- Database name -->
      <div class="pm-conn-form__group">
        <label class="pm-conn-form__label">Database</label>
        <PmInput v-model="databaseName" placeholder="postgres" />
      </div>

      <!-- Username -->
      <div class="pm-conn-form__group">
        <label class="pm-conn-form__label">Username</label>
        <PmInput v-model="username" placeholder="postgres" />
      </div>

      <!-- Password -->
      <div class="pm-conn-form__group">
        <label class="pm-conn-form__label">Password</label>
        <PmInput v-model="password" type="password" placeholder="Enter password" />
      </div>

      <!-- SSL Mode -->
      <div class="pm-conn-form__group">
        <label class="pm-conn-form__label">SSL Mode</label>
        <PmSelect
          v-model="sslMode"
          :options="sslOptions"
          placeholder="Select SSL mode"
        />
      </div>

      <!-- Test result feedback -->
      <div v-if="testLoading || testResult" class="pm-conn-form__test-result">
        <span v-if="testLoading" class="pm-conn-form__test-loading">
          <span class="pm-conn-form__spinner" />
          Testing connection...
        </span>
        <span
          v-else-if="testResult"
          class="pm-conn-form__test-message"
          :class="{
            'pm-conn-form__test-message--success': testResultType === 'success',
            'pm-conn-form__test-message--error': testResultType === 'error',
          }"
        >
          {{ testResult }}
        </span>
      </div>
    </div>

    <template #footer>
      <PmButton variant="ghost" @click="$emit('close')">Cancel</PmButton>
      <PmButton variant="ghost" :loading="testLoading" @click="onTest">
        Test Connection
      </PmButton>
      <PmButton @click="onSave">
        {{ isEditing ? 'Update' : 'Save' }}
      </PmButton>
    </template>
  </PmModal>
</template>

<style scoped>
.pm-conn-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 420px;
}

.pm-conn-form__group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.pm-conn-form__group--grow {
  flex: 1;
}

.pm-conn-form__group--port {
  width: 100px;
  flex-shrink: 0;
}

.pm-conn-form__label {
  font-family: var(--pm-font-body);
  font-size: 13px;
  font-weight: 500;
  color: var(--pm-text-secondary);
}

.pm-conn-form__hint {
  font-family: var(--pm-font-body);
  font-size: 11px;
  color: var(--pm-text-muted);
}

.pm-conn-form__row {
  display: flex;
  gap: 12px;
}

.pm-conn-form__test-result {
  padding: 8px 12px;
  border-radius: var(--pm-radius-sm);
  background: var(--pm-surface-elevated);
  border: 1px solid var(--pm-border-subtle);
}

.pm-conn-form__test-loading {
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: var(--pm-font-body);
  font-size: 13px;
  color: var(--pm-text-secondary);
}

.pm-conn-form__spinner {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid transparent;
  border-top-color: var(--pm-accent);
  border-radius: 50%;
  animation: pm-spin 0.6s linear infinite;
}

.pm-conn-form__test-message {
  font-family: var(--pm-font-body);
  font-size: 13px;
}

.pm-conn-form__test-message--success {
  color: var(--pm-success);
}

.pm-conn-form__test-message--error {
  color: var(--pm-danger);
}
</style>
