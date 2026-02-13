use postgres_types::Type;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::State;
use tokio_postgres::Row;

use crate::AppState;

// ── Models ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct PgConnection {
    pub id: String,
    pub label: Option<String>,
    pub forward_id: Option<String>,
    pub favorite_id: Option<String>,
    pub host: String,
    pub port: i64,
    pub database_name: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: Option<Vec<u8>>,
    pub ssl_mode: String,
    pub color: Option<String>,
    pub created_at: String,
    pub last_used: Option<String>,
}

// ── Query Execution Models ─────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct PgColumnMeta {
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Serialize)]
pub struct PgQueryResult {
    pub columns: Vec<PgColumnMeta>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub total_rows: Option<i64>,
    pub affected_rows: Option<u64>,
    pub duration_ms: u64,
    pub query_type: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PgQueryHistoryEntry {
    pub id: String,
    pub connection_id: String,
    pub sql_text: String,
    pub executed_at: String,
    pub duration_ms: Option<i64>,
    pub row_count: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PgSavedQuery {
    pub id: String,
    pub connection_id: Option<String>,
    pub label: String,
    pub sql_text: String,
    pub created_at: String,
}

// ── Row → JSON Helper ─────────────────────────────────────────────────

fn row_to_json(row: &Row) -> Vec<serde_json::Value> {
    let mut values = Vec::new();
    for i in 0..row.len() {
        let col_type = row.columns()[i].type_();
        let value = match *col_type {
            Type::BOOL => row
                .get::<_, Option<bool>>(i)
                .map(serde_json::Value::Bool)
                .unwrap_or(serde_json::Value::Null),
            Type::INT2 => row
                .get::<_, Option<i16>>(i)
                .map(|v| serde_json::Value::Number(v.into()))
                .unwrap_or(serde_json::Value::Null),
            Type::INT4 => row
                .get::<_, Option<i32>>(i)
                .map(|v| serde_json::Value::Number(v.into()))
                .unwrap_or(serde_json::Value::Null),
            Type::INT8 => row
                .get::<_, Option<i64>>(i)
                .map(|v| serde_json::Value::Number(v.into()))
                .unwrap_or(serde_json::Value::Null),
            Type::FLOAT4 => row
                .get::<_, Option<f32>>(i)
                .map(|v| serde_json::json!(v))
                .unwrap_or(serde_json::Value::Null),
            Type::FLOAT8 => row
                .get::<_, Option<f64>>(i)
                .map(|v| serde_json::json!(v))
                .unwrap_or(serde_json::Value::Null),
            Type::TEXT | Type::VARCHAR | Type::CHAR | Type::NAME | Type::BPCHAR => row
                .get::<_, Option<String>>(i)
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
            Type::UUID => row
                .get::<_, Option<uuid::Uuid>>(i)
                .map(|v| serde_json::Value::String(v.to_string()))
                .unwrap_or(serde_json::Value::Null),
            Type::JSON | Type::JSONB => row
                .get::<_, Option<serde_json::Value>>(i)
                .unwrap_or(serde_json::Value::Null),
            Type::TIMESTAMP => row
                .get::<_, Option<chrono::NaiveDateTime>>(i)
                .map(|v| serde_json::Value::String(v.to_string()))
                .unwrap_or(serde_json::Value::Null),
            Type::TIMESTAMPTZ => row
                .get::<_, Option<chrono::DateTime<chrono::Utc>>>(i)
                .map(|v| serde_json::Value::String(v.to_rfc3339()))
                .unwrap_or(serde_json::Value::Null),
            Type::BYTEA => row
                .get::<_, Option<Vec<u8>>>(i)
                .map(|v| {
                    let hex: String = v.iter().map(|b| format!("{:02x}", b)).collect();
                    serde_json::Value::String(format!("\\x{}", hex))
                })
                .unwrap_or(serde_json::Value::Null),
            // Fallback: try to get as string
            _ => row
                .get::<_, Option<String>>(i)
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        };
        values.push(value);
    }
    values
}

// ── Save (upsert) ──────────────────────────────────────────────────────

#[tauri::command]
pub async fn pg_save_connection(
    id: Option<String>,
    label: Option<String>,
    forward_id: Option<String>,
    favorite_id: Option<String>,
    host: String,
    port: i64,
    database_name: String,
    username: String,
    password: Option<String>,
    ssl_mode: String,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<PgConnection, String> {
    let key = crate::crypto::get_encryption_key()?;

    // Determine if this is an update or insert
    let conn_id = if let Some(ref existing_id) = id {
        // Check if it actually exists
        let exists: Option<(String,)> =
            sqlx::query_as("SELECT id FROM pg_connections WHERE id = ?")
                .bind(existing_id)
                .fetch_optional(&state.db)
                .await
                .map_err(|e| e.to_string())?;

        if exists.is_some() {
            // UPDATE existing connection
            if let Some(ref pw) = password {
                let encrypted = crate::crypto::encrypt_decrypt(pw.as_bytes(), &key);
                sqlx::query(
                    "UPDATE pg_connections SET label = ?, forward_id = ?, favorite_id = ?, host = ?, port = ?, database_name = ?, username = ?, password = ?, ssl_mode = ?, color = ? WHERE id = ?"
                )
                .bind(&label)
                .bind(&forward_id)
                .bind(&favorite_id)
                .bind(&host)
                .bind(port)
                .bind(&database_name)
                .bind(&username)
                .bind(&encrypted)
                .bind(&ssl_mode)
                .bind(&color)
                .bind(existing_id)
                .execute(&state.db)
                .await
                .map_err(|e| e.to_string())?;
            } else {
                // No password provided — don't update password field
                sqlx::query(
                    "UPDATE pg_connections SET label = ?, forward_id = ?, favorite_id = ?, host = ?, port = ?, database_name = ?, username = ?, ssl_mode = ?, color = ? WHERE id = ?"
                )
                .bind(&label)
                .bind(&forward_id)
                .bind(&favorite_id)
                .bind(&host)
                .bind(port)
                .bind(&database_name)
                .bind(&username)
                .bind(&ssl_mode)
                .bind(&color)
                .bind(existing_id)
                .execute(&state.db)
                .await
                .map_err(|e| e.to_string())?;
            }

            existing_id.clone()
        } else {
            // ID provided but doesn't exist — insert with this ID
            let encrypted = password
                .as_ref()
                .map(|pw| crate::crypto::encrypt_decrypt(pw.as_bytes(), &key));

            sqlx::query(
                "INSERT INTO pg_connections (id, label, forward_id, favorite_id, host, port, database_name, username, password, ssl_mode, color) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(existing_id)
            .bind(&label)
            .bind(&forward_id)
            .bind(&favorite_id)
            .bind(&host)
            .bind(port)
            .bind(&database_name)
            .bind(&username)
            .bind(&encrypted)
            .bind(&ssl_mode)
            .bind(&color)
            .execute(&state.db)
            .await
            .map_err(|e| e.to_string())?;

            existing_id.clone()
        }
    } else {
        // No ID provided — generate new UUID and insert
        let new_id = uuid::Uuid::new_v4().to_string();
        let encrypted = password
            .as_ref()
            .map(|pw| crate::crypto::encrypt_decrypt(pw.as_bytes(), &key));

        sqlx::query(
            "INSERT INTO pg_connections (id, label, forward_id, favorite_id, host, port, database_name, username, password, ssl_mode, color) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&new_id)
        .bind(&label)
        .bind(&forward_id)
        .bind(&favorite_id)
        .bind(&host)
        .bind(port)
        .bind(&database_name)
        .bind(&username)
        .bind(&encrypted)
        .bind(&ssl_mode)
        .bind(&color)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

        new_id
    };

    // Return the saved connection
    let row: PgConnection = sqlx::query_as(
        "SELECT id, label, forward_id, favorite_id, host, port, database_name, username, password, ssl_mode, color, created_at, last_used FROM pg_connections WHERE id = ?"
    )
    .bind(&conn_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row)
}

// ── List ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn pg_list_connections(
    state: State<'_, AppState>,
) -> Result<Vec<PgConnection>, String> {
    let connections: Vec<PgConnection> = sqlx::query_as(
        "SELECT id, label, forward_id, favorite_id, host, port, database_name, username, password, ssl_mode, color, created_at, last_used FROM pg_connections ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(connections)
}

// ── Delete ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn pg_delete_connection(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Remove pool from pg_pools if connected
    {
        let mut pools = state.pg_pools.lock().await;
        pools.remove(&id);
    }

    sqlx::query("DELETE FROM pg_connections WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ── Test connection ─────────────────────────────────────────────────────

#[tauri::command]
pub async fn pg_test_connection(
    host: String,
    port: i64,
    database_name: String,
    username: String,
    password: String,
) -> Result<String, String> {
    let mut config = tokio_postgres::Config::new();
    config.host(&host);
    config.port(port as u16);
    config.dbname(&database_name);
    config.user(&username);
    config.password(&password);

    let (client, connection) =
        tokio_postgres::connect(&format!(
            "host={} port={} dbname={} user={} password={}",
            host, port, database_name, username, password
        ), tokio_postgres::NoTls)
        .await
        .map_err(|e| format!("Connection failed: {e}"))?;

    // Spawn the connection handler so the client can work
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            log::error!("pg test connection error: {e}");
        }
    });

    // Run a simple query to verify it works
    client
        .simple_query("SELECT 1")
        .await
        .map_err(|e| format!("Query failed: {e}"))?;

    Ok(format!(
        "Successfully connected to {database_name}@{host}:{port} as {username}"
    ))
}

// ── Connect (create pool) ───────────────────────────────────────────────

#[tauri::command]
pub async fn pg_connect(
    id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Load connection details from DB
    let conn: PgConnection = sqlx::query_as(
        "SELECT id, label, forward_id, favorite_id, host, port, database_name, username, password, ssl_mode, color, created_at, last_used FROM pg_connections WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Connection profile not found: {e}"))?;

    // Decrypt password
    let password = if let Some(ref encrypted) = conn.password {
        let key = crate::crypto::get_encryption_key()?;
        let decrypted = crate::crypto::encrypt_decrypt(encrypted, &key);
        String::from_utf8(decrypted)
            .map_err(|e| format!("Failed to decode password: {e}"))?
    } else {
        String::new()
    };

    // Build deadpool config
    let mut cfg = deadpool_postgres::Config::new();
    cfg.host = Some(conn.host.clone());
    cfg.port = Some(conn.port as u16);
    cfg.dbname = Some(conn.database_name.clone());
    cfg.user = Some(conn.username.clone());
    cfg.password = Some(password);

    let pool = cfg
        .create_pool(Some(deadpool_postgres::Runtime::Tokio1), tokio_postgres::NoTls)
        .map_err(|e| format!("Failed to create pool: {e}"))?;

    // Verify the pool actually connects
    let _client = pool
        .get()
        .await
        .map_err(|e| format!("Failed to connect: {e}"))?;

    // Store the pool
    {
        let mut pools = state.pg_pools.lock().await;
        pools.insert(id.clone(), pool);
    }

    // Update last_used timestamp
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE pg_connections SET last_used = ? WHERE id = ?")
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let label = conn.label.unwrap_or_else(|| conn.database_name.clone());
    Ok(format!("Connected to {label}"))
}

// ── Disconnect ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn pg_disconnect(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut pools = state.pg_pools.lock().await;
    if pools.remove(&id).is_none() {
        return Err(format!("No active connection for id {id}"));
    }
    Ok(())
}

// ── View / Function / DDL Models ──────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PgViewInfo {
    pub schema_name: String,
    pub view_name: String,
    pub definition: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PgFunctionInfo {
    pub schema_name: String,
    pub function_name: String,
    pub result_type: String,
    pub argument_types: String,
    pub function_type: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateColumnDef {
    pub name: String,
    pub data_type: String,
    pub is_primary_key: bool,
    pub is_nullable: bool,
    pub default_value: Option<String>,
}

// ── Schema Browser Models ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PgTableInfo {
    pub schema_name: String,
    pub table_name: String,
    pub table_type: String,
    pub estimated_rows: Option<i64>,
    pub total_size: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PgColumnInfo {
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: Option<String>,
    pub is_primary_key: bool,
    pub ordinal_position: i32,
}

#[derive(Debug, Serialize)]
pub struct PgIndexInfo {
    pub index_name: String,
    pub index_def: String,
    pub is_unique: bool,
    pub is_primary: bool,
}

// ── Schema Browser Commands ────────────────────────────────────────────

#[tauri::command]
pub async fn pg_list_schemas(
    id: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| format!("Pool error: {e}"))?;

    let rows = client
        .query(
            "SELECT schema_name FROM information_schema.schemata \
             WHERE schema_name NOT IN ('pg_catalog', 'information_schema', 'pg_toast') \
             ORDER BY schema_name",
            &[],
        )
        .await
        .map_err(|e| format!("Query failed: {e}"))?;

    let schemas: Vec<String> = rows.iter().map(|r| r.get::<_, String>(0)).collect();
    Ok(schemas)
}

#[tauri::command]
pub async fn pg_list_tables(
    id: String,
    schema: String,
    state: State<'_, AppState>,
) -> Result<Vec<PgTableInfo>, String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| format!("Pool error: {e}"))?;

    let rows = client
        .query(
            "SELECT \
                t.table_schema as schema_name, \
                t.table_name, \
                t.table_type, \
                s.n_live_tup as estimated_rows, \
                pg_size_pretty(pg_total_relation_size(quote_ident(t.table_schema) || '.' || quote_ident(t.table_name))) as total_size \
             FROM information_schema.tables t \
             LEFT JOIN pg_stat_user_tables s \
                ON s.schemaname = t.table_schema AND s.relname = t.table_name \
             WHERE t.table_schema = $1 \
             ORDER BY t.table_name",
            &[&schema],
        )
        .await
        .map_err(|e| format!("Query failed: {e}"))?;

    let tables: Vec<PgTableInfo> = rows
        .iter()
        .map(|r| PgTableInfo {
            schema_name: r.get::<_, String>(0),
            table_name: r.get::<_, String>(1),
            table_type: r.get::<_, String>(2),
            estimated_rows: r.get::<_, Option<i64>>(3),
            total_size: r.get::<_, Option<String>>(4),
        })
        .collect();

    Ok(tables)
}

