<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  PmButton, PmBadge, PmSelect, PmTable, PmSplitPane,
  PmCodeEditor, PmConnectionModal,
} from '@/components/ui'
import type { ConnectionFormData } from '@/components/ui/PmConnectionModal.vue'
import { usePgManager } from '@/composables/usePgManager'
import type { SaveConnectionParams, TestConnectionParams } from '@/composables/usePgManager'

const {
  connections, activeConnectionId, activeConnection, isConnected,
  schemas, tables, columns,
  queryResult, queryLoading, queryError,
  queryHistory, savedQueries,
  loadConnections, saveConnection, testConnection,
  connect, disconnect,
  loadSchemas, loadTables, loadColumns,
  executeQuery,
  loadHistory, saveQuery, loadSavedQueries, deleteSavedQuery,
} = usePgManager()

// ── Connection toolbar ────────────────────────────────────────────────
const connectionOptions = computed(() =>
  connections.value.map(c => ({
    value: c.id,
    label: c.label || `${c.host}:${c.port}/${c.database_name}`,
  }))
)

const connecting = ref(false)

async function onConnectToggle() {
  if (!activeConnectionId.value) return
  connecting.value = true
  try {
    if (isConnected.value) {
      await disconnect(activeConnectionId.value)
    } else {
      await connect(activeConnectionId.value)
      await loadHistory()
      await loadSavedQueries()
    }
  } catch (e) {
    queryError.value = String(e)
  } finally {
    connecting.value = false
  }
}

// ── Connection modal ──────────────────────────────────────────────────
const showConnectionModal = ref(false)
const editingConnection = ref<typeof activeConnection.value>(null)
const testResult = ref<string | null>(null)
const testLoading = ref(false)

function openNewConnection() {
  editingConnection.value = null
  testResult.value = null
  showConnectionModal.value = true
}

async function onConnectionSave(data: ConnectionFormData) {
  const params: SaveConnectionParams = {
    id: data.id,
    label: data.label,
    forwardId: data.forwardId ?? undefined,
    favoriteId: data.favoriteId ?? undefined,
    host: data.host,
    port: data.port,
    databaseName: data.databaseName,
    username: data.username,
    password: data.password,
    sslMode: data.sslMode,
    color: data.color ?? undefined,
  }
  await saveConnection(params)
  showConnectionModal.value = false
}

async function onConnectionTest(data: ConnectionFormData) {
  testLoading.value = true
  testResult.value = null
  try {
    const params: TestConnectionParams = {
      host: data.host,
      port: data.port,
      databaseName: data.databaseName,
      username: data.username,
      password: data.password,
    }
    testResult.value = await testConnection(params)
  } catch (e) {
    testResult.value = String(e)
  } finally {
    testLoading.value = false
  }
}

// ── Schema tree ───────────────────────────────────────────────────────
const expandedSchemas = ref<Set<string>>(new Set())
const schemaTables = ref<Map<string, typeof tables.value>>(new Map())
const loadingSchema = ref<string | null>(null)
const selectedTable = ref<{ schema: string; table: string } | null>(null)

async function toggleSchema(schema: string) {
  if (expandedSchemas.value.has(schema)) {
    expandedSchemas.value.delete(schema)
    return
  }
  expandedSchemas.value.add(schema)
  if (!schemaTables.value.has(schema)) {
    loadingSchema.value = schema
    try {
      await loadTables(schema)
      schemaTables.value.set(schema, [...tables.value])
    } finally {
      loadingSchema.value = null
    }
  }
}

function onTableClick(schema: string, table: string) {
  selectedTable.value = { schema, table }
  loadColumns(schema, table)
}

function onTableDblClick(schema: string, table: string) {
  sqlText.value = `SELECT * FROM ${schema}.${table} LIMIT 100`
}

// Reset schema tree when connection changes
watch(isConnected, (connected) => {
  if (!connected) {
    expandedSchemas.value.clear()
    schemaTables.value.clear()
    selectedTable.value = null
  }
})

// ── Query editor ──────────────────────────────────────────────────────
const sqlText = ref('')

async function runQuery() {
  if (!sqlText.value.trim() || !isConnected.value) return
  await executeQuery(sqlText.value.trim())
  await loadHistory()
}

// ── Save query ────────────────────────────────────────────────────────
const showSaveDialog = ref(false)
const saveLabel = ref('')

