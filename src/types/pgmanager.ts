export interface PgConnection {
  id: string
  label: string | null
  forward_id: string | null
  favorite_id: string | null
  host: string
  port: number
  database_name: string
  username: string
  // password is never sent to frontend (skip_serializing in Rust)
  ssl_mode: string
  color: string | null
  created_at: string
  last_used: string | null
}

export interface PgColumnMeta {
  name: string
  data_type: string
}

export interface PgQueryResult {
  columns: PgColumnMeta[]
  rows: (string | number | boolean | null | object)[][]
  total_rows: number | null
  affected_rows: number | null
  duration_ms: number
  query_type: string
}

export interface PgTableInfo {
  schema_name: string
  table_name: string
  table_type: string
  estimated_rows: number | null
  total_size: string | null
}

export interface PgColumnInfo {
  column_name: string
  data_type: string
  is_nullable: boolean
  column_default: string | null
  is_primary_key: boolean
  ordinal_position: number
}

export interface PgIndexInfo {
  index_name: string
  index_def: string
  is_unique: boolean
  is_primary: boolean
}

export interface PgQueryHistoryEntry {
  id: string
  connection_id: string
  sql_text: string
  executed_at: string
  duration_ms: number | null
  row_count: number | null
  error: string | null
}

export interface PgSavedQuery {
  id: string
  connection_id: string | null
  label: string
  sql_text: string
  created_at: string
}

export interface QueryTab {
  id: string
  label: string
  sql: string
  cursorPos: number
  result: PgQueryResult | null
  error: string | null
  loading: boolean
}

export interface PgViewInfo {
  schema_name: string
  view_name: string
  definition: string | null
}

export interface PgFunctionInfo {
  schema_name: string
  function_name: string
  result_type: string
  argument_types: string
  function_type: string
}

export interface ExplainNode {
  id: string
  type: string
  relation?: string
  alias?: string
  startupCost: number
  totalCost: number
  planRows: number
  actualTime: number | null
  actualRows: number | null
  loops: number
  filter?: string
  indexName?: string
  sharedHitBlocks?: number
  sharedReadBlocks?: number
  children: ExplainNode[]
  percentOfTotal: number
}

export type ContextMenuAction =
  | { type: 'select-star'; schema: string; table: string }
  | { type: 'select-count'; schema: string; table: string }
  | { type: 'drop-table'; schema: string; table: string }
  | { type: 'drop-view'; schema: string; view: string }
  | { type: 'drop-index'; schema: string; index: string }
  | { type: 'drop-function'; schema: string; func: string }
  | { type: 'export-csv'; schema: string; table: string }
  | { type: 'export-json'; schema: string; table: string }
  | { type: 'create-table'; schema: string }
  | { type: 'add-column'; schema: string; table: string }
  | { type: 'rename-table'; schema: string; table: string }
  | { type: 'refresh' }
