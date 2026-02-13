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