function openSaveQuery() {
  saveLabel.value = ''
  showSaveDialog.value = true
}

async function confirmSaveQuery() {
  if (!saveLabel.value.trim()) return
  await saveQuery(saveLabel.value.trim(), sqlText.value)
  showSaveDialog.value = false
}

// ── History dropdown ──────────────────────────────────────────────────
const showHistory = ref(false)

function applyHistoryEntry(sql: string) {
  sqlText.value = sql
  showHistory.value = false
}

// ── Saved queries dropdown ────────────────────────────────────────────
const showSaved = ref(false)

function applySavedQuery(sql: string) {
  sqlText.value = sql
  showSaved.value = false
}

// ── Results conversion ────────────────────────────────────────────────
const tableColumns = computed(() =>
  queryResult.value?.columns.map(c => ({ key: c.name, label: c.name, sortable: true })) ?? []
)

const tableData = computed(() => {
  if (!queryResult.value) return []
  return queryResult.value.rows.map(row => {
    const obj: Record<string, unknown> = {}
    queryResult.value!.columns.forEach((col, i) => {
      obj[col.name] = row[i]
    })
    return obj
  })
})

const statsText = computed(() => {
  if (!queryResult.value) return ''
  const r = queryResult.value
  if (r.query_type === 'SELECT') {
    return `${r.rows.length} rows${r.total_rows != null ? ` of ${r.total_rows}` : ''} in ${r.duration_ms}ms`
  }
  return `${r.affected_rows ?? 0} affected in ${r.duration_ms}ms`
})
</script>

