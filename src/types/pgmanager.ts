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
