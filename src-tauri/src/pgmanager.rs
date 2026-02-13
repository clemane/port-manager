use serde::{Deserialize, Serialize};
use tauri::State;

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