#[tauri::command]
pub async fn pg_list_columns(
    id: String,
    schema: String,
    table: String,
    state: State<'_, AppState>,
) -> Result<Vec<PgColumnInfo>, String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| format!("Pool error: {e}"))?;

    let rows = client
        .query(
            "SELECT \
                c.column_name, \
                c.data_type, \
                c.is_nullable = 'YES' as is_nullable, \
                c.column_default, \
                COALESCE( \
                    EXISTS( \
                        SELECT 1 FROM pg_constraint pc \
                        JOIN pg_class cl ON cl.oid = pc.conrelid \
                        JOIN pg_namespace ns ON ns.oid = cl.relnamespace \
                        JOIN pg_attribute a ON a.attrelid = cl.oid AND a.attname = c.column_name \
                        WHERE pc.contype = 'p' \
                        AND ns.nspname = c.table_schema \
                        AND cl.relname = c.table_name \
                        AND a.attnum = ANY(pc.conkey) \
                    ), false \
                ) as is_primary_key, \
                c.ordinal_position \
             FROM information_schema.columns c \
             WHERE c.table_schema = $1 AND c.table_name = $2 \
             ORDER BY c.ordinal_position",
            &[&schema, &table],
        )
        .await
        .map_err(|e| format!("Query failed: {e}"))?;

    let columns: Vec<PgColumnInfo> = rows
        .iter()
        .map(|r| PgColumnInfo {
            column_name: r.get::<_, String>(0),
            data_type: r.get::<_, String>(1),
            is_nullable: r.get::<_, bool>(2),
            column_default: r.get::<_, Option<String>>(3),
            is_primary_key: r.get::<_, bool>(4),
            ordinal_position: r.get::<_, i32>(5),
        })
        .collect();

    Ok(columns)
}