<template>
  <div class="database-view">
    <!-- Connection toolbar -->
    <div class="db-toolbar">
      <div class="db-toolbar__left">
        <PmSelect
          :model-value="activeConnectionId ?? ''"
          :options="connectionOptions"
          @update:model-value="activeConnectionId = $event || null"
          placeholder="Select connection..."
          :searchable="true"
          class="db-toolbar__select"
        />
        <PmButton
          :variant="isConnected ? 'danger' : 'primary'"
          size="sm"
          :loading="connecting"
          :disabled="!activeConnectionId"
          @click="onConnectToggle"
        >
          {{ isConnected ? 'Disconnect' : 'Connect' }}
        </PmButton>
        <PmButton variant="ghost" size="sm" @click="openNewConnection">
          New Connection
        </PmButton>
      </div>
      <div class="db-toolbar__right">
        <PmBadge :variant="isConnected ? 'running' : 'stopped'">
          {{ isConnected ? 'Connected' : 'Disconnected' }}
        </PmBadge>
        <span v-if="activeConnection" class="db-toolbar__info">
          {{ activeConnection.host }}:{{ activeConnection.port }}/{{ activeConnection.database_name }}
        </span>
      </div>
    </div>

    <!-- Main split: schema tree | query area -->
    <PmSplitPane direction="horizontal" :initial-ratio="0.22" :min-size="180" storage-key="db-main">
      <template #first>
        <div class="schema-panel">
          <div class="schema-panel__header">
            <span class="schema-panel__title">Schemas</span>
          </div>
          <div v-if="!isConnected" class="schema-panel__empty">
            Connect to browse schemas
          </div>
          <div v-else-if="schemas.length === 0" class="schema-panel__empty">
            No schemas found
          </div>
          <div v-else class="schema-tree">
            <div v-for="schema in schemas" :key="schema" class="schema-tree__schema">
              <div
                class="schema-tree__item schema-tree__item--schema"
                @click="toggleSchema(schema)"
              >
                <span class="schema-tree__toggle">
                  {{ expandedSchemas.has(schema) ? '\u25BE' : '\u25B8' }}
                </span>
                <svg class="schema-tree__icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                  <path d="M2 4h12v8a1 1 0 01-1 1H3a1 1 0 01-1-1V4z" />
                  <path d="M2 4l2-2h4l2 2" />
                </svg>
                <span class="schema-tree__label">{{ schema }}</span>
              </div>
              <div v-if="expandedSchemas.has(schema)" class="schema-tree__children">
                <div v-if="loadingSchema === schema" class="schema-tree__loading">
                  Loading...
                </div>
                <template v-else-if="schemaTables.get(schema)?.length">
                  <div
                    v-for="t in schemaTables.get(schema)"
                    :key="`${schema}.${t.table_name}`"
                    class="schema-tree__item schema-tree__item--table"
                    :class="{ 'schema-tree__item--selected': selectedTable?.schema === schema && selectedTable?.table === t.table_name }"
                    @click="onTableClick(schema, t.table_name)"
                    @dblclick="onTableDblClick(schema, t.table_name)"
                  >
                    <span class="schema-tree__spacer" />
                    <svg class="schema-tree__icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3">
                      <rect x="2" y="2" width="12" height="12" rx="1" />
                      <line x1="2" y1="6" x2="14" y2="6" />
                      <line x1="2" y1="10" x2="14" y2="10" />
                      <line x1="7" y1="6" x2="7" y2="14" />
                    </svg>
                    <span class="schema-tree__label">{{ t.table_name }}</span>
                    <span v-if="t.estimated_rows != null" class="schema-tree__count">
                      ~{{ t.estimated_rows }}
                    </span>
                  </div>
                </template>
                <div v-else class="schema-tree__loading">No tables</div>
              </div>
            </div>
          </div>

          <!-- Column info for selected table -->
          <div v-if="selectedTable && columns.length > 0" class="column-panel">
            <div class="column-panel__header">
              {{ selectedTable.schema }}.{{ selectedTable.table }}
            </div>
            <div
              v-for="col in columns"
              :key="col.column_name"
              class="column-panel__item"
            >
              <span class="column-panel__name" :class="{ 'column-panel__name--pk': col.is_primary_key }">
                {{ col.is_primary_key ? '\u{1F511}' : '' }}{{ col.column_name }}
              </span>
              <span class="column-panel__type">{{ col.data_type }}</span>
            </div>
          </div>
        </div>
      </template>

      <template #second>
        <!-- Query area: editor | results -->
        <PmSplitPane direction="vertical" :initial-ratio="0.4" :min-size="120" storage-key="db-query">
          <template #first>
            <div class="query-panel">
              <div class="query-panel__toolbar">
                <PmButton
                  size="sm"
                  :loading="queryLoading"
                  :disabled="!isConnected || !sqlText.trim()"
                  @click="runQuery"
                >
                  Run
                </PmButton>
                <PmButton
                  variant="ghost"
                  size="sm"
                  :disabled="!sqlText.trim()"
                  @click="openSaveQuery"
                >
                  Save
                </PmButton>

                <!-- Saved queries dropdown -->
                <div class="dropdown-wrapper">
                  <PmButton
                    variant="ghost"
                    size="sm"
                    :disabled="savedQueries.length === 0"
                    @click="showSaved = !showSaved"
                  >
                    Saved ({{ savedQueries.length }})
                  </PmButton>
                  <div v-if="showSaved" class="dropdown-menu" @mouseleave="showSaved = false">
                    <div
                      v-for="sq in savedQueries"
                      :key="sq.id"
                      class="dropdown-menu__item"
                    >
                      <button class="dropdown-menu__btn" @click="applySavedQuery(sq.sql_text)">
                        {{ sq.label }}
                      </button>
                      <button class="dropdown-menu__delete" title="Delete" @click="deleteSavedQuery(sq.id)">
                        &times;
                      </button>
                    </div>
                    <div v-if="savedQueries.length === 0" class="dropdown-menu__empty">
                      No saved queries
                    </div>
                  </div>
                </div>

                <!-- History dropdown -->
                <div class="dropdown-wrapper">
                  <PmButton
                    variant="ghost"
                    size="sm"
                    :disabled="queryHistory.length === 0"
                    @click="showHistory = !showHistory"
                  >
                    History ({{ queryHistory.length }})
                  </PmButton>
                  <div v-if="showHistory" class="dropdown-menu dropdown-menu--wide" @mouseleave="showHistory = false">
                    <button
                      v-for="h in queryHistory.slice(0, 20)"
                      :key="h.id"
                      class="dropdown-menu__btn"
                      @click="applyHistoryEntry(h.sql_text)"
                    >
                      <span class="dropdown-menu__sql">{{ h.sql_text.slice(0, 80) }}{{ h.sql_text.length > 80 ? '...' : '' }}</span>
                      <span class="dropdown-menu__meta">
                        {{ h.duration_ms != null ? `${h.duration_ms}ms` : '' }}
                        {{ h.error ? ' (error)' : '' }}
                      </span>
                    </button>
                  </div>
                </div>

                <span class="query-panel__hint">Ctrl+Enter to run</span>
              </div>
              <PmCodeEditor
                v-model="sqlText"
                placeholder="Enter SQL query..."
                @execute="runQuery"
              />
            </div>
          </template>

          <template #second>
            <div class="results-panel">
              <!-- Stats bar -->
              <div v-if="statsText || queryError" class="results-panel__stats">
                <span v-if="queryError" class="results-panel__error">{{ queryError }}</span>
                <span v-else class="results-panel__info">{{ statsText }}</span>
              </div>

              <!-- Results table -->
              <div class="results-panel__table">
                <PmTable
                  :data="tableData"
                  :columns="tableColumns"
                  :loading="queryLoading"
                  :page-size="100"
                />
              </div>
            </div>
          </template>
        </PmSplitPane>
      </template>
    </PmSplitPane>

    <!-- Save query dialog -->
    <div v-if="showSaveDialog" class="save-overlay" @click.self="showSaveDialog = false">
      <div class="save-dialog">
        <div class="save-dialog__title">Save Query</div>
        <input
          v-model="saveLabel"
          class="save-dialog__input"
          placeholder="Query label..."
          autofocus
          @keydown.enter="confirmSaveQuery"
          @keydown.escape="showSaveDialog = false"
        />
        <div class="save-dialog__actions">
          <PmButton variant="ghost" size="sm" @click="showSaveDialog = false">Cancel</PmButton>
          <PmButton size="sm" @click="confirmSaveQuery">Save</PmButton>
        </div>
      </div>
    </div>

    <!-- Connection modal -->
    <PmConnectionModal
      :open="showConnectionModal"
      :connection="editingConnection"
      :forwards="[]"
      :favorites="[]"
      :test-result="testResult"
      :test-loading="testLoading"
      @close="showConnectionModal = false"
      @save="onConnectionSave"
      @test="onConnectionTest"
    />
  </div>
