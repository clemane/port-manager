<script setup lang="ts">
import { ref, computed } from 'vue'
import { PmInput, PmSelect, PmTable, PmBadge } from '@/components/ui'
import { usePorts } from '@/composables/usePorts'

const { ports, loading } = usePorts()

const searchQuery = ref('')
const stateFilter = ref('')

const stateOptions = [
  { value: '', label: 'All States' },
  { value: 'LISTEN', label: 'LISTEN' },
  { value: 'ESTABLISHED', label: 'ESTABLISHED' },
  { value: 'TIME_WAIT', label: 'TIME_WAIT' },
  { value: 'CLOSE_WAIT', label: 'CLOSE_WAIT' },
]

const columns = [
  { key: 'local_port', label: 'Port', sortable: true, width: '80px' },
  { key: 'protocol', label: 'Protocol', sortable: true, width: '80px' },
  { key: 'state', label: 'State', sortable: true, width: '120px' },
  { key: 'pid', label: 'PID', sortable: true, width: '80px' },
  { key: 'process_name', label: 'Process', sortable: true },
  { key: 'remote_port', label: 'Remote Port', sortable: true, width: '100px' },
]

const filteredPorts = computed(() => {
  return ports.value.filter(p => {
    const matchesSearch = !searchQuery.value ||
      String(p.local_port).includes(searchQuery.value) ||
      (p.process_name?.toLowerCase().includes(searchQuery.value.toLowerCase()) ?? false)
    const matchesState = !stateFilter.value || p.state === stateFilter.value
    return matchesSearch && matchesState
  })
})

// Find conflicting ports (same port used by multiple processes)
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

function isConflict(port: number): boolean {
  return conflictPorts.value.has(port)
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
    <div class="dashboard__header">
      <div>
        <h1 class="view-title">Dashboard</h1>
        <p class="view-subtitle">
          {{ filteredPorts.length }} ports
          <span v-if="loading" class="loading-indicator">refreshing...</span>
        </p>
      </div>
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
    </div>

    <PmTable :data="filteredPorts" :columns="columns">
      <template #cell-local_port="{ row }">
        <span :class="{ 'conflict-port': isConflict(row.local_port) }">
          {{ row.local_port }}
        </span>
        <span v-if="isConflict(row.local_port)" class="conflict-badge">CONFLICT</span>
      </template>
      <template #cell-state="{ value }">
        <PmBadge :variant="stateBadgeVariant(value)">{{ value }}</PmBadge>
      </template>
      <template #cell-pid="{ value }">
        <span class="pid">{{ value ?? '-' }}</span>
      </template>
      <template #cell-process_name="{ value }">
        <span class="process-name">{{ value ?? '-' }}</span>
      </template>
    </PmTable>
  </div>
</template>

<style scoped>
.dashboard__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}
.view-title { font-size: 20px; font-weight: 600; color: var(--pm-text-primary); margin: 0 0 4px; }
.view-subtitle { font-size: 13px; color: var(--pm-text-secondary); margin: 0; }
.loading-indicator { color: var(--pm-accent); margin-left: 8px; }

.dashboard__filters {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}
.dashboard__search { max-width: 300px; }

.conflict-port { color: var(--pm-danger); font-weight: 600; }
.conflict-badge {
  margin-left: 6px;
  font-size: 10px;
  padding: 1px 4px;
  border-radius: 3px;
  background: var(--pm-badge-error-bg);
  color: var(--pm-badge-error-text);
  font-weight: 600;
}
.pid { font-family: monospace; color: var(--pm-text-secondary); }
.process-name { font-family: monospace; }
</style>