#[tauri::command]
pub async fn pg_list_indexes(
    id: String,
    schema: String,
    table: String,
    state: State<'_, AppState>,
) -> Result<Vec<PgIndexInfo>, String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| format!("Pool error: {e}"))?;

    let rows = client
        .query(
            "SELECT \
                indexname as index_name, \
                indexdef as index_def, \
                indexdef LIKE '%UNIQUE%' as is_unique, \
                indexname LIKE '%_pkey' as is_primary \
             FROM pg_indexes \
             WHERE schemaname = $1 AND tablename = $2 \
             ORDER BY indexname",
            &[&schema, &table],
        )
        .await
        .map_err(|e| format!("Query failed: {e}"))?;

    let indexes: Vec<PgIndexInfo> = rows
        .iter()
        .map(|r| PgIndexInfo {
            index_name: r.get::<_, String>(0),
            index_def: r.get::<_, String>(1),
            is_unique: r.get::<_, bool>(2),
            is_primary: r.get::<_, bool>(3),
        })
        .collect();

    Ok(indexes)
}

#[tauri::command]
pub async fn pg_table_row_count(
    id: String,
    schema: String,
    table: String,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    // Validate schema and table names to prevent SQL injection
    // Only allow alphanumeric characters and underscores
    let is_valid_identifier = |s: &str| -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_')
    };

    if !is_valid_identifier(&schema) {
        return Err("Invalid schema name".to_string());
    }
    if !is_valid_identifier(&table) {
        return Err("Invalid table name".to_string());
    }

    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| format!("Pool error: {e}"))?;

    let query = format!(
        "SELECT count(*) FROM \"{}\".\"{}\"",
        schema, table
    );

    let row = client
        .query_one(&query, &[])
        .await
        .map_err(|e| format!("Query failed: {e}"))?;

    let count: i64 = row.get::<_, i64>(0);
    Ok(count)
}

