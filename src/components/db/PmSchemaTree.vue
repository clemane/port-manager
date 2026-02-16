<script setup lang="ts">
import { ref } from 'vue'
import type { PgTableInfo, PgColumnInfo, PgIndexInfo, PgViewInfo, PgFunctionInfo, ContextMenuAction } from '@/types/pgmanager'
import PmContextMenu from './PmContextMenu.vue'

const props = defineProps<{
  schemas: string[]
  tables: PgTableInfo[]
  columns: PgColumnInfo[]
  indexes: PgIndexInfo[]
  views: PgViewInfo[]
  functions: PgFunctionInfo[]
  isConnected: boolean
  selectedSchema: string | null
  selectedTable: { schema: string; table: string } | null
}>()

const emit = defineEmits<{
  selectSchema: [schema: string]
  selectTable: [schema: string, table: string]
  doubleClickTable: [schema: string, table: string]
  contextMenu: [action: ContextMenuAction]
  refresh: []
  loadViews: [schema: string]
  loadFunctions: [schema: string]
  loadColumns: [schema: string, table: string]
  loadIndexes: [schema: string, table: string]
}>()

const searchQuery = ref('')
const expandedSchemas = ref<Set<string>>(new Set())
const expandedSections = ref<Map<string, Set<string>>>(new Map())

// ── Context menu ──────────────────────────────────────────────────
const ctxMenu = ref({ visible: false, x: 0, y: 0, items: [] as { label: string; danger?: boolean; separator?: boolean }[] })
const ctxActions = ref<(ContextMenuAction | null)[]>([])

function openCtxMenu(e: MouseEvent, items: { label: string; danger?: boolean; separator?: boolean }[], actions: (ContextMenuAction | null)[]) {
  ctxMenu.value = { visible: true, x: e.clientX, y: e.clientY, items }
  ctxActions.value = actions
}

function onCtxMenuSelect(index: number) {
  const action = ctxActions.value[index]
  if (action) emit('contextMenu', action)
  ctxMenu.value.visible = false
}

function showSchemaContextMenu(e: MouseEvent, schema: string) {
  openCtxMenu(e,
    [{ label: 'Create Table...' }, { separator: true, label: '' }, { label: 'Refresh' }],
    [{ type: 'create-table', schema }, null, { type: 'refresh' }],
  )
}

function showTableContextMenu(e: MouseEvent, schema: string, table: string) {
  openCtxMenu(e, [
    { label: 'SELECT *' },
    { label: 'SELECT COUNT(*)' },
    { separator: true, label: '' },
    { label: 'Add Column...' },
    { label: 'Rename Table...' },
    { separator: true, label: '' },
    { label: 'Export CSV' },
    { label: 'Export JSON' },
    { separator: true, label: '' },
    { label: 'Drop Table', danger: true },
  ], [
    { type: 'select-star', schema, table },
    { type: 'select-count', schema, table },
    null,
    { type: 'add-column', schema, table },
    { type: 'rename-table', schema, table },
    null,
    { type: 'export-csv', schema, table },
    { type: 'export-json', schema, table },
    null,
    { type: 'drop-table', schema, table },
  ])
}

function showViewContextMenu(e: MouseEvent, schema: string, view: string) {
  openCtxMenu(e, [
    { label: 'SELECT *' },
    { separator: true, label: '' },
    { label: 'Drop View', danger: true },
  ], [
    { type: 'select-star', schema, table: view },
    null,
    { type: 'drop-view', schema, view },
  ])
}

function showIndexContextMenu(e: MouseEvent, schema: string, index: string) {
  openCtxMenu(e, [
    { label: 'Drop Index', danger: true },
  ], [
    { type: 'drop-index', schema, index },
  ])
}

function showFunctionContextMenu(e: MouseEvent, schema: string, func: string) {
  openCtxMenu(e, [
    { label: 'Drop Function', danger: true },
  ], [
    { type: 'drop-function', schema, func },
  ])
}

