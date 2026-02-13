<script setup lang="ts">
import { ref, computed } from 'vue'
import { PmInput, PmSelect, PmTable, PmBadge, PmMetricCard } from '@/components/ui'
import { usePorts } from '@/composables/usePorts'

const { ports, loading } = usePorts()

const searchQuery = ref('')
const stateFilter = ref('')
const protocolFilter = ref('')

const stateOptions = [
  { value: '', label: 'All States' },
  { value: 'LISTEN', label: 'LISTEN' },
  { value: 'ESTABLISHED', label: 'ESTABLISHED' },
  { value: 'TIME_WAIT', label: 'TIME_WAIT' },
  { value: 'CLOSE_WAIT', label: 'CLOSE_WAIT' },
]

const protocolOptions = [
  { value: '', label: 'All Protocols' },
  { value: 'tcp', label: 'TCP' },
  { value: 'udp', label: 'UDP' },
  { value: 'tcp6', label: 'TCP6' },
  { value: 'udp6', label: 'UDP6' },
]

const columns = [
  { key: 'local_port', label: 'Port', sortable: true, width: '80px' },
  { key: 'protocol', label: 'Protocol', sortable: true, width: '80px' },
  { key: 'state', label: 'State', sortable: true, width: '120px' },
  { key: 'pid', label: 'PID', sortable: true, width: '80px' },
  { key: 'process_name', label: 'Process', sortable: true },
  { key: 'remote_port', label: 'Remote', sortable: true, width: '100px' },
]

const filteredPorts = computed(() => {
  return ports.value.filter(p => {
    const matchesSearch = !searchQuery.value ||
      String(p.local_port).includes(searchQuery.value) ||
      (p.process_name?.toLowerCase().includes(searchQuery.value.toLowerCase()) ?? false)
    const matchesState = !stateFilter.value || p.state === stateFilter.value
    const matchesProtocol = !protocolFilter.value || p.protocol === protocolFilter.value
    return matchesSearch && matchesState && matchesProtocol
  })
})

const conflictPorts = computed(() => {
  const portMap = new Map<number, Set<number | null>>()
  for (const p of ports.value) {
    if (p.state !== 'LISTEN') continue
    if (!portMap.has(p.local_port)) portMap.set(p.local_port, new Set())
    portMap.get(p.local_port)!.add(p.pid)
  }
  const conflicts = new Set<number>()
  for (const [port, pids] of portMap) {
    if (pids.size > 1) conflicts.add(port)
  }
  return conflicts
})

const listenCount = computed(() => ports.value.filter(p => p.state === 'LISTEN').length)
const establishedCount = computed(() => ports.value.filter(p => p.state === 'ESTABLISHED').length)
const conflictCount = computed(() => conflictPorts.value.size)

function isConflict(port: number): boolean {
  return conflictPorts.value.has(port)
}

function rowClass(row: Record<string, any>): string | undefined {
  return isConflict(row.local_port) ? 'pm-table__row--conflict' : undefined
}

function stateBadgeVariant(state: string): 'running' | 'error' | 'stopped' | 'info' {
  switch (state) {
    case 'LISTEN': return 'running'
    case 'ESTABLISHED': return 'info'
    case 'CLOSE_WAIT':
    case 'TIME_WAIT': return 'stopped'
    default: return 'stopped'
  }
}
</script>

<template>
  <div class="dashboard">
    <div class="dashboard__metrics">
      <PmMetricCard label="Listening" :value="listenCount" color="success" />
      <PmMetricCard label="Established" :value="establishedCount" color="accent" />
      <PmMetricCard label="Conflicts" :value="conflictCount" color="danger" />
      <PmMetricCard label="Total" :value="filteredPorts.length" color="muted" />
    </div>

    <div class="dashboard__filters">
      <PmInput
        v-model="searchQuery"
        type="search"
        placeholder="Search by port or process..."
        class="dashboard__search"
      />
      <PmSelect
        v-model="stateFilter"
        :options="stateOptions"
        placeholder="Filter by state"
      />
      <PmSelect
        v-model="protocolFilter"
        :options="protocolOptions"
        placeholder="Filter by protocol"
      />
    </div>

    <PmTable :data="filteredPorts" :columns="columns" :loading="loading" :row-class="rowClass">
      <template #cell-local_port="{ row }">
        <span class="port-value" :class="{ 'port-value--conflict': isConflict(row.local_port) }">
          {{ row.local_port }}
        </span>
        <span v-if="isConflict(row.local_port)" class="conflict-badge">CONFLICT</span>
      </template>
      <template #cell-state="{ value }">
        <PmBadge :variant="stateBadgeVariant(value)">{{ value }}</PmBadge>
      </template>
      <template #cell-pid="{ value }">
        <span class="mono-data">{{ value ?? '-' }}</span>
      </template>
      <template #cell-process_name="{ value }">
        <span class="mono-data">{{ value ?? '-' }}</span>
      </template>
    </PmTable>
  </div>
</template>

<style scoped>
.dashboard__metrics {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 20px;
}

.dashboard__filters {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  padding: 12px 16px;
  background: var(--pm-surface);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius);
}

.dashboard__search {
  max-width: 300px;
}

.port-value {
  font-family: var(--pm-font-mono);
  font-weight: 500;
}

.port-value--conflict {
  color: var(--pm-danger);
  font-weight: 600;
}

.conflict-badge {
  margin-left: 6px;
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 3px;
  background: var(--pm-badge-error-bg);
  color: var(--pm-badge-error-text);
  font-weight: 600;
  font-family: var(--pm-font-body);
}

.mono-data {
  font-family: var(--pm-font-mono);
  color: var(--pm-text-secondary);
  font-size: 12px;
}

:deep(.pm-table__row--conflict) {
  background: color-mix(in srgb, var(--pm-danger) 5%, transparent);
}
</style>