</template>

<style scoped>
.database-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* ── Toolbar ─────────────────────────────────────────────────────── */
.db-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 16px;
  background: var(--pm-surface);
  border-bottom: 1px solid var(--pm-border);
  flex-shrink: 0;
}

.db-toolbar__left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.db-toolbar__select {
  width: 240px;
}

.db-toolbar__right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.db-toolbar__info {
  font-family: var(--pm-font-mono);
  font-size: 11px;
  color: var(--pm-text-muted);
}

/* ── Schema panel ────────────────────────────────────────────────── */
.schema-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--pm-surface);
  overflow: hidden;
}

.schema-panel__header {
  padding: 10px 12px;
  border-bottom: 1px solid var(--pm-border-subtle);
  flex-shrink: 0;
}

.schema-panel__title {
  font-family: var(--pm-font-display);
  font-size: 12px;
  font-weight: 600;
  color: var(--pm-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.schema-panel__empty {
  padding: 24px 12px;
  color: var(--pm-text-muted);
  font-size: 13px;
  text-align: center;
}

.schema-tree {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.schema-tree__item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 5px 8px;
  cursor: pointer;
  color: var(--pm-text-primary);
  font-size: 13px;
  font-family: var(--pm-font-body);
  transition: background 0.1s;
}

.schema-tree__item:hover {
  background: var(--pm-surface-hover);
}

.schema-tree__item--selected {
  background: var(--pm-surface-hover);
  color: var(--pm-accent);
}

.schema-tree__item--table {
  padding-left: 24px;
}

.schema-tree__toggle {
  width: 14px;
  text-align: center;
  font-size: 10px;
  color: var(--pm-text-muted);
  flex-shrink: 0;
}

.schema-tree__spacer {
  width: 14px;
  flex-shrink: 0;
}

.schema-tree__icon {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  opacity: 0.6;
}

.schema-tree__label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.schema-tree__count {
  font-size: 10px;
  color: var(--pm-text-muted);
  font-family: var(--pm-font-mono);
  flex-shrink: 0;
}

.schema-tree__loading {
  padding: 6px 24px;
  font-size: 12px;
  color: var(--pm-text-muted);
}

.schema-tree__children {
  /* no additional style needed */
}

/* ── Column panel ────────────────────────────────────────────────── */
.column-panel {
  border-top: 1px solid var(--pm-border);
  flex-shrink: 0;
  max-height: 200px;
  overflow-y: auto;
}

.column-panel__header {
  padding: 8px 12px;
  font-family: var(--pm-font-mono);
  font-size: 11px;
  font-weight: 600;
  color: var(--pm-text-secondary);
  background: var(--pm-surface-elevated);
  border-bottom: 1px solid var(--pm-border-subtle);
  position: sticky;
  top: 0;
}

.column-panel__item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 3px 12px;
  font-size: 12px;
}

.column-panel__name {
  font-family: var(--pm-font-mono);
  color: var(--pm-text-primary);
}

.column-panel__name--pk {
  color: var(--pm-accent);
}

.column-panel__type {
  font-family: var(--pm-font-mono);
  color: var(--pm-text-muted);
  font-size: 11px;
}

/* ── Query panel ─────────────────────────────────────────────────── */
.query-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.query-panel__toolbar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  background: var(--pm-surface);
  border-bottom: 1px solid var(--pm-border-subtle);
  flex-shrink: 0;
}

