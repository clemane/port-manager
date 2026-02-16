import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type {
  PgConnection,
  PgQueryResult,
  PgTableInfo,
  PgColumnInfo,
  PgIndexInfo,
  PgQueryHistoryEntry,
  PgSavedQuery,
  QueryTab,
  PgViewInfo,
  PgFunctionInfo,
} from '@/types/pgmanager'

export interface SaveConnectionParams {
  id?: string
  label?: string
  forwardId?: string
  favoriteId?: string
  host: string
  port: number
  databaseName: string
  username: string
  password?: string
  sslMode: string
  color?: string
}

export interface TestConnectionParams {
  host: string
  port: number
  databaseName: string
  username: string
  password: string
}

// ── Module-level singleton state ──────────────────────────────────
const connections = ref<PgConnection[]>([])
const activeConnectionId = ref<string | null>(null)
const connectedIds = ref<Set<string>>(new Set())

const schemas = ref<string[]>([])
const tables = ref<PgTableInfo[]>([])
const columns = ref<PgColumnInfo[]>([])
const indexes = ref<PgIndexInfo[]>([])

const queryResult = ref<PgQueryResult | null>(null)
const queryLoading = ref(false)
const queryError = ref<string | null>(null)

const queryHistory = ref<PgQueryHistoryEntry[]>([])
const savedQueries = ref<PgSavedQuery[]>([])

const tabs = ref<QueryTab[]>([{
  id: crypto.randomUUID(),
  label: 'Query 1',
  sql: '',
  cursorPos: 0,
  result: null,
  error: null,
  loading: false,
}])
const activeTabId = ref<string>(tabs.value[0]!.id)
const views = ref<PgViewInfo[]>([])
const functions = ref<PgFunctionInfo[]>([])

let initialized = false