// ── Query Execution ────────────────────────────────────────────────────

#[tauri::command]
pub async fn pg_execute_query(
    id: String,
    sql: String,
    page: Option<i64>,
    page_size: Option<i64>,
    state: State<'_, AppState>,
) -> Result<PgQueryResult, String> {
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(100);

    // Detect query type
    let trimmed = sql.trim().to_uppercase();
    let query_type = if trimmed.starts_with("SELECT")
        || trimmed.starts_with("WITH")
        || trimmed.starts_with("EXPLAIN")
    {
        "SELECT"
    } else if trimmed.starts_with("INSERT") {
        "INSERT"
    } else if trimmed.starts_with("UPDATE") {
        "UPDATE"
    } else if trimmed.starts_with("DELETE") {
        "DELETE"
    } else {
        "DDL"
    };

    let is_select = query_type == "SELECT";

    // Get pool/client
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| format!("Pool error: {e}"))?;

    // Drop the lock before executing the query
    drop(pools);

    let start = Instant::now();

    let result = if is_select {
        // Get total count (may fail for complex queries)
        let count_sql = format!("SELECT count(*) FROM ({}) AS _count_subquery", sql);
        let total_rows = match client.query_one(&count_sql, &[]).await {
            Ok(row) => Some(row.get::<_, i64>(0)),
            Err(_) => None,
        };

        // Execute with pagination
        let paged_sql = format!("{} LIMIT {} OFFSET {}", sql, page_size, page * page_size);
        match client.query(&paged_sql, &[]).await {
            Ok(rows) => {
                let columns: Vec<PgColumnMeta> = if !rows.is_empty() {
                    rows[0]
                        .columns()
                        .iter()
                        .map(|c| PgColumnMeta {
                            name: c.name().to_string(),
                            data_type: c.type_().name().to_string(),
                        })
                        .collect()
                } else {
                    Vec::new()
                };

                let row_count = rows.len() as i64;
                let json_rows: Vec<Vec<serde_json::Value>> =
                    rows.iter().map(|r| row_to_json(r)).collect();

                let duration_ms = start.elapsed().as_millis() as u64;

                // Record success in history
                let history_id = uuid::Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    "INSERT INTO pg_query_history (id, connection_id, sql_text, duration_ms, row_count, error) VALUES (?, ?, ?, ?, ?, ?)"
                )
                .bind(&history_id)
                .bind(&id)
                .bind(&sql)
                .bind(duration_ms as i64)
                .bind(row_count)
                .bind::<Option<&str>>(None)
                .execute(&state.db)
                .await;

                Ok(PgQueryResult {
                    columns,
                    rows: json_rows,
                    total_rows,
                    affected_rows: None,
                    duration_ms,
                    query_type: query_type.to_string(),
                })
            }
            Err(e) => {
                let error_str = e.to_string();
                let duration_ms = start.elapsed().as_millis() as u64;

                // Record error in history
                let history_id = uuid::Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    "INSERT INTO pg_query_history (id, connection_id, sql_text, duration_ms, row_count, error) VALUES (?, ?, ?, ?, ?, ?)"
                )
                .bind(&history_id)
                .bind(&id)
                .bind(&sql)
                .bind(duration_ms as i64)
                .bind::<Option<i64>>(None)
                .bind(&error_str)
                .execute(&state.db)
                .await;

                Err(format!("Query failed: {e}"))
            }
        }
    } else {
        // DML/DDL
        match client.execute(sql.as_str(), &[]).await {
            Ok(affected) => {
                let row_count = affected as i64;
                let duration_ms = start.elapsed().as_millis() as u64;

                // Record success in history
                let history_id = uuid::Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    "INSERT INTO pg_query_history (id, connection_id, sql_text, duration_ms, row_count, error) VALUES (?, ?, ?, ?, ?, ?)"
                )
                .bind(&history_id)
                .bind(&id)
                .bind(&sql)
                .bind(duration_ms as i64)
                .bind(row_count)
                .bind::<Option<&str>>(None)
                .execute(&state.db)
                .await;

                Ok(PgQueryResult {
                    columns: Vec::new(),
                    rows: Vec::new(),
                    total_rows: None,
                    affected_rows: Some(affected),
                    duration_ms,
                    query_type: query_type.to_string(),
                })
            }
            Err(e) => {
                let error_str = e.to_string();
                let duration_ms = start.elapsed().as_millis() as u64;

                // Record error in history
                let history_id = uuid::Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    "INSERT INTO pg_query_history (id, connection_id, sql_text, duration_ms, row_count, error) VALUES (?, ?, ?, ?, ?, ?)"
                )
                .bind(&history_id)
                .bind(&id)
                .bind(&sql)
                .bind(duration_ms as i64)
                .bind::<Option<i64>>(None)
                .bind(&error_str)
                .execute(&state.db)
                .await;

                Err(format!("Query failed: {e}"))
            }
        }
    };

    result
}