.query-panel__hint {
  margin-left: auto;
  font-size: 11px;
  color: var(--pm-text-muted);
  font-family: var(--pm-font-mono);
}

.query-panel :deep(.pm-code-editor) {
  flex: 1;
  border: none;
  border-radius: 0;
  resize: none;
  min-height: 0;
}

/* ── Dropdown ────────────────────────────────────────────────────── */
.dropdown-wrapper {
  position: relative;
}

.dropdown-menu {
  position: absolute;
  top: 100%;
  left: 0;
  z-index: 50;
  min-width: 200px;
  max-height: 280px;
  overflow-y: auto;
  background: var(--pm-surface-elevated);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  padding: 4px 0;
}

.dropdown-menu--wide {
  min-width: 340px;
}

.dropdown-menu__item {
  display: flex;
  align-items: center;
}

.dropdown-menu__btn {
  display: block;
  width: 100%;
  text-align: left;
  padding: 6px 12px;
  background: none;
  border: none;
  color: var(--pm-text-primary);
  font-size: 12px;
  font-family: var(--pm-font-body);
  cursor: pointer;
  transition: background 0.1s;
}

.dropdown-menu__btn:hover {
  background: var(--pm-surface-hover);
}

.dropdown-menu__delete {
  background: none;
  border: none;
  color: var(--pm-text-muted);
  cursor: pointer;
  padding: 4px 8px;
  font-size: 14px;
  flex-shrink: 0;
}

.dropdown-menu__delete:hover {
  color: var(--pm-danger, #e55);
}

.dropdown-menu__sql {
  font-family: var(--pm-font-mono);
  font-size: 11px;
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dropdown-menu__meta {
  font-size: 10px;
  color: var(--pm-text-muted);
  display: block;
}

.dropdown-menu__empty {
  padding: 8px 12px;
  color: var(--pm-text-muted);
  font-size: 12px;
}

/* ── Results panel ───────────────────────────────────────────────── */
.results-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.results-panel__stats {
  padding: 6px 12px;
  background: var(--pm-surface);
  border-bottom: 1px solid var(--pm-border-subtle);
  font-size: 12px;
  flex-shrink: 0;
}

.results-panel__info {
  font-family: var(--pm-font-mono);
  color: var(--pm-text-secondary);
}

.results-panel__error {
  font-family: var(--pm-font-mono);
  color: var(--pm-danger, #e55);
}

.results-panel__table {
  flex: 1;
  overflow: auto;
}

.results-panel__table :deep(.pm-table-wrapper) {
  border: none;
  border-radius: 0;
}

/* ── Save dialog ─────────────────────────────────────────────────── */
.save-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.save-dialog {
  background: var(--pm-surface-elevated);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius);
  padding: 20px;
  min-width: 320px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}

.save-dialog__title {
  font-family: var(--pm-font-display);
  font-size: 14px;
  font-weight: 600;
  color: var(--pm-text-primary);
  margin-bottom: 12px;
}

.save-dialog__input {
  width: 100%;
  padding: 8px 12px;
  background: var(--pm-surface);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  color: var(--pm-text-primary);
  font-family: var(--pm-font-body);
  font-size: 13px;
  outline: none;
  box-sizing: border-box;
  transition: border-color 0.15s;
}

.save-dialog__input:focus {
  border-color: var(--pm-accent);
}

.save-dialog__actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 12px;
}
</style>