// ── Tree logic ────────────────────────────────────────────────────
function toggleSchema(schema: string) {
  if (expandedSchemas.value.has(schema)) {
    expandedSchemas.value.delete(schema)
  } else {
    expandedSchemas.value.add(schema)
    emit('selectSchema', schema)
  }
}

function toggleSection(schema: string, section: string) {
  if (!expandedSections.value.has(schema)) {
    expandedSections.value.set(schema, new Set())
  }
  const sections = expandedSections.value.get(schema)!
  if (sections.has(section)) {
    sections.delete(section)
  } else {
    sections.add(section)
  }
}

function isSectionExpanded(schema: string, section: string): boolean {
  return expandedSections.value.get(schema)?.has(section) ?? false
}

function filteredTables(schema: string) {
  const q = searchQuery.value.toLowerCase()
  return props.tables.filter(t =>
    t.schema_name === schema && (!q || t.table_name.toLowerCase().includes(q))
  )
}

function filteredViews(schema: string) {
  const q = searchQuery.value.toLowerCase()
  return props.views.filter(v =>
    v.schema_name === schema && (!q || v.view_name.toLowerCase().includes(q))
  )
}

function filteredFunctions(schema: string) {
  const q = searchQuery.value.toLowerCase()
  return props.functions.filter(f =>
    f.schema_name === schema && (!q || f.function_name.toLowerCase().includes(q))
  )
}

function formatCount(n: number | null): string {
  if (n === null) return '\u2014'
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return String(n)
}

function onToggleViewsSection(schema: string) {
  toggleSection(schema, 'views')
  if (isSectionExpanded(schema, 'views')) {
    emit('loadViews', schema)
  }
}

function onToggleFunctionsSection(schema: string) {
  toggleSection(schema, 'functions')
  if (isSectionExpanded(schema, 'functions')) {
    emit('loadFunctions', schema)
  }
}
</script>

