<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue'
import { PmButton, PmBadge, PmModal, PmInput, PmTreeView, PmConnectionModal, PmCredentialPicker } from '@/components/ui'
import type { ConnectionFormData, ConnectionInitialValues } from '@/components/ui'
import { useK8s } from '@/composables/useK8s'
import { useDbDetection } from '@/composables/useDbDetection'
import { usePgManager } from '@/composables/usePgManager'
import { invoke } from '@tauri-apps/api/core'
import { useRouter } from 'vue-router'
import type { TreeNode } from '@/types/tree'
import type { ServicePort, PodPort, DetectedCredentials } from '@/types/k8s'
import type { ActiveForward } from '@/composables/useForwards'

interface ResourceData {
  type: string
  name: string
  ports: (ServicePort | PodPort)[]
  status?: string
}

const { kubeconfigs, namespaces, services, pods, loading, loadKubeconfigs, loadNamespaces, loadServices, loadPods } = useK8s()

const selectedCluster = ref('')
const selectedNamespace = ref('')
const selectedResource = ref<ResourceData | null>(null)

const showForwardModal = ref(false)
const forwardPort = ref<number>(0)
const localPort = ref<string>('')

const router = useRouter()
const { detecting, detectedCreds, detectError, detectCredentials, reset: resetDetection } = useDbDetection()
const { saveConnection } = usePgManager()

const dbCheckbox = ref(false)
const showCredentialPicker = ref(false)
const showConnectionModal = ref(false)
const connectionInitialValues = ref<ConnectionInitialValues | null>(null)
const dbForwardId = ref<string | null>(null)

onMounted(async () => {
  await loadKubeconfigs()
})

watch(selectedCluster, async (id) => {
  if (id) {
    selectedNamespace.value = ''
    selectedResource.value = null
    services.value = []
    pods.value = []
    await loadNamespaces(id)
  }
})

watch(selectedNamespace, async (ns) => {
  if (ns && selectedCluster.value) {
    selectedResource.value = null
    await Promise.all([
      loadServices(selectedCluster.value, ns),
      loadPods(selectedCluster.value, ns),
    ])
  }
})

const expandedClusters = ref<Set<string>>(new Set())

function toggleCluster(id: string) {
  if (expandedClusters.value.has(id)) {
    expandedClusters.value.delete(id)
  } else {
    expandedClusters.value.add(id)
    selectedCluster.value = id
  }
}

function selectNamespace(ns: string) {
  selectedNamespace.value = ns
}

const treeNodes = computed((): TreeNode[] => {
  const nodes: TreeNode[] = []
  if (services.value.length > 0) {
    nodes.push({
      id: 'services',
      label: `Services (${services.value.length})`,
      children: services.value.map(svc => ({
        id: `svc-${svc.name}`,
        label: svc.name,
        data: { type: 'service', name: svc.name, ports: svc.ports } as ResourceData,
      })),
    })
  }
  if (pods.value.length > 0) {
    nodes.push({
      id: 'pods',
      label: `Pods (${pods.value.length})`,
      children: pods.value.map(pod => ({
        id: `pod-${pod.name}`,
        label: pod.name,
        data: { type: 'pod', name: pod.name, ports: pod.ports, status: pod.status } as ResourceData,
      })),
    })
  }
  return nodes
})

function onNodeSelect(node: TreeNode) {
  if (node.data) {
    selectedResource.value = node.data as ResourceData
  }
}

function getPortNumber(port: ServicePort | PodPort): number {
  if ('port' in port) return port.port
  return port.container_port
}

function openForwardModal(port: number) {
  forwardPort.value = port
  localPort.value = ''
  dbCheckbox.value = false
  showForwardModal.value = true
}

async function createForward() {
  if (!selectedResource.value || !selectedCluster.value || !selectedNamespace.value) return
  try {
    const result = await invoke<ActiveForward>('create_forward', {
      kubeconfigId: selectedCluster.value,
      namespace: selectedNamespace.value,
      resourceType: selectedResource.value.type,
      resourceName: selectedResource.value.name,
      remotePort: forwardPort.value,
      localPort: localPort.value ? parseInt(localPort.value) : null,
      favoriteId: null,
    })
    showForwardModal.value = false

    if (dbCheckbox.value && result) {
      dbForwardId.value = result.id
      const lp = result.local_port
      connectionInitialValues.value = {
        host: '127.0.0.1',
        port: lp,
        forwardId: result.id ?? undefined,
      }
      await detectCredentials(selectedCluster.value, selectedNamespace.value)
      if (detectedCreds.value.length > 1) {
        showCredentialPicker.value = true
      } else if (detectedCreds.value.length === 1) {
        applyCredential(detectedCreds.value[0])
      } else {
        showConnectionModal.value = true
      }
    }
  } catch (e) {
    alert(String(e))
  }
}

