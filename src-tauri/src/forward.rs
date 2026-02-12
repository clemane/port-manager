use serde::Serialize;
use sqlx::sqlite::SqlitePool;
use std::process::{Command, Stdio};
use tauri::State;

use crate::kubeconfig::get_kubeconfig_content;
use crate::AppState;

#[derive(Debug, Serialize, sqlx::FromRow, Clone)]
pub struct ActiveForward {
    pub id: String,
    pub favorite_id: Option<String>,
    pub kubeconfig_id: String,
    pub namespace: String,
    pub resource_type: String,
    pub resource_name: String,
    pub remote_port: i64,
    pub local_port: i64,
    pub pid: Option<i64>,
    pub status: String,
    pub started_at: Option<String>,
    pub error_msg: Option<String>,
}

#[tauri::command]
pub async fn create_forward(
    kubeconfig_id: String,
    namespace: String,
    resource_type: String,
    resource_name: String,
    remote_port: i64,
    local_port: Option<i64>,
    favorite_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<ActiveForward, String> {
    // Determine local port: use the provided one or find a free one
    let local = local_port.unwrap_or_else(|| find_free_port(&state.db));

    // Check if port is already in use
    if is_port_in_use(local as u16) {
        return Err(format!("Port {} is already in use", local));
    }

    // Write kubeconfig to a temp file for kubectl to consume
    let content = get_kubeconfig_content(&kubeconfig_id, &state.db).await?;
    let tmp_dir = std::env::temp_dir().join("port-manager");
    std::fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
    let kubeconfig_path = tmp_dir.join(format!("kubeconfig-{}.yaml", kubeconfig_id));
    std::fs::write(&kubeconfig_path, &content).map_err(|e| e.to_string())?;

    // Build and spawn the kubectl port-forward command
    let resource = format!("{}/{}", resource_type, resource_name);
    let port_mapping = format!("{}:{}", local, remote_port);

    let child = Command::new("kubectl")
        .arg("port-forward")
        .arg("-n")
        .arg(&namespace)
        .arg(&resource)
        .arg(&port_mapping)
        .env("KUBECONFIG", &kubeconfig_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start kubectl: {}", e))?;

    let pid = child.id() as i64;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Persist the forward record in the database
    sqlx::query(
        "INSERT INTO active_forwards (id, favorite_id, kubeconfig_id, namespace, resource_type, resource_name, remote_port, local_port, pid, status, started_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'running', ?)"
    )
    .bind(&id)
    .bind(&favorite_id)
    .bind(&kubeconfig_id)
    .bind(&namespace)
    .bind(&resource_type)
    .bind(&resource_name)
    .bind(remote_port)
    .bind(local)
    .bind(pid)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(ActiveForward {
        id,
        favorite_id,
        kubeconfig_id,
        namespace,
        resource_type,
        resource_name,
        remote_port,
        local_port: local,
        pid: Some(pid),
        status: "running".to_string(),
        started_at: Some(now),
        error_msg: None,
    })
}

#[tauri::command]
pub async fn kill_forward(id: String, state: State<'_, AppState>) -> Result<(), String> {
    // Retrieve the PID from the database
    let row: Option<(Option<i64>,)> =
        sqlx::query_as("SELECT pid FROM active_forwards WHERE id = ?")
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| e.to_string())?;

    if let Some((Some(pid),)) = row {
        // Send SIGTERM to the kubectl process
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
    }

    // Mark the forward as stopped in the database
    sqlx::query("UPDATE active_forwards SET status = 'stopped', pid = NULL WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn restart_forward(id: String, state: State<'_, AppState>) -> Result<ActiveForward, String> {
    // Fetch the existing forward details
    let forward: ActiveForward = sqlx::query_as(
        "SELECT id, favorite_id, kubeconfig_id, namespace, resource_type, resource_name, remote_port, local_port, pid, status, started_at, error_msg FROM active_forwards WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    // Kill the existing process if it is still running
    if let Some(pid) = forward.pid {
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
    }

    // Remove the old record
    sqlx::query("DELETE FROM active_forwards WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    // Create a fresh forward with the same parameters
    create_forward(
        forward.kubeconfig_id,
        forward.namespace,
        forward.resource_type,
        forward.resource_name,
        forward.remote_port,
        Some(forward.local_port),
        forward.favorite_id,
        state,
    )
    .await
}

#[tauri::command]
pub async fn list_forwards(state: State<'_, AppState>) -> Result<Vec<ActiveForward>, String> {
    let forwards: Vec<ActiveForward> = sqlx::query_as(
        "SELECT id, favorite_id, kubeconfig_id, namespace, resource_type, resource_name, remote_port, local_port, pid, status, started_at, error_msg FROM active_forwards ORDER BY started_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(forwards)
}

/// Find forwards that were marked as "running" from a previous session (stale)
/// and mark them as "stopped". Processes are dead after an app restart, so their
/// PIDs are no longer valid.
pub async fn cleanup_stale_forwards(pool: &SqlitePool) -> Result<Vec<ActiveForward>, String> {
    // Find forwards that were "running" (stale from previous session)
    let stale: Vec<ActiveForward> = sqlx::query_as(
        "SELECT id, favorite_id, kubeconfig_id, namespace, resource_type, resource_name, remote_port, local_port, pid, status, started_at, error_msg FROM active_forwards WHERE status = 'running'"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // Mark them all as stopped (processes are dead after app restart)
    sqlx::query("UPDATE active_forwards SET status = 'stopped', pid = NULL WHERE status = 'running'")
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(stale)
}

/// Check whether a given TCP port is already bound on localhost.
fn is_port_in_use(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_err()
}

/// Find the first available port in the 3000..4000 range.
/// Falls back to OS-assigned port if the entire range is occupied.
fn find_free_port(_pool: &SqlitePool) -> i64 {
    for port in 3000..4000 {
        if !is_port_in_use(port) {
            return port as i64;
        }
    }
    // Fallback: let the OS assign an ephemeral port
    if let Ok(listener) = std::net::TcpListener::bind("127.0.0.1:0") {
        if let Ok(addr) = listener.local_addr() {
            return addr.port() as i64;
        }
    }
    8080
}