// ── Query History ──────────────────────────────────────────────────────

#[tauri::command]
pub async fn pg_list_query_history(
    connection_id: String,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<PgQueryHistoryEntry>, String> {
    let limit = limit.unwrap_or(50);

    let entries: Vec<PgQueryHistoryEntry> = sqlx::query_as(
        "SELECT id, connection_id, sql_text, executed_at, duration_ms, row_count, error \
         FROM pg_query_history \
         WHERE connection_id = ? \
         ORDER BY executed_at DESC \
         LIMIT ?",
    )
    .bind(&connection_id)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(entries)
}

// ── Saved Queries ──────────────────────────────────────────────────────

#[tauri::command]
pub async fn pg_save_query(
    connection_id: Option<String>,
    label: String,
    sql_text: String,
    state: State<'_, AppState>,
) -> Result<PgSavedQuery, String> {
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO pg_saved_queries (id, connection_id, label, sql_text) VALUES (?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&connection_id)
    .bind(&label)
    .bind(&sql_text)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let saved: PgSavedQuery = sqlx::query_as(
        "SELECT id, connection_id, label, sql_text, created_at FROM pg_saved_queries WHERE id = ?",
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(saved)
}

#[tauri::command]
pub async fn pg_list_saved_queries(
    connection_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<PgSavedQuery>, String> {
    let queries: Vec<PgSavedQuery> = if let Some(conn_id) = connection_id {
        sqlx::query_as(
            "SELECT id, connection_id, label, sql_text, created_at \
             FROM pg_saved_queries \
             WHERE connection_id = ? \
             ORDER BY created_at DESC",
        )
        .bind(&conn_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as(
            "SELECT id, connection_id, label, sql_text, created_at \
             FROM pg_saved_queries \
             ORDER BY created_at DESC",
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?
    };

    Ok(queries)
}

#[tauri::command]
pub async fn pg_delete_saved_query(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM pg_saved_queries WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ── Views ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn pg_list_views(
    id: String,
    schema: String,
    state: State<'_, AppState>,
) -> Result<Vec<PgViewInfo>, String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| format!("Pool error: {e}"))?;

    let rows = client
        .query(
            "SELECT schemaname, viewname, definition \
             FROM pg_views \
             WHERE schemaname = $1 \
             ORDER BY viewname",
            &[&schema],
        )
        .await
        .map_err(|e| format!("Query failed: {e}"))?;

    let views: Vec<PgViewInfo> = rows
        .iter()
        .map(|r| PgViewInfo {
            schema_name: r.get::<_, String>(0),
            view_name: r.get::<_, String>(1),
            definition: r.get::<_, Option<String>>(2),
        })
        .collect();

    Ok(views)
}

// ── Functions ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn pg_list_functions(
    id: String,
    schema: String,
    state: State<'_, AppState>,
) -> Result<Vec<PgFunctionInfo>, String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| format!("Pool error: {e}"))?;

    let rows = client
        .query(
            "SELECT \
                n.nspname AS schema_name, \
                p.proname AS function_name, \
                pg_catalog.pg_get_function_result(p.oid) AS result_type, \
                pg_catalog.pg_get_function_arguments(p.oid) AS argument_types, \
                CASE p.prokind \
                    WHEN 'f' THEN 'function' \
                    WHEN 'p' THEN 'procedure' \
                    WHEN 'a' THEN 'aggregate' \
                    WHEN 'w' THEN 'window' \
                    ELSE 'unknown' \
                END AS function_type \
             FROM pg_catalog.pg_proc p \
             JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace \
             WHERE n.nspname = $1 \
             ORDER BY p.proname",
            &[&schema],
        )
        .await
        .map_err(|e| format!("Query failed: {e}"))?;

    let functions: Vec<PgFunctionInfo> = rows
        .iter()
        .map(|r| PgFunctionInfo {
            schema_name: r.get::<_, String>(0),
            function_name: r.get::<_, String>(1),
            result_type: r.get::<_, String>(2),
            argument_types: r.get::<_, String>(3),
            function_type: r.get::<_, String>(4),
        })
        .collect();

    Ok(functions)
}

// ── DDL Commands ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn pg_create_table(
    id: String,
    schema: String,
    table_name: String,
    columns: Vec<CreateColumnDef>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if columns.is_empty() {
        return Err("At least one column is required".to_string());
    }

    let mut col_defs: Vec<String> = Vec::new();
    let mut pk_cols: Vec<String> = Vec::new();

    for col in &columns {
        let mut def = format!("\"{}\" {}", col.name, col.data_type);
        if !col.is_nullable {
            def.push_str(" NOT NULL");
        }
        if let Some(ref default) = col.default_value {
            def.push_str(&format!(" DEFAULT {}", default));
        }
        col_defs.push(def);
        if col.is_primary_key {
            pk_cols.push(format!("\"{}\"", col.name));
        }
    }

    if !pk_cols.is_empty() {
        col_defs.push(format!("PRIMARY KEY ({})", pk_cols.join(", ")));
    }

    let sql = format!(
        "CREATE TABLE \"{}\".\"{}\" (\n  {}\n)",
        schema,
        table_name,
        col_defs.join(",\n  ")
    );

    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| format!("Pool error: {e}"))?;

    client
        .execute(sql.as_str(), &[])
        .await
        .map_err(|e| format!("Create table failed: {e}"))?;

    Ok(format!("Table \"{}\".\"{}\" created", schema, table_name))
}