function applyCredential(cred: DetectedCredentials) {
  showCredentialPicker.value = false
  connectionInitialValues.value = {
    ...connectionInitialValues.value,
    host: cred.host ?? connectionInitialValues.value?.host ?? '127.0.0.1',
    port: cred.port ?? connectionInitialValues.value?.port ?? 5432,
    databaseName: cred.database ?? undefined,
    username: cred.username ?? undefined,
    password: cred.password ?? undefined,
    sslMode: cred.ssl_mode ?? undefined,
    label: cred.source,
    forwardId: dbForwardId.value ?? undefined,
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
  <div class="k8s">
    <div class="k8s__columns">
      <!-- Column 1: Clusters & Namespaces -->
      <div class="k8s__col k8s__col--clusters">
        <div class="col-header">Clusters</div>
        <div v-if="kubeconfigs.length === 0" class="col-empty">
          No clusters imported
        </div>
        <div v-for="kc in kubeconfigs" :key="kc.id" class="cluster-group">
          <button
            class="cluster-item"
            :class="{ 'cluster-item--active': selectedCluster === kc.id }"
            @click="toggleCluster(kc.id)"
          >
            <svg class="cluster-item__chevron" :class="{ 'cluster-item__chevron--open': expandedClusters.has(kc.id) }" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12"><path d="M6 4l4 4-4 4" stroke-linecap="round" stroke-linejoin="round"/></svg>
            <span class="cluster-item__name">{{ kc.name }}</span>
          </button>
          <div v-if="expandedClusters.has(kc.id)" class="namespace-list">
            <div v-if="loading" class="col-loading">Loading...</div>
            <button
              v-for="ns in namespaces"
              :key="ns"
              class="namespace-item"
              :class="{ 'namespace-item--active': selectedNamespace === ns }"
              @click="selectNamespace(ns)"
            >
              {{ ns }}
            </button>
            <div v-if="!loading && namespaces.length === 0" class="col-empty-small">No namespaces</div>
          </div>
        </div>
      </div>

      <!-- Column 2: Resources -->
      <div class="k8s__col k8s__col--resources">
        <div class="col-header">Resources</div>
        <div v-if="selectedNamespace">
          <div v-if="loading" class="col-loading">Loading...</div>
          <PmTreeView
            v-else
            :nodes="treeNodes"
            @select="onNodeSelect"
          />
          <div v-if="!loading && treeNodes.length === 0" class="col-empty">No resources found</div>
        </div>
        <div v-else class="col-empty">
          Select a namespace
        </div>
      </div>

      <!-- Column 3: Details -->
      <div class="k8s__col k8s__col--detail">
        <div class="col-header">Details</div>
        <div v-if="selectedResource" class="detail-panel">
          <div class="detail-panel__header">
            <h3 class="detail-panel__name">{{ selectedResource.name }}</h3>
            <PmBadge v-if="selectedResource.type === 'pod' && selectedResource.status" :variant="selectedResource.status === 'Running' ? 'running' : 'stopped'">
              {{ selectedResource.status }}
            </PmBadge>
          </div>
          <span class="detail-panel__type">{{ selectedResource.type }}</span>

          <div class="detail-panel__ports">
            <div class="detail-panel__ports-header">Ports</div>
            <div v-for="(port, i) in selectedResource.ports" :key="i" class="port-row">
              <span class="port-row__info">
                <span class="port-row__name">{{ port.name || 'unnamed' }}</span>
                <span class="port-row__value">{{ getPortNumber(port) }}/{{ port.protocol }}</span>
              </span>
              <PmButton size="sm" @click="openForwardModal(getPortNumber(port))">
                Forward
              </PmButton>
            </div>
            <div v-if="selectedResource.ports.length === 0" class="col-empty-small">No ports exposed</div>
          </div>
        </div>
        <div v-else class="col-empty col-empty--centered">
          <svg viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.5" width="48" height="48" opacity="0.3">
            <circle cx="24" cy="24" r="18" />
            <path d="M24 16v8m0 4h.01" stroke-linecap="round" />
          </svg>
          <span>Select a resource to view ports</span>
        </div>
      </div>
    </div>

    <PmModal :open="showForwardModal" title="Create Port Forward" @close="showForwardModal = false">
      <div class="forward-form">
        <p class="forward-info">
          {{ selectedResource?.type }}/{{ selectedResource?.name }} : {{ forwardPort }}
        </p>
        <label class="form-label">
          Local port (leave empty for auto)
          <PmInput v-model="localPort" type="number" placeholder="Auto-detect" />
        </label>
        <label class="form-checkbox">
          <input type="checkbox" v-model="dbCheckbox" />
          <span>Open Database Manager</span>
        </label>
      </div>
      <template #footer>
        <PmButton variant="ghost" @click="showForwardModal = false">Cancel</PmButton>
        <PmButton @click="createForward">Forward</PmButton>
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
.k8s__columns {
  display: grid;
  grid-template-columns: 200px 250px 1fr;
  height: calc(100vh - 160px);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius);
  overflow: hidden;
}

.k8s__col {
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  background: var(--pm-surface);
}

.k8s__col--resources {
  border-left: 1px solid var(--pm-border);
  border-right: 1px solid var(--pm-border);
}

.col-header {
  font-family: var(--pm-font-body);
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--pm-text-muted);
  padding: 12px 12px 8px;
  position: sticky;
  top: 0;
  background: var(--pm-surface);
  z-index: 1;
}