export function usePgManager() {
  // ── Computed ────────────────────────────────────────────────────────
  const activeConnection = computed(() =>
    connections.value.find((c) => c.id === activeConnectionId.value) ?? null,
  )
  const isConnected = computed(() =>
    activeConnectionId.value ? connectedIds.value.has(activeConnectionId.value) : false,
  )
  const activeTab = computed(() => tabs.value.find(t => t.id === activeTabId.value) ?? null)

  // ── Connection methods ──────────────────────────────────────────────

  async function loadConnections() {
    connections.value = await invoke<PgConnection[]>('pg_list_connections')
  }

  async function saveConnection(params: SaveConnectionParams) {
    const saved = await invoke<PgConnection>('pg_save_connection', {
      id: params.id,
      label: params.label,
      forwardId: params.forwardId,
      favoriteId: params.favoriteId,
      host: params.host,
      port: params.port,
      databaseName: params.databaseName,
      username: params.username,
      password: params.password,
      sslMode: params.sslMode,
      color: params.color,
    })
    await loadConnections()
    return saved
  }

  async function deleteConnection(id: string) {
    await invoke('pg_delete_connection', { id })
    connectedIds.value = new Set([...connectedIds.value].filter((x) => x !== id))
    if (activeConnectionId.value === id) {
      activeConnectionId.value = null
      schemas.value = []
      tables.value = []
      columns.value = []
      indexes.value = []
      queryResult.value = null
    }
    await loadConnections()
  }

  async function testConnection(params: TestConnectionParams) {
    return invoke<string>('pg_test_connection', {
      host: params.host,
      port: params.port,
      databaseName: params.databaseName,
      username: params.username,
      password: params.password,
    })
  }

  async function connect(id: string) {
    await invoke<string>('pg_connect', { id })
    connectedIds.value = new Set([...connectedIds.value, id])
    activeConnectionId.value = id
    await loadSchemas()
  }

  async function disconnect(id: string) {
    await invoke('pg_disconnect', { id })
    connectedIds.value = new Set([...connectedIds.value].filter((x) => x !== id))
    if (activeConnectionId.value === id) {
      schemas.value = []
      tables.value = []
      columns.value = []
      indexes.value = []
      queryResult.value = null
    }
  }

  // ── Schema methods ──────────────────────────────────────────────────

  async function loadSchemas() {
    if (!activeConnectionId.value) return
    schemas.value = await invoke<string[]>('pg_list_schemas', {
      id: activeConnectionId.value,
    })
  }

  async function loadTables(schema: string) {
    if (!activeConnectionId.value) return
    tables.value = await invoke<PgTableInfo[]>('pg_list_tables', {
      id: activeConnectionId.value,
      schema,
    })
  }

  async function loadColumns(schema: string, table: string) {
    if (!activeConnectionId.value) return
    columns.value = await invoke<PgColumnInfo[]>('pg_list_columns', {
      id: activeConnectionId.value,
      schema,
      table,
    })
  }

  async function loadIndexes(schema: string, table: string) {
    if (!activeConnectionId.value) return
    indexes.value = await invoke<PgIndexInfo[]>('pg_list_indexes', {
      id: activeConnectionId.value,
      schema,
      table,
    })
  }

  async function getRowCount(schema: string, table: string) {
    if (!activeConnectionId.value) return 0
    return await invoke<number>('pg_table_row_count', {
      id: activeConnectionId.value,
      schema,
      table,
    })
  }

  // ── Query methods ───────────────────────────────────────────────────

  async function executeQuery(sql: string, page = 0, pageSize = 100) {
    if (!activeConnectionId.value) return
    queryLoading.value = true
    queryError.value = null
    try {
      queryResult.value = await invoke<PgQueryResult>('pg_execute_query', {
        id: activeConnectionId.value,
        sql,
        page,
        pageSize,
      })
    } catch (e) {
      queryError.value = String(e)
      queryResult.value = null
    } finally {
      queryLoading.value = false
    }
  }

  // ── History methods ─────────────────────────────────────────────────

  async function loadHistory() {
    if (!activeConnectionId.value) return
    queryHistory.value = await invoke<PgQueryHistoryEntry[]>('pg_list_query_history', {
      connectionId: activeConnectionId.value,
    })
  }

  async function saveQuery(label: string, sqlText: string) {
    await invoke('pg_save_query', {
      connectionId: activeConnectionId.value,
      label,
      sqlText,
    })
    await loadSavedQueries()
  }

  async function loadSavedQueries() {
    savedQueries.value = await invoke<PgSavedQuery[]>('pg_list_saved_queries', {
      connectionId: activeConnectionId.value,
    })
  }

  async function deleteSavedQuery(id: string) {
    await invoke('pg_delete_saved_query', { id })
    await loadSavedQueries()
  }

  // ── Tab management ────────────────────────────────────────────────────

  function createTab(label?: string, sql?: string) {
    if (tabs.value.length >= 10) return
    const tab: QueryTab = {
      id: crypto.randomUUID(),
      label: label ?? `Query ${tabs.value.length + 1}`,
      sql: sql ?? '',
      cursorPos: 0,
      result: null,
      error: null,
      loading: false,
    }
    tabs.value.push(tab)
    activeTabId.value = tab.id
  }

  function closeTab(tabId: string) {
    const idx = tabs.value.findIndex(t => t.id === tabId)
    if (idx === -1 || tabs.value.length <= 1) return
    tabs.value.splice(idx, 1)
    if (activeTabId.value === tabId) {
      activeTabId.value = tabs.value[Math.min(idx, tabs.value.length - 1)]!.id
    }
  }

  function setActiveTab(tabId: string) {
    activeTabId.value = tabId
  }

  // ── Schema: views, functions ─────────────────────────────────────────

  async function loadViews(schema: string) {
    if (!activeConnectionId.value) return
    views.value = await invoke<PgViewInfo[]>('pg_list_views', {
      id: activeConnectionId.value,
      schema,
    })
  }

  async function loadFunctions(schema: string) {
    if (!activeConnectionId.value) return
    functions.value = await invoke<PgFunctionInfo[]>('pg_list_functions', {
      id: activeConnectionId.value,
      schema,
    })
  }

  // ── DDL methods ──────────────────────────────────────────────────────

  async function createTable(schema: string, tableName: string, columns: { name: string; data_type: string; is_primary_key: boolean; is_nullable: boolean; default_value: string | null }[]) {
    if (!activeConnectionId.value) return
    await invoke('pg_create_table', {
      id: activeConnectionId.value,
      schema,
      tableName,
      columns,
    })
  }

  async function addColumn(schema: string, table: string, name: string, dataType: string, isNullable: boolean, defaultValue?: string) {
    if (!activeConnectionId.value) return
    await invoke('pg_add_column', {
      id: activeConnectionId.value,
      schema,
      table,
      name,
      dataType,
      isNullable,
      defaultValue: defaultValue ?? null,
    })
  }

  async function dropObject(objectType: string, schema: string, name: string, cascade = false) {
    if (!activeConnectionId.value) return
    await invoke('pg_drop_object', {
      id: activeConnectionId.value,
      objectType,
      schema,
      name,
      cascade,
    })
  }

  async function renameTable(schema: string, oldName: string, newName: string) {
    if (!activeConnectionId.value) return
    await invoke('pg_rename_table', {
      id: activeConnectionId.value,
      schema,
      oldName,
      newName,
    })
  }

  // ── Export methods ───────────────────────────────────────────────────

  async function exportCsv(sql: string, filePath: string) {
    if (!activeConnectionId.value) return 0
    return await invoke<number>('pg_export_csv', {
      id: activeConnectionId.value,
      sql,
      filePath,
    })
  }

  async function exportJson(sql: string, filePath: string) {
    if (!activeConnectionId.value) return 0
    return await invoke<number>('pg_export_json', {
      id: activeConnectionId.value,
      sql,
      filePath,
    })
  }

  // ── Init guard (replaces onMounted) ─────────────────────────────────
  if (!initialized) {
    initialized = true
    loadConnections()
  }

  return {
    // State
    connections,
    activeConnectionId,
    activeConnection,
    connectedIds,
    isConnected,
    schemas,
    tables,
    columns,
    indexes,
    queryResult,
    queryLoading,
    queryError,
    queryHistory,
    savedQueries,
    tabs,
    activeTabId,
    activeTab,
    views,
    functions,
    // Methods
    loadConnections,
    saveConnection,
    deleteConnection,
    testConnection,
    connect,
    disconnect,
    loadSchemas,
    loadTables,
    loadColumns,
    loadIndexes,
    getRowCount,
    executeQuery,
    loadHistory,
    saveQuery,
    loadSavedQueries,
    deleteSavedQuery,
    createTab,
    closeTab,
    setActiveTab,
    loadViews,
    loadFunctions,
    createTable,
    addColumn,
    dropObject,
    renameTable,
    exportCsv,
    exportJson,
  }
}
