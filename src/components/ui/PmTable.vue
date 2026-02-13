<script setup lang="ts" generic="T extends Record<string, any>">
import { ref, computed, watch } from 'vue'
import PmSkeletonLoader from './PmSkeletonLoader.vue'

const props = withDefaults(defineProps<{
  data: T[]
  columns: { key: string; label: string; sortable?: boolean; width?: string }[]
  pageSize?: number
  loading?: boolean
  rowClass?: (row: T, index: number) => string | Record<string, boolean> | undefined
}>(), {
  pageSize: 50,
  loading: false,
  rowClass: undefined,
})

const sortKey = ref('')
const sortDir = ref<'asc' | 'desc'>('asc')
const currentPage = ref(1)

// Reset to page 1 when data changes
watch(() => props.data.length, () => {
  currentPage.value = 1
})

function toggleSort(key: string) {
  if (sortKey.value === key) {
    sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortKey.value = key
    sortDir.value = 'asc'
  }
  currentPage.value = 1
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

const totalPages = computed(() => Math.max(1, Math.ceil(sortedData.value.length / props.pageSize)))

const pagedData = computed(() => {
  const start = (currentPage.value - 1) * props.pageSize
  return sortedData.value.slice(start, start + props.pageSize)
})

function goToPage(page: number) {
  if (page >= 1 && page <= totalPages.value) {
    currentPage.value = page
  }
}
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
      <tbody v-if="loading">
        <tr>
          <td :colspan="columns.length">
            <PmSkeletonLoader variant="table" :lines="5" />
          </td>
        </tr>
      </tbody>
      <tbody v-else>
        <tr
          v-for="(row, i) in pagedData"
          :key="i"
          class="pm-table__row-animate"
          :class="rowClass ? rowClass(row, i) : undefined"
          :style="{ animationDelay: i < 15 ? `${i * 20}ms` : '0ms' }"
        >
          <td v-for="col in columns" :key="col.key">
            <slot :name="`cell-${col.key}`" :row="row" :value="row[col.key]">
              {{ row[col.key] }}
            </slot>
          </td>
        </tr>
        <tr v-if="pagedData.length === 0">
          <td :colspan="columns.length" class="pm-table__empty">No data</td>
        </tr>
      </tbody>
    </table>
    <div v-if="totalPages > 1" class="pm-table__pagination">
      <span class="pm-table__page-info">
        {{ (currentPage - 1) * pageSize + 1 }}-{{ Math.min(currentPage * pageSize, sortedData.length) }}
        of {{ sortedData.length }}
      </span>
      <div class="pm-table__page-btns">
        <button :disabled="currentPage <= 1" @click="goToPage(1)">&laquo;</button>
        <button :disabled="currentPage <= 1" @click="goToPage(currentPage - 1)">&lsaquo;</button>
        <span class="pm-table__page-num">{{ currentPage }} / {{ totalPages }}</span>
        <button :disabled="currentPage >= totalPages" @click="goToPage(currentPage + 1)">&rsaquo;</button>
        <button :disabled="currentPage >= totalPages" @click="goToPage(totalPages)">&raquo;</button>
      </div>
    </div>
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
  font-family: var(--pm-font-body);
}
.pm-table th {
  text-align: left;
  padding: 10px 12px;
  background: var(--pm-surface-elevated);
  color: var(--pm-text-secondary);
  font-weight: 600;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  border-bottom: 1px solid var(--pm-border);
  white-space: nowrap;
  user-select: none;
  position: sticky;
  top: 0;
  z-index: 1;
}
.pm-table__sortable { cursor: pointer; }
.pm-table__sortable:hover { color: var(--pm-text-primary); }
.pm-table td {
  padding: 10px 12px;
  color: var(--pm-text-primary);
  border-bottom: 1px solid var(--pm-border-subtle);
  height: 44px;
}
.pm-table tbody tr {
  transition: background 0.15s, transform 0.15s;
}
.pm-table tbody tr:hover {
  background: var(--pm-surface-hover);
  transform: translateX(2px);
}
.pm-table__row-animate {
  animation: pm-slide-up 0.3s ease forwards;
  opacity: 0;
}
.pm-table__empty {
  text-align: center;
  color: var(--pm-text-muted);
  padding: 32px;
  font-size: 14px;
}
.pm-table__pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-top: 1px solid var(--pm-border);
  background: var(--pm-surface);
}
.pm-table__page-info {
  font-size: 12px;
  color: var(--pm-text-muted);
}
.pm-table__page-btns {
  display: flex;
  align-items: center;
  gap: 2px;
}
.pm-table__page-btns button {
  background: transparent;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  color: var(--pm-text-secondary);
  padding: 4px 8px;
  font-size: 12px;
  cursor: pointer;
  font-family: var(--pm-font-body);
  transition: all 0.15s;
}
.pm-table__page-btns button:hover:not(:disabled) {
  background: var(--pm-surface-hover);
  color: var(--pm-text-primary);
}
.pm-table__page-btns button:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}
.pm-table__page-num {
  font-size: 12px;
  color: var(--pm-text-secondary);
  padding: 0 8px;
}
</style>
