<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { PmSelect, PmButton, PmBadge, PmModal, PmInput, PmTreeView } from '@/components/ui'
import { useK8s } from '@/composables/useK8s'
import { invoke } from '@tauri-apps/api/core'
import type { TreeNode } from '@/types/tree'
import type { ServicePort, PodPort } from '@/types/k8s'

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

// Forward modal
const showForwardModal = ref(false)
const forwardPort = ref<number>(0)
const localPort = ref<string>('')

onMounted(async () => {
  await loadKubeconfigs()
})

watch(selectedCluster, async (id) => {
  if (id) {
    selectedNamespace.value = ''
    services.value = []
    pods.value = []
    await loadNamespaces(id)
  }
})

watch(selectedNamespace, async (ns) => {
  if (ns && selectedCluster.value) {
    await Promise.all([
      loadServices(selectedCluster.value, ns),
      loadPods(selectedCluster.value, ns),
    ])
  }
})

const clusterOptions = () => kubeconfigs.value.map(k => ({ value: k.id, label: k.name }))
const namespaceOptions = () => namespaces.value.map(ns => ({ value: ns, label: ns }))

const treeNodes = (): TreeNode[] => {
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
}

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
  showForwardModal.value = true
}

async function createForward() {
  if (!selectedResource.value || !selectedCluster.value || !selectedNamespace.value) return

  try {
    await invoke('create_forward', {
      kubeconfigId: selectedCluster.value,
      namespace: selectedNamespace.value,
      resourceType: selectedResource.value.type,
      resourceName: selectedResource.value.name,
      remotePort: forwardPort.value,
      localPort: localPort.value ? parseInt(localPort.value) : null,
      favoriteId: null,
    })
    showForwardModal.value = false
  } catch (e) {
    alert(String(e))
  }
}
</script>

<template>
  <div class="k8s-browser">
    <div class="k8s-browser__header">
      <h1 class="view-title">Kubernetes</h1>
      <p class="view-subtitle">Browse clusters and resources</p>
    </div>

    <div class="k8s-browser__selectors">
      <PmSelect
        v-model="selectedCluster"
        :options="clusterOptions()"
        placeholder="Select cluster..."
        :searchable="true"
      />
      <PmSelect
        v-if="selectedCluster"
        v-model="selectedNamespace"
        :options="namespaceOptions()"
        placeholder="Select namespace..."
        :searchable="true"
      />
      <span v-if="loading" class="loading-text">Loading...</span>
    </div>

    <div v-if="selectedNamespace" class="k8s-browser__content">
      <div class="k8s-browser__tree">
        <PmTreeView
          :nodes="treeNodes()"
          @select="onNodeSelect"
        />
      </div>

      <div v-if="selectedResource" class="k8s-browser__detail">
        <h3 class="detail-title">
          {{ selectedResource.name }}
          <PmBadge v-if="selectedResource.type === 'pod'" :variant="selectedResource.status === 'Running' ? 'running' : 'stopped'">
            {{ selectedResource.status }}
          </PmBadge>
        </h3>
        <p class="detail-type">{{ selectedResource.type }}</p>

        <div class="detail-ports">
          <h4>Ports</h4>
          <div v-for="(port, i) in selectedResource.ports" :key="i" class="port-item">
            <span class="port-info">
              {{ port.name || 'unnamed' }} —
              {{ getPortNumber(port) }}/{{ port.protocol }}
            </span>
            <PmButton size="sm" @click="openForwardModal(getPortNumber(port))">
              Forward
            </PmButton>
          </div>
          <p v-if="selectedResource.ports.length === 0" class="no-ports">No ports exposed</p>
        </div>
      </div>
    </div>

    <div v-else-if="selectedCluster" class="k8s-browser__empty">
      Select a namespace to browse resources
    </div>
    <div v-else class="k8s-browser__empty">
      Select a cluster to get started
    </div>

    <!-- Forward Modal -->
    <PmModal :open="showForwardModal" title="Create Port Forward" @close="showForwardModal = false">
      <div class="forward-form">
        <p class="forward-info">
          {{ selectedResource?.type }}/{{ selectedResource?.name }} : {{ forwardPort }}
        </p>
        <label class="forward-label">
          Local port (leave empty for auto)
          <PmInput v-model="localPort" type="number" placeholder="Auto-detect" />
        </label>
      </div>
      <template #footer>
        <PmButton variant="ghost" @click="showForwardModal = false">Cancel</PmButton>
        <PmButton @click="createForward">Forward</PmButton>
      </template>
    </PmModal>
  </div>
</template>

<style scoped>
.k8s-browser__header { margin-bottom: 20px; }
.view-title { font-size: 20px; font-weight: 600; color: var(--pm-text-primary); margin: 0 0 4px; }
.view-subtitle { font-size: 13px; color: var(--pm-text-secondary); margin: 0; }

.k8s-browser__selectors {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 20px;
}
.k8s-browser__selectors > * { min-width: 200px; }
.loading-text { color: var(--pm-accent); font-size: 13px; }

.k8s-browser__content {
  display: grid;
  grid-template-columns: 280px 1fr;
  gap: 20px;
  min-height: 400px;
}

.k8s-browser__tree {
  background: var(--pm-surface);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius);
  padding: 8px;
  overflow-y: auto;
  max-height: 60vh;
}

.k8s-browser__detail {
  background: var(--pm-surface);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius);
  padding: 20px;
}

.detail-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--pm-text-primary);
  margin: 0 0 4px;
  display: flex;
  align-items: center;
  gap: 8px;
}
.detail-type {
  font-size: 12px;
  color: var(--pm-text-muted);
  margin: 0 0 20px;
  text-transform: uppercase;
}
.detail-ports h4 {
  font-size: 13px;
  color: var(--pm-text-secondary);
  margin: 0 0 8px;
}
.port-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid var(--pm-border-subtle);
}
.port-info { font-family: monospace; font-size: 13px; }
.no-ports { color: var(--pm-text-muted); font-size: 13px; }

.k8s-browser__empty {
  text-align: center;
  padding: 60px 20px;
  color: var(--pm-text-muted);
  font-size: 14px;
}

.forward-form { display: flex; flex-direction: column; gap: 12px; }
.forward-info { font-family: monospace; color: var(--pm-text-secondary); font-size: 13px; margin: 0; }
.forward-label { font-size: 13px; color: var(--pm-text-secondary); display: flex; flex-direction: column; gap: 6px; }
</style>