#[tauri::command]
pub async fn pg_add_column(
    id: String,
    schema: String,
    table: String,
    column_name: String,
    data_type: String,
    is_nullable: bool,
    default_value: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut sql = format!(
        "ALTER TABLE \"{}\".\"{}\" ADD COLUMN \"{}\" {}",
        schema, table, column_name, data_type
    );

    if !is_nullable {
        sql.push_str(" NOT NULL");
    }
    if let Some(ref default) = default_value {
        sql.push_str(&format!(" DEFAULT {}", default));
    }

    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| format!("Pool error: {e}"))?;

    client
        .execute(sql.as_str(), &[])
        .await
        .map_err(|e| format!("Add column failed: {e}"))?;

    Ok(format!(
        "Column \"{}\" added to \"{}\".\"{}\"",
        column_name, schema, table
    ))
}

#[tauri::command]
pub async fn pg_drop_object(
    id: String,
    schema: String,
    name: String,
    object_type: String,
    cascade: Option<bool>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let obj_type = object_type.to_uppercase();
    let valid_types = ["TABLE", "VIEW", "INDEX", "FUNCTION"];
    if !valid_types.contains(&obj_type.as_str()) {
        return Err(format!(
            "Invalid object type '{}'. Must be one of: TABLE, VIEW, INDEX, FUNCTION",
            object_type
        ));
    }

    let mut sql = format!("DROP {} \"{}\".\"{}\"", obj_type, schema, name);
    if cascade.unwrap_or(false) {
        sql.push_str(" CASCADE");
    }

    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| format!("Pool error: {e}"))?;

    client
        .execute(sql.as_str(), &[])
        .await
        .map_err(|e| format!("Drop failed: {e}"))?;

    Ok(format!(
        "Dropped {} \"{}\".\"{}\"",
        obj_type, schema, name
    ))
}