.col-empty {
  color: var(--pm-text-muted);
  font-size: 13px;
  padding: 20px 12px;
  text-align: center;
}

.col-empty--centered {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  height: 100%;
  padding: 40px;
}

.col-empty-small {
  color: var(--pm-text-muted);
  font-size: 12px;
  padding: 8px 12px;
}

.col-loading {
  color: var(--pm-accent);
  font-size: 12px;
  padding: 8px 12px;
}

.cluster-item {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 8px 12px;
  background: none;
  border: none;
  color: var(--pm-text-primary);
  font-family: var(--pm-font-body);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s;
  text-align: left;
}

.cluster-item:hover { background: var(--pm-surface-hover); }
.cluster-item--active { color: var(--pm-accent); }

.cluster-item__chevron {
  flex-shrink: 0;
  transition: transform 0.15s;
}
.cluster-item__chevron--open { transform: rotate(90deg); }

.namespace-list {
  padding: 0 0 4px 20px;
}

.namespace-item {
  display: block;
  width: 100%;
  padding: 4px 12px;
  background: none;
  border: none;
  color: var(--pm-text-secondary);
  font-family: var(--pm-font-mono);
  font-size: 12px;
  cursor: pointer;
  text-align: left;
  border-radius: var(--pm-radius-sm);
  transition: background 0.15s, color 0.15s;
}

.namespace-item:hover {
  background: var(--pm-surface-hover);
  color: var(--pm-text-primary);
}

.namespace-item--active {
  background: var(--pm-surface-active);
  color: var(--pm-accent);
}

.detail-panel {
  padding: 16px;
}

.detail-panel__header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.detail-panel__name {
  font-family: var(--pm-font-display);
  font-size: 16px;
  font-weight: 600;
  color: var(--pm-text-primary);
  margin: 0;
}

.detail-panel__type {
  font-family: var(--pm-font-body);
  font-size: 11px;
  color: var(--pm-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  display: block;
  margin-bottom: 20px;
}

.detail-panel__ports-header {
  font-family: var(--pm-font-body);
  font-size: 12px;
  font-weight: 600;
  color: var(--pm-text-secondary);
  margin-bottom: 8px;
}

.port-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid var(--pm-border-subtle);
}

.port-row__info {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.port-row__name {
  font-family: var(--pm-font-body);
  font-size: 13px;
  color: var(--pm-text-secondary);
}

.port-row__value {
  font-family: var(--pm-font-mono);
  font-size: 13px;
  color: var(--pm-text-primary);
  font-weight: 500;
}

.forward-form { display: flex; flex-direction: column; gap: 12px; }
.forward-info { font-family: var(--pm-font-mono); color: var(--pm-text-secondary); font-size: 13px; margin: 0; }
.form-label { font-family: var(--pm-font-body); font-size: 13px; color: var(--pm-text-secondary); display: flex; flex-direction: column; gap: 6px; }

.form-checkbox {
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: var(--pm-font-body);
  font-size: 13px;
  color: var(--pm-text-secondary);
  cursor: pointer;
  margin-top: 4px;
}

.form-checkbox input[type="checkbox"] {
  accent-color: var(--pm-accent);
  width: 14px;
  height: 14px;
}
</style>
