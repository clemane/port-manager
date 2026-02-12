<script setup lang="ts" generic="T extends Record<string, any>">
import { ref, computed } from 'vue'

const props = defineProps<{
  data: T[]
  columns: { key: string; label: string; sortable?: boolean; width?: string }[]
}>()

const sortKey = ref('')
const sortDir = ref<'asc' | 'desc'>('asc')

function toggleSort(key: string) {
  if (sortKey.value === key) {
    sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortKey.value = key
    sortDir.value = 'asc'
  }
}

const sortedData = computed(() => {
  if (!sortKey.value) return props.data
  return [...props.data].sort((a, b) => {
    const aVal = a[sortKey.value]
    const bVal = b[sortKey.value]
    const cmp = aVal < bVal ? -1 : aVal > bVal ? 1 : 0
    return sortDir.value === 'asc' ? cmp : -cmp
  })
})
</script>

<template>
  <div class="pm-table-wrapper">
    <table class="pm-table">
      <thead>
        <tr>
          <th
            v-for="col in columns"
            :key="col.key"
            :style="col.width ? { width: col.width } : {}"
            :class="{ 'pm-table__sortable': col.sortable }"
            @click="col.sortable && toggleSort(col.key)"
          >
            {{ col.label }}
            <span v-if="col.sortable && sortKey === col.key">
              {{ sortDir === 'asc' ? '\u25B2' : '\u25BC' }}
            </span>
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(row, i) in sortedData" :key="i">
          <td v-for="col in columns" :key="col.key">
            <slot :name="`cell-${col.key}`" :row="row" :value="row[col.key]">
              {{ row[col.key] }}
            </slot>
          </td>
        </tr>
        <tr v-if="sortedData.length === 0">
          <td :colspan="columns.length" class="pm-table__empty">No data</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.pm-table-wrapper {
  overflow-x: auto;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius);
}
.pm-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}
.pm-table th {
  text-align: left;
  padding: 8px 12px;
  background: var(--pm-surface);
  color: var(--pm-text-secondary);
  font-weight: 500;
  border-bottom: 1px solid var(--pm-border);
  white-space: nowrap;
  user-select: none;
}
.pm-table__sortable { cursor: pointer; }
.pm-table__sortable:hover { color: var(--pm-text-primary); }
.pm-table td {
  padding: 8px 12px;
  color: var(--pm-text-primary);
  border-bottom: 1px solid var(--pm-border-subtle);
}
.pm-table tbody tr:hover { background: var(--pm-surface-hover); }
.pm-table__empty {
  text-align: center;
  color: var(--pm-text-muted);
  padding: 24px;
}
</style>