<template>
  <div class="pm-schema-tree">
    <!-- Search -->
    <div class="tree-search">
      <input
        v-model="searchQuery"
        type="text"
        class="tree-search-input"
        placeholder="Search..."
      />
      <button class="tree-refresh" @click="emit('refresh')" title="Refresh">
        &#x21BB;
      </button>
    </div>

    <!-- Empty state when not connected -->
    <div v-if="!isConnected" class="tree-empty">
      Connect to browse schemas
    </div>

    <!-- Schema list -->
    <div v-else class="tree-content">
      <div v-for="schema in schemas" :key="schema" class="schema-node">
        <!-- Schema header -->
        <button
          class="tree-item tree-item--schema"
          :class="{ 'tree-item--expanded': expandedSchemas.has(schema) }"
          @click="toggleSchema(schema)"
          @contextmenu.prevent="showSchemaContextMenu($event, schema)"
        >
          <span class="tree-arrow">{{ expandedSchemas.has(schema) ? '\u25BE' : '\u25B8' }}</span>
          <span class="tree-label">{{ schema }}</span>
        </button>

        <!-- Schema contents -->
        <div v-if="expandedSchemas.has(schema)" class="schema-contents">
          <!-- Tables section -->
          <div class="section-node">
            <button
              class="tree-item tree-item--section"
              @click="toggleSection(schema, 'tables')"
            >
              <span class="tree-arrow">{{ isSectionExpanded(schema, 'tables') ? '\u25BE' : '\u25B8' }}</span>
              <span class="tree-label">Tables</span>
              <span class="tree-count">({{ filteredTables(schema).length }})</span>
            </button>

            <div v-if="isSectionExpanded(schema, 'tables')" class="section-contents">
              <button
                v-for="table in filteredTables(schema)"
                :key="table.table_name"
                class="tree-item tree-item--leaf"
                :class="{
                  'tree-item--selected': selectedTable?.schema === schema && selectedTable?.table === table.table_name
                }"
                @click="emit('selectTable', schema, table.table_name); emit('loadColumns', schema, table.table_name); emit('loadIndexes', schema, table.table_name)"
                @dblclick="emit('doubleClickTable', schema, table.table_name)"
                @contextmenu.prevent="showTableContextMenu($event, schema, table.table_name)"
              >
                <span class="tree-icon">&#x2630;</span>
                <span class="tree-label">{{ table.table_name }}</span>
                <span class="tree-meta">
                  <span v-if="table.estimated_rows !== null" class="tree-rows">~{{ formatCount(table.estimated_rows) }}</span>
                  <span v-if="table.total_size" class="tree-size">{{ table.total_size }}</span>
                </span>
              </button>
            </div>
          </div>

          <!-- Views section -->
          <div class="section-node">
            <button
              class="tree-item tree-item--section"
              @click="onToggleViewsSection(schema)"
            >
              <span class="tree-arrow">{{ isSectionExpanded(schema, 'views') ? '\u25BE' : '\u25B8' }}</span>
              <span class="tree-label">Views</span>
              <span class="tree-count">({{ filteredViews(schema).length }})</span>
            </button>

            <div v-if="isSectionExpanded(schema, 'views')" class="section-contents">
              <button
                v-for="view in filteredViews(schema)"
                :key="view.view_name"
                class="tree-item tree-item--leaf"
                @dblclick="emit('doubleClickTable', schema, view.view_name)"
                @contextmenu.prevent="showViewContextMenu($event, schema, view.view_name)"
              >
                <span class="tree-icon">&#x25C9;</span>
                <span class="tree-label">{{ view.view_name }}</span>
              </button>
            </div>
          </div>

          <!-- Indexes section -->
          <div class="section-node">
            <button
              class="tree-item tree-item--section"
              @click="toggleSection(schema, 'indexes')"
            >
              <span class="tree-arrow">{{ isSectionExpanded(schema, 'indexes') ? '\u25BE' : '\u25B8' }}</span>
              <span class="tree-label">Indexes</span>
              <span class="tree-count">({{ indexes.length }})</span>
            </button>

            <div v-if="isSectionExpanded(schema, 'indexes')" class="section-contents">
              <button
                v-for="idx in indexes"
                :key="idx.index_name"
                class="tree-item tree-item--leaf"
                @contextmenu.prevent="showIndexContextMenu($event, schema, idx.index_name)"
              >
                <span class="tree-icon">&#x26A1;</span>
                <span class="tree-label">{{ idx.index_name }}</span>
                <span v-if="idx.is_unique" class="tree-badge">UNIQUE</span>
              </button>
            </div>
          </div>

          <!-- Functions section -->
          <div class="section-node">
            <button
              class="tree-item tree-item--section"
              @click="onToggleFunctionsSection(schema)"
            >
              <span class="tree-arrow">{{ isSectionExpanded(schema, 'functions') ? '\u25BE' : '\u25B8' }}</span>
              <span class="tree-label">Functions</span>
              <span class="tree-count">({{ filteredFunctions(schema).length }})</span>
            </button>

            <div v-if="isSectionExpanded(schema, 'functions')" class="section-contents">
              <button
                v-for="fn in filteredFunctions(schema)"
                :key="fn.function_name"
                class="tree-item tree-item--leaf"
                @contextmenu.prevent="showFunctionContextMenu($event, schema, fn.function_name)"
              >
                <span class="tree-icon">&fnof;</span>
                <span class="tree-label">{{ fn.function_name }}</span>
                <span class="tree-meta tree-signature">({{ fn.argument_types }})</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Context menu -->
    <PmContextMenu
      :items="ctxMenu.items"
      :x="ctxMenu.x"
      :y="ctxMenu.y"
      :visible="ctxMenu.visible"
      @select="onCtxMenuSelect"
      @close="ctxMenu.visible = false"
    />

    <!-- Column details panel when a table is selected -->
    <div v-if="selectedTable && columns.length" class="columns-panel">
      <div class="columns-header">Columns</div>
      <div v-for="col in columns" :key="col.column_name" class="column-item">
        <span class="column-name" :class="{ 'column-name--pk': col.is_primary_key }">
          {{ col.is_primary_key ? '\uD83D\uDD11 ' : '' }}{{ col.column_name }}
        </span>
        <span class="column-type">{{ col.data_type }}</span>
        <span v-if="col.column_default" class="column-default">= {{ col.column_default }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pm-schema-tree {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  font-size: 12px;
}

.tree-search {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px;
  border-bottom: 1px solid var(--pm-border);
  flex-shrink: 0;
}
.tree-search-input {
  flex: 1;
  padding: 4px 8px;
  background: var(--pm-surface-elevated);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  color: var(--pm-text-primary);
  font-size: 12px;
  font-family: var(--pm-font-body);
  outline: none;
}
.tree-search-input:focus {
  border-color: var(--pm-accent);
}
.tree-search-input::placeholder {
  color: var(--pm-text-muted);
}
.tree-refresh {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: none;
  border: none;
  color: var(--pm-text-muted);
  cursor: pointer;
  border-radius: var(--pm-radius-sm);
  font-size: 14px;
}
.tree-refresh:hover {
  color: var(--pm-accent);
  background: var(--pm-surface-hover);
}

.tree-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: var(--pm-text-muted);
  padding: 20px;
  text-align: center;
}