#[tauri::command]
pub async fn pg_rename_table(
    id: String,
    schema: String,
    old_name: String,
    new_name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let sql = format!(
        "ALTER TABLE \"{}\".\"{}\" RENAME TO \"{}\"",
        schema, old_name, new_name
    );

    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| format!("Pool error: {e}"))?;

    client
        .execute(sql.as_str(), &[])
        .await
        .map_err(|e| format!("Rename failed: {e}"))?;

    Ok(format!(
        "Table \"{}\".\"{}\" renamed to \"{}\"",
        schema, old_name, new_name
    ))
}

// ── Export Commands ────────────────────────────────────────────────────

#[tauri::command]
pub async fn pg_export_csv(
    id: String,
    sql: String,
    file_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| format!("Pool error: {e}"))?;

    // Drop the lock before executing the query
    drop(pools);

    let rows = client
        .query(sql.as_str(), &[])
        .await
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut wtr = csv::Writer::from_path(&file_path)
        .map_err(|e| format!("Failed to create CSV file: {e}"))?;

    // Write header row
    if !rows.is_empty() {
        let headers: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();
        wtr.write_record(&headers)
            .map_err(|e| format!("Failed to write CSV header: {e}"))?;
    }

    // Write data rows
    let mut row_count: u64 = 0;
    for row in &rows {
        let values = row_to_json(row);
        let string_values: Vec<String> = values
            .iter()
            .map(|v| match v {
                serde_json::Value::Null => String::new(),
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .collect();
        wtr.write_record(&string_values)
            .map_err(|e| format!("Failed to write CSV row: {e}"))?;
        row_count += 1;
    }

    wtr.flush()
        .map_err(|e| format!("Failed to flush CSV: {e}"))?;

    Ok(format!(
        "Exported {} rows to {}",
        row_count, file_path
    ))
}

#[tauri::command]
pub async fn pg_export_json(
    id: String,
    sql: String,
    file_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| format!("Pool error: {e}"))?;

    // Drop the lock before executing the query
    drop(pools);

    let rows = client
        .query(sql.as_str(), &[])
        .await
        .map_err(|e| format!("Query failed: {e}"))?;

    // Build JSON array of objects
    let mut json_rows: Vec<serde_json::Value> = Vec::new();
    for row in &rows {
        let values = row_to_json(row);
        let columns = row.columns();
        let mut obj = serde_json::Map::new();
        for (i, val) in values.into_iter().enumerate() {
            obj.insert(columns[i].name().to_string(), val);
        }
        json_rows.push(serde_json::Value::Object(obj));
    }

    let json_str = serde_json::to_string_pretty(&json_rows)
        .map_err(|e| format!("JSON serialization failed: {e}"))?;

    std::fs::write(&file_path, json_str)
        .map_err(|e| format!("Failed to write JSON file: {e}"))?;

    Ok(format!(
        "Exported {} rows to {}",
        json_rows.len(),
        file_path
    ))
}
