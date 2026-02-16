<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { save } from '@tauri-apps/plugin-dialog'
import {
  PmButton, PmBadge, PmSelect, PmTable, PmSplitPane,
  PmConnectionModal,
} from '@/components/ui'
import type { ConnectionFormData } from '@/components/ui/PmConnectionModal.vue'
import PmSqlEditor from '@/components/db/PmSqlEditor.vue'
import PmQueryTabs from '@/components/db/PmQueryTabs.vue'
import PmSchemaTree from '@/components/db/PmSchemaTree.vue'
import PmExplainView from '@/components/db/PmExplainView.vue'
import PmCreateTableModal from '@/components/db/PmCreateTableModal.vue'
import PmAddColumnModal from '@/components/db/PmAddColumnModal.vue'
import PmDropConfirmModal from '@/components/db/PmDropConfirmModal.vue'
import { usePgManager } from '@/composables/usePgManager'
import type { SaveConnectionParams, TestConnectionParams } from '@/composables/usePgManager'
import type {
  PgTableInfo, PgViewInfo, PgFunctionInfo, PgQueryResult, ContextMenuAction,
} from '@/types/pgmanager'

const {
  connections, activeConnectionId, activeConnection, isConnected,
  schemas, tables, columns, indexes, views, functions,
  queryResult, queryLoading, queryError,
  queryHistory, savedQueries,
  tabs, activeTabId, activeTab,
  loadConnections, saveConnection, deleteConnection, testConnection,
  connect, disconnect,
  loadSchemas, loadTables, loadColumns, loadIndexes, getRowCount,
  executeQuery,
  loadHistory, saveQuery, loadSavedQueries, deleteSavedQuery,
  createTab, closeTab, setActiveTab,
  loadViews, loadFunctions,
  createTable, addColumn, dropObject, renameTable,
  exportCsv, exportJson,
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

// ── Schema tree - accumulated state ──────────────────────────────────
const selectedSchema = ref<string | null>(null)
const selectedTable = ref<{ schema: string; table: string } | null>(null)
const allTables = ref<PgTableInfo[]>([])
const allViews = ref<PgViewInfo[]>([])
const allFunctions = ref<PgFunctionInfo[]>([])

async function onSelectSchema(schema: string) {
  selectedSchema.value = schema
  await loadTables(schema)
  // Accumulate: replace tables for this schema, keep others
  allTables.value = [
    ...allTables.value.filter(t => t.schema_name !== schema),
    ...tables.value,
  ]
}

function onSelectTable(schema: string, table: string) {
  selectedTable.value = { schema, table }
  loadColumns(schema, table)
  loadIndexes(schema, table)
}

function onDoubleClickTable(schema: string, table: string) {
  selectedTable.value = { schema, table }
  if (activeTab.value) {
    activeTab.value.sql = `SELECT * FROM "${schema}"."${table}"`
  }
  runQuery()
}

async function onLoadViews(schema: string) {
  await loadViews(schema)
  allViews.value = [
    ...allViews.value.filter(v => v.schema_name !== schema),
    ...views.value,
  ]
}

async function onLoadFunctions(schema: string) {
  await loadFunctions(schema)
  allFunctions.value = [
    ...allFunctions.value.filter(f => f.schema_name !== schema),
    ...functions.value,
  ]
}

function onLoadColumns(schema: string, table: string) {
  loadColumns(schema, table)
}

function onLoadIndexes(schema: string, table: string) {
  loadIndexes(schema, table)
}

async function onRefreshTree() {
  if (!isConnected.value) return
  await loadSchemas()
  allTables.value = []
  allViews.value = []
  allFunctions.value = []
  selectedSchema.value = null
  selectedTable.value = null
}

// Reset schema tree when connection changes
watch(isConnected, (connected) => {
  if (!connected) {
    selectedSchema.value = null
    selectedTable.value = null
    allTables.value = []
    allViews.value = []
    allFunctions.value = []
  }
})

// ── Tab management ───────────────────────────────────────────────────
function onTabSelect(tabId: string) {
  setActiveTab(tabId)
}

function onTabClose(tabId: string) {
  closeTab(tabId)
}

function onTabCreate() {
  createTab()
}

function onTabRename(tabId: string, label: string) {
  const tab = tabs.value.find(t => t.id === tabId)
  if (tab) tab.label = label
}

// ── SQL editor binding ───────────────────────────────────────────────
const sqlEditorRef = ref<InstanceType<typeof PmSqlEditor> | null>(null)

const editorSql = computed({
  get: () => activeTab.value?.sql ?? '',
  set: (val: string) => {
    if (activeTab.value) {
      activeTab.value.sql = val
    }
  },
})

// Autocomplete data for the SQL editor
const editorTableNames = computed(() =>
  allTables.value.map(t => `${t.schema_name}.${t.table_name}`)
)

const editorColumnMap = computed(() => {
  const map: Record<string, string[]> = {}
  // Provide columns for the currently selected table
  if (selectedTable.value) {
    const key = `${selectedTable.value.schema}.${selectedTable.value.table}`
    map[key] = columns.value.map(c => c.column_name)
  }
  return map
})

const editorSchemas = computed(() => schemas.value)

// ── Query execution (per-tab) ────────────────────────────────────────
async function runQuery() {
  const tab = activeTab.value
  if (!tab || !tab.sql.trim() || !isConnected.value) return

  tab.loading = true
  tab.error = null

  try {
    await executeQuery(tab.sql.trim())
    tab.result = queryResult.value
    tab.error = queryError.value
  } catch (e) {
    tab.error = String(e)
    tab.result = null
  } finally {
    tab.loading = false
    await loadHistory()
  }
}

// ── EXPLAIN ──────────────────────────────────────────────────────────
const explainPlan = ref<unknown>(null)
const resultView = ref<'results' | 'explain'>('results')

async function runExplain() {
  const tab = activeTab.value
  if (!tab || !tab.sql.trim() || !isConnected.value) return

  tab.loading = true
  tab.error = null

  const explainSql = `EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) ${tab.sql.trim()}`
  try {
    await executeQuery(explainSql)
    if (queryResult.value && queryResult.value.rows.length > 0) {
      const raw = queryResult.value.rows[0][0]
      explainPlan.value = typeof raw === 'string' ? JSON.parse(raw) : raw
      resultView.value = 'explain'
    }
    tab.error = queryError.value
  } catch (e) {
    tab.error = String(e)
    explainPlan.value = null
  } finally {
    tab.loading = false
    await loadHistory()
  }
}

// ── Export ────────────────────────────────────────────────────────────
const showExportMenu = ref(false)

async function onExportCsv() {
  showExportMenu.value = false
  const tab = activeTab.value
  if (!tab || !tab.sql.trim()) return

  const filePath = await save({
    filters: [{ name: 'CSV', extensions: ['csv'] }],
    defaultPath: 'export.csv',
  })
  if (!filePath) return

  try {
    const count = await exportCsv(tab.sql.trim(), filePath)
    tab.error = null
    tab.result = {
      columns: [],
      rows: [],
      total_rows: count,
      affected_rows: null,
      duration_ms: 0,
      query_type: 'EXPORT',
    }
  } catch (e) {
    if (activeTab.value) activeTab.value.error = String(e)
  }
}

async function onExportJson() {
  showExportMenu.value = false
  const tab = activeTab.value
  if (!tab || !tab.sql.trim()) return

  const filePath = await save({
    filters: [{ name: 'JSON', extensions: ['json'] }],
    defaultPath: 'export.json',
  })
  if (!filePath) return

  try {
    const count = await exportJson(tab.sql.trim(), filePath)
    tab.error = null
    tab.result = {
      columns: [],
      rows: [],
      total_rows: count,
      affected_rows: null,
      duration_ms: 0,
      query_type: 'EXPORT',
    }
  } catch (e) {
    if (activeTab.value) activeTab.value.error = String(e)
  }
}

// ── Format ───────────────────────────────────────────────────────────
function formatSql() {
  sqlEditorRef.value?.formatSql()
}

// ── Context menu actions ─────────────────────────────────────────────
const showCreateTableModal = ref(false)
const createTableSchema = ref('')

const showAddColumnModal = ref(false)
const addColumnSchema = ref('')
const addColumnTable = ref('')

const showDropModal = ref(false)
const dropObjectType = ref('')
const dropSchema = ref('')
const dropName = ref('')
const dropRowCount = ref<number | undefined>(undefined)

async function handleContextMenu(action: ContextMenuAction) {
  switch (action.type) {
    case 'select-star': {
      if (activeTab.value) {
        activeTab.value.sql = `SELECT * FROM "${action.schema}"."${action.table}"`
      }
      await runQuery()
      break
    }
    case 'select-count': {
      if (activeTab.value) {
        activeTab.value.sql = `SELECT COUNT(*) FROM "${action.schema}"."${action.table}"`
      }
      await runQuery()
      break
    }
    case 'drop-table': {
      dropObjectType.value = 'TABLE'
      dropSchema.value = action.schema
      dropName.value = action.table
      try {
        dropRowCount.value = await getRowCount(action.schema, action.table)
      } catch {
        dropRowCount.value = undefined
      }
      showDropModal.value = true
      break
    }
    case 'drop-view': {
      dropObjectType.value = 'VIEW'
      dropSchema.value = action.schema
      dropName.value = action.view
      dropRowCount.value = undefined
      showDropModal.value = true
      break
    }
    case 'drop-index': {
      dropObjectType.value = 'INDEX'
      dropSchema.value = action.schema
      dropName.value = action.index
      dropRowCount.value = undefined
      showDropModal.value = true
      break
    }
    case 'drop-function': {
      dropObjectType.value = 'FUNCTION'
      dropSchema.value = action.schema
      dropName.value = action.func
      dropRowCount.value = undefined
      showDropModal.value = true
      break
    }
    case 'export-csv': {
      if (activeTab.value) {
        activeTab.value.sql = `SELECT * FROM "${action.schema}"."${action.table}"`
      }
      await onExportCsv()
      break
    }
    case 'export-json': {
      if (activeTab.value) {
        activeTab.value.sql = `SELECT * FROM "${action.schema}"."${action.table}"`
      }
      await onExportJson()
      break
    }
    case 'create-table': {
      createTableSchema.value = action.schema
      showCreateTableModal.value = true
      break
    }
    case 'add-column': {
      addColumnSchema.value = action.schema
      addColumnTable.value = action.table
      showAddColumnModal.value = true
      break
    }
    case 'rename-table': {
      const newName = prompt(`Rename table "${action.table}" to:`, action.table)
      if (newName && newName !== action.table) {
        try {
          await renameTable(action.schema, action.table, newName)
          await onSelectSchema(action.schema)
        } catch (e) {
          if (activeTab.value) activeTab.value.error = String(e)
        }
      }
      break
    }
    case 'refresh': {
      await onRefreshTree()
      break
    }
  }
}

// ── Modal handlers ───────────────────────────────────────────────────
async function onCreateTable(
  schema: string,
  tableName: string,
  cols: { name: string; data_type: string; is_primary_key: boolean; is_nullable: boolean; default_value: string | null }[],
) {
  try {
    await createTable(schema, tableName, cols)
    showCreateTableModal.value = false
    await onSelectSchema(schema)
  } catch (e) {
    if (activeTab.value) activeTab.value.error = String(e)
  }
}

async function onAddColumn(name: string, dataType: string, isNullable: boolean, defaultValue: string | null) {
  try {
    await addColumn(addColumnSchema.value, addColumnTable.value, name, dataType, isNullable, defaultValue ?? undefined)
    showAddColumnModal.value = false
    loadColumns(addColumnSchema.value, addColumnTable.value)
  } catch (e) {
    if (activeTab.value) activeTab.value.error = String(e)
  }
}

async function onDropConfirm() {
  try {
    await dropObject(dropObjectType.value, dropSchema.value, dropName.value)
    showDropModal.value = false
    // Refresh the affected schema
    await onSelectSchema(dropSchema.value)
    if (dropObjectType.value === 'VIEW') {
      await onLoadViews(dropSchema.value)
    }
  } catch (e) {
    if (activeTab.value) activeTab.value.error = String(e)
  }
}

// ── Results conversion (per-tab) ─────────────────────────────────────
const tabResult = computed<PgQueryResult | null>(() => activeTab.value?.result ?? null)
const tabError = computed<string | null>(() => activeTab.value?.error ?? null)
const tabLoading = computed(() => activeTab.value?.loading ?? false)

const tableColumns = computed(() =>
  tabResult.value?.columns.map(c => ({ key: c.name, label: c.name, sortable: true })) ?? []
)

const tableData = computed(() => {
  if (!tabResult.value) return []
  return tabResult.value.rows.map((row, i) => {
    const obj: Record<string, unknown> = { __rowIndex: i }
    tabResult.value!.columns.forEach((col, j) => {
      obj[col.name] = row[j]
    })
    return obj
  })
})

const statsText = computed(() => {
  if (!tabResult.value) return ''
  const r = tabResult.value
  if (r.query_type === 'EXPORT') {
    return `Exported ${r.total_rows ?? 0} rows`
  }
  if (r.query_type === 'SELECT') {
    return `${r.rows.length} rows${r.total_rows != null ? ` of ${r.total_rows}` : ''} in ${r.duration_ms}ms`
  }
  return `${r.affected_rows ?? 0} affected in ${r.duration_ms}ms`
})

function durationColorClass(ms: number | undefined): string {
  if (ms === undefined) return ''
  if (ms < 100) return 'duration--fast'
  if (ms < 1000) return 'duration--medium'
  return 'duration--slow'
}

// ── Save query ────────────────────────────────────────────────────────
const showSaveDialog = ref(false)
const saveLabel = ref('')

function openSaveQuery() {
  saveLabel.value = ''
  showSaveDialog.value = true
}

async function confirmSaveQuery() {
  if (!saveLabel.value.trim() || !activeTab.value) return
  await saveQuery(saveLabel.value.trim(), activeTab.value.sql)
  showSaveDialog.value = false
}

// ── History dropdown ──────────────────────────────────────────────────
const showHistory = ref(false)

function applyHistoryEntry(sql: string) {
  if (activeTab.value) activeTab.value.sql = sql
  showHistory.value = false
}

// ── Saved queries dropdown ────────────────────────────────────────────
const showSaved = ref(false)

function applySavedQuery(sql: string) {
  if (activeTab.value) activeTab.value.sql = sql
  showSaved.value = false
}

// ── Inline editing ───────────────────────────────────────────────────
const editingCell = ref<{ rowIdx: number; col: string } | null>(null)
const editValue = ref('')
const pendingEdits = ref<Map<number, Map<string, string>>>(new Map())

const hasEdits = computed(() => pendingEdits.value.size > 0)
const editCount = computed(() => {
  let count = 0
  for (const cols of pendingEdits.value.values()) count += cols.size
  return count
})

const primaryKeys = computed(() =>
  columns.value.filter(c => c.is_primary_key).map(c => c.column_name)
)

function startEdit(rowIdx: number, col: string, currentValue: unknown) {
  editingCell.value = { rowIdx, col }
  const pending = pendingEdits.value.get(rowIdx)?.get(col)
  editValue.value = pending ?? String(currentValue ?? '')
  nextTick(() => {
    const input = document.querySelector('.cell-edit-input') as HTMLInputElement | null
    input?.focus()
  })
}

function commitEdit() {
  if (!editingCell.value) return
  const { rowIdx, col } = editingCell.value
  const originalValue = String(tableData.value[rowIdx]?.[col] ?? '')

  if (editValue.value !== originalValue) {
    if (!pendingEdits.value.has(rowIdx)) {
      pendingEdits.value.set(rowIdx, new Map())
    }
    pendingEdits.value.get(rowIdx)!.set(col, editValue.value)
  }
  editingCell.value = null
}

function cancelEdit() {
  editingCell.value = null
}

function discardEdits() {
  pendingEdits.value = new Map()
}

function getCellValue(rowIdx: number, col: string, original: unknown): string {
  return pendingEdits.value.get(rowIdx)?.get(col) ?? String(original ?? '')
}

function isCellModified(rowIdx: number, col: string): boolean {
  return pendingEdits.value.get(rowIdx)?.has(col) ?? false
}

async function saveEdits() {
  if (!selectedTable.value || primaryKeys.value.length === 0) {
    if (activeTab.value) activeTab.value.error = 'Cannot save: no primary key detected for this table'
    return
  }
  const tab = activeTab.value
  if (!tab) return

  tab.loading = true
  tab.error = null

  try {
    for (const [rowIdx, colEdits] of pendingEdits.value) {
      const row = tableData.value[rowIdx]
      if (!row) continue

      const setClauses: string[] = []
      for (const [col, val] of colEdits) {
        setClauses.push(`"${col}" = '${val.replace(/'/g, "''")}'`)
      }

      const whereClauses = primaryKeys.value.map(pk => {
        const pkVal = row[pk]
        if (pkVal === null || pkVal === undefined) return `"${pk}" IS NULL`
        return `"${pk}" = '${String(pkVal).replace(/'/g, "''")}'`
      })

      const sql = `UPDATE "${selectedTable.value!.schema}"."${selectedTable.value!.table}" SET ${setClauses.join(', ')} WHERE ${whereClauses.join(' AND ')}`
      await executeQuery(sql)

      if (queryError.value) {
        tab.error = queryError.value
        break
      }
    }

    if (!tab.error) {
      pendingEdits.value = new Map()
      if (activeTab.value) {
        activeTab.value.sql = `SELECT * FROM "${selectedTable.value.schema}"."${selectedTable.value.table}"`
      }
      await runQuery()
    }
  } finally {
    tab.loading = false
  }
}

// Reset edits when table changes
watch(selectedTable, () => {
  pendingEdits.value = new Map()
  editingCell.value = null
})

// Reset result view when switching tabs
watch(activeTabId, () => {
  resultView.value = 'results'
  explainPlan.value = null
})

// ── Status bar info ──────────────────────────────────────────────────
const lastDuration = computed(() => tabResult.value?.duration_ms ?? undefined)

const connectionInfo = computed(() => {
  if (!activeConnection.value) return ''
  const c = activeConnection.value
  return `${c.database_name}@${c.host}:${c.port}`
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
          + New
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
        <PmSchemaTree
          :schemas="schemas"
          :tables="allTables"
          :columns="columns"
          :indexes="indexes"
          :views="allViews"
          :functions="allFunctions"
          :is-connected="isConnected"
          :selected-schema="selectedSchema"
          :selected-table="selectedTable"
          @select-schema="onSelectSchema"
          @select-table="onSelectTable"
          @double-click-table="onDoubleClickTable"
          @context-menu="handleContextMenu"
          @refresh="onRefreshTree"
          @load-views="onLoadViews"
          @load-functions="onLoadFunctions"
          @load-columns="onLoadColumns"
          @load-indexes="onLoadIndexes"
        />
      </template>

      <template #second>
        <div class="query-area">
          <!-- Tabs -->
          <PmQueryTabs
            :tabs="tabs"
            :active-tab-id="activeTabId"
            @select="onTabSelect"
            @close="onTabClose"
            @create="onTabCreate"
            @rename="onTabRename"
          />

          <!-- Query area: editor | results -->
          <PmSplitPane direction="vertical" :initial-ratio="0.4" :min-size="120" storage-key="db-query">
            <template #first>
              <div class="query-panel">
                <PmSqlEditor
                  ref="sqlEditorRef"
                  v-model="editorSql"
                  :tables="editorTableNames"
                  :columns="editorColumnMap"
                  :schemas="editorSchemas"
                  @execute="runQuery"
                  @explain="runExplain"
                  @save="openSaveQuery"
                  @format="formatSql"
                />
              </div>
            </template>

            <template #second>
              <div class="results-panel">
                <!-- Query toolbar -->
                <div class="query-toolbar">
                  <div class="query-toolbar__left">
                    <PmButton
                      size="sm"
                      :loading="tabLoading"
                      :disabled="!isConnected || !editorSql.trim()"
                      @click="runQuery"
                    >
                      Run
                    </PmButton>
                    <PmButton
                      variant="ghost"
                      size="sm"
                      :disabled="!isConnected || !editorSql.trim()"
                      @click="runExplain"
                    >
                      Explain
                    </PmButton>

                    <!-- Export dropdown -->
                    <div class="dropdown-wrapper">
                      <PmButton
                        variant="ghost"
                        size="sm"
                        :disabled="!isConnected || !editorSql.trim()"
                        @click="showExportMenu = !showExportMenu"
                      >
                        Export
                      </PmButton>
                      <div v-if="showExportMenu" class="dropdown-menu" @mouseleave="showExportMenu = false">
                        <button class="dropdown-menu__btn" @click="onExportCsv">
                          Export as CSV
                        </button>
                        <button class="dropdown-menu__btn" @click="onExportJson">
                          Export as JSON
                        </button>
                      </div>
                    </div>

                    <PmButton
                      variant="ghost"
                      size="sm"
                      :disabled="!editorSql.trim()"
                      @click="formatSql"
                    >
                      Format
                    </PmButton>
                    <PmButton
                      variant="ghost"
                      size="sm"
                      :disabled="!editorSql.trim()"
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
                  </div>

                  <div class="query-toolbar__right">
                    <span v-if="statsText" class="query-toolbar__stats" :class="durationColorClass(tabResult?.duration_ms)">
                      {{ statsText }}
                    </span>
                    <span class="query-toolbar__hint">Ctrl+Enter to run</span>
                  </div>
                </div>

                <!-- Result / Explain toggle -->
                <div v-if="explainPlan" class="result-tabs">
                  <button
                    class="result-tab"
                    :class="{ 'result-tab--active': resultView === 'results' }"
                    @click="resultView = 'results'"
                  >
                    Results
                  </button>
                  <button
                    class="result-tab"
                    :class="{ 'result-tab--active': resultView === 'explain' }"
                    @click="resultView = 'explain'"
                  >
                    Explain
                  </button>
                </div>

                <!-- Error bar -->
                <div v-if="tabError" class="results-panel__error-bar">
                  <span class="results-panel__error">{{ tabError }}</span>
                </div>

                <!-- Edit toolbar -->
                <div v-if="hasEdits" class="edit-toolbar">
                  <span class="edit-toolbar__count">{{ editCount }} modification(s)</span>
                  <PmButton size="sm" variant="ghost" @click="discardEdits">Discard</PmButton>
                  <PmButton size="sm" @click="saveEdits" :loading="tabLoading">Save</PmButton>
                </div>

                <!-- Explain view -->
                <div v-if="resultView === 'explain' && explainPlan" class="results-panel__content">
                  <PmExplainView :plan="explainPlan" />
                </div>

                <!-- Results table -->
                <div v-else class="results-panel__table">
                  <PmTable
                    :data="tableData"
                    :columns="tableColumns"
                    :loading="tabLoading"
                    :page-size="100"
                  >
                    <template v-for="col in tableColumns" :key="col.key" #[`cell-${col.key}`]="{ row, value }">
                      <input
                        v-if="editingCell?.rowIdx === row.__rowIndex && editingCell?.col === col.key"
                        :value="editValue"
                        @input="editValue = ($event.target as HTMLInputElement).value"
                        @blur="commitEdit"
                        @keydown.enter="commitEdit"
                        @keydown.escape="cancelEdit"
                        class="cell-edit-input"
                      />
                      <span
                        v-else
                        class="cell-display"
                        :class="{ 'cell-display--modified': isCellModified(row.__rowIndex, col.key) }"
                        @dblclick="startEdit(row.__rowIndex, col.key, value)"
                      >
                        {{ getCellValue(row.__rowIndex, col.key, value) }}
                      </span>
                    </template>
                  </PmTable>
                </div>
              </div>
            </template>
          </PmSplitPane>
        </div>
      </template>
    </PmSplitPane>

    <!-- Status bar -->
    <div class="status-bar">
      <div class="status-bar__left">
        <span class="status-dot" :class="isConnected ? 'status-dot--connected' : 'status-dot--disconnected'" />
        <span class="status-bar__label">{{ isConnected ? 'Connected' : 'Disconnected' }}</span>
        <span v-if="isConnected && connectionInfo" class="status-bar__connection">{{ connectionInfo }}</span>
      </div>
      <div class="status-bar__right">
        <span
          v-if="lastDuration !== undefined"
          class="status-bar__duration"
          :class="durationColorClass(lastDuration)"
        >
          {{ lastDuration }}ms last query
        </span>
      </div>
    </div>

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

    <!-- Create table modal -->
    <PmCreateTableModal
      :visible="showCreateTableModal"
      :schema="createTableSchema"
      @close="showCreateTableModal = false"
      @create="onCreateTable"
    />

    <!-- Add column modal -->
    <PmAddColumnModal
      :visible="showAddColumnModal"
      :schema="addColumnSchema"
      :table="addColumnTable"
      @close="showAddColumnModal = false"
      @add="onAddColumn"
    />

    <!-- Drop confirm modal -->
    <PmDropConfirmModal
      :visible="showDropModal"
      :object-type="dropObjectType"
      :schema="dropSchema"
      :name="dropName"
      :row-count="dropRowCount"
      @close="showDropModal = false"
      @confirm="onDropConfirm"
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

/* ── Query area container ────────────────────────────────────────── */
.query-area {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* ── Query panel ─────────────────────────────────────────────────── */
.query-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.query-panel :deep(.pm-sql-editor) {
  flex: 1;
  border: none;
  border-radius: 0;
  min-height: 0;
}

/* ── Query toolbar ───────────────────────────────────────────────── */
.query-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  padding: 6px 8px;
  background: var(--pm-surface);
  border-bottom: 1px solid var(--pm-border-subtle);
  flex-shrink: 0;
}

.query-toolbar__left {
  display: flex;
  align-items: center;
  gap: 6px;
}

.query-toolbar__right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.query-toolbar__stats {
  font-family: var(--pm-font-mono);
  font-size: 11px;
  color: var(--pm-text-secondary);
}

.query-toolbar__hint {
  font-size: 11px;
  color: var(--pm-text-muted);
  font-family: var(--pm-font-mono);
}

/* ── Duration color classes ──────────────────────────────────────── */
.duration--fast {
  color: var(--pm-success);
}

.duration--medium {
  color: var(--pm-warning);
}

.duration--slow {
  color: var(--pm-danger);
}

/* ── Result tabs ─────────────────────────────────────────────────── */
.result-tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--pm-border-subtle);
  background: var(--pm-surface);
  flex-shrink: 0;
}

.result-tab {
  padding: 6px 16px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--pm-text-muted);
  font-size: 12px;
  font-family: var(--pm-font-body);
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s;
}