.tree-content {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.tree-item {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;
  padding: 4px 8px;
  background: none;
  border: none;
  color: var(--pm-text-secondary);
  font-size: 12px;
  font-family: var(--pm-font-body);
  cursor: pointer;
  text-align: left;
  transition: background 0.1s, color 0.1s;
}
.tree-item:hover {
  background: var(--pm-surface-hover);
  color: var(--pm-text-primary);
}

.tree-item--schema {
  font-weight: 600;
  color: var(--pm-text-primary);
}

.tree-item--section {
  padding-left: 16px;
  color: var(--pm-text-muted);
  text-transform: uppercase;
  font-size: 10px;
  letter-spacing: 0.5px;
}

.tree-item--leaf {
  padding-left: 28px;
}

.tree-item--selected {
  background: var(--pm-accent-glow);
  border-left: 2px solid var(--pm-accent);
  color: var(--pm-text-primary);
}

.tree-arrow {
  width: 12px;
  font-size: 10px;
  color: var(--pm-text-muted);
  flex-shrink: 0;
}

.tree-icon {
  width: 14px;
  font-size: 10px;
  text-align: center;
  flex-shrink: 0;
  color: var(--pm-text-muted);
}

.tree-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tree-count {
  color: var(--pm-text-muted);
  font-size: 10px;
}

.tree-meta {
  display: flex;
  gap: 6px;
  color: var(--pm-text-muted);
  font-family: var(--pm-font-mono);
  font-size: 10px;
  flex-shrink: 0;
}

.tree-badge {
  font-size: 9px;
  padding: 1px 4px;
  background: var(--pm-accent-glow);
  color: var(--pm-accent);
  border-radius: 2px;
  font-weight: 600;
}

.tree-signature {
  font-style: italic;
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.schema-contents {
  animation: tree-expand 0.2s ease;
}
@keyframes tree-expand {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}

.columns-panel {
  border-top: 1px solid var(--pm-border);
  padding: 8px;
  max-height: 200px;
  overflow-y: auto;
  flex-shrink: 0;
}
.columns-header {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--pm-text-muted);
  margin-bottom: 6px;
}
.column-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 2px 0;
  font-size: 11px;
}
.column-name {
  font-family: var(--pm-font-mono);
  color: var(--pm-text-primary);
}
.column-name--pk {
  color: var(--pm-accent);
  font-weight: 600;
}
.column-type {
  color: var(--pm-text-muted);
  font-family: var(--pm-font-mono);
  font-size: 10px;
}
.column-default {
  color: var(--pm-text-muted);
  font-size: 10px;
  font-style: italic;
}
</style>
