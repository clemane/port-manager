import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type {
  PgConnection,
  PgQueryResult,
  PgTableInfo,
  PgColumnInfo,
  PgIndexInfo,
  PgQueryHistoryEntry,
  PgSavedQuery,
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

export function usePgManager() {
  // ── Connection state ────────────────────────────────────────────────
  const connections = ref<PgConnection[]>([])
  const activeConnectionId = ref<string | null>(null)
  const connectedIds = ref<Set<string>>(new Set())

  // ── Schema state ────────────────────────────────────────────────────
  const schemas = ref<string[]>([])
  const tables = ref<PgTableInfo[]>([])
  const columns = ref<PgColumnInfo[]>([])
  const indexes = ref<PgIndexInfo[]>([])

  // ── Query state ─────────────────────────────────────────────────────
  const queryResult = ref<PgQueryResult | null>(null)
  const queryLoading = ref(false)
  const queryError = ref<string | null>(null)

  // ── History & saved ─────────────────────────────────────────────────
  const queryHistory = ref<PgQueryHistoryEntry[]>([])
  const savedQueries = ref<PgSavedQuery[]>([])

  // ── Computed ────────────────────────────────────────────────────────
  const activeConnection = computed(() =>
    connections.value.find((c) => c.id === activeConnectionId.value) ?? null,
  )
  const isConnected = computed(() =>
    activeConnectionId.value ? connectedIds.value.has(activeConnectionId.value) : false,
  )

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

  // ── Lifecycle ───────────────────────────────────────────────────────
  onMounted(() => loadConnections())

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
  }
}