.result-tab:hover {
  color: var(--pm-text-secondary);
}

.result-tab--active {
  color: var(--pm-text-primary);
  border-bottom-color: var(--pm-accent);
}

/* ── Results panel ───────────────────────────────────────────────── */
.results-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.results-panel__error-bar {
  padding: 6px 12px;
  background: var(--pm-surface);
  border-bottom: 1px solid var(--pm-border-subtle);
  flex-shrink: 0;
}

.results-panel__error {
  font-family: var(--pm-font-mono);
  font-size: 12px;
  color: var(--pm-danger);
}

.results-panel__content {
  flex: 1;
  overflow: auto;
}

.results-panel__table {
  flex: 1;
  overflow: auto;
}

.results-panel__table :deep(.pm-table-wrapper) {
  border: none;
  border-radius: 0;
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
  color: var(--pm-danger);
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

/* ── Edit toolbar ────────────────────────────────────────────────── */
.edit-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: var(--pm-accent-dim, rgba(99, 102, 241, 0.1));
  border-bottom: 1px solid var(--pm-accent);
  flex-shrink: 0;
}

.edit-toolbar__count {
  font-size: 12px;
  font-family: var(--pm-font-mono);
  color: var(--pm-accent);
  margin-right: auto;
}

/* ── Inline cell editing ─────────────────────────────────────────── */
.cell-edit-input {
  width: 100%;
  padding: 2px 4px;
  background: var(--pm-surface);
  border: 1px solid var(--pm-accent);
  border-radius: 2px;
  color: var(--pm-text-primary);
  font-family: var(--pm-font-mono);
  font-size: 12px;
  outline: none;
}

.cell-display {
  cursor: default;
  display: block;
  min-height: 1em;
}

.cell-display--modified {
  color: var(--pm-accent);
  font-weight: 600;
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

/* ── Status bar ──────────────────────────────────────────────────── */
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 16px;
  background: var(--pm-surface);
  border-top: 1px solid var(--pm-border);
  flex-shrink: 0;
  min-height: 24px;
}

.status-bar__left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-bar__right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot--connected {
  background: var(--pm-success);
  box-shadow: 0 0 6px var(--pm-success);
}

.status-dot--disconnected {
  background: var(--pm-text-muted);
}

.status-bar__label {
  font-size: 11px;
  color: var(--pm-text-secondary);
  font-family: var(--pm-font-body);
}

.status-bar__connection {
  font-family: var(--pm-font-mono);
  font-size: 11px;
  color: var(--pm-text-muted);
}

.status-bar__duration {
  font-family: var(--pm-font-mono);
  font-size: 11px;
  color: var(--pm-text-secondary);
}
</style>
