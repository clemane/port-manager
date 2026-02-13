use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::process::{Command, Stdio};
use tauri::State;

use crate::AppState;

// ── Models ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, sqlx::FromRow, Clone)]
pub struct NgrokDomain {
    pub id: String,
    pub domain: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow, Clone)]
pub struct NgrokTunnel {
    pub id: String,
    pub domain_id: Option<String>,
    pub domain: String,
    pub local_port: i64,
    pub pid: Option<i64>,
    pub status: String,
    pub tunnel_url: Option<String>,
    pub started_at: Option<String>,
    pub error_msg: Option<String>,
}

// ── Local API models (http://127.0.0.1:4040) ───────────────────────────

#[derive(Debug, Deserialize)]
struct NgrokLocalApiResponse {
    tunnels: Vec<NgrokLocalApiTunnel>,
}

#[derive(Debug, Deserialize)]
struct NgrokLocalApiTunnel {
    public_url: String,
    config: NgrokLocalApiTunnelConfig,
}

#[derive(Debug, Deserialize)]
struct NgrokLocalApiTunnelConfig {
    addr: String,
}

// ── Cloud API models (https://api.ngrok.com) ────────────────────────────

#[derive(Debug, Deserialize)]
struct NgrokCloudDomainsResponse {
    reserved_domains: Vec<NgrokCloudDomain>,
}

#[derive(Debug, Deserialize)]
struct NgrokCloudDomain {
    domain: String,
}

// ── Domain CRUD ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn add_ngrok_domain(
    domain: String,
    state: State<'_, AppState>,
) -> Result<NgrokDomain, String> {
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO ngrok_domains (id, domain) VALUES (?, ?)")
        .bind(&id)
        .bind(&domain)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let row: NgrokDomain =
        sqlx::query_as("SELECT id, domain, created_at FROM ngrok_domains WHERE id = ?")
            .bind(&id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| e.to_string())?;

    Ok(row)
}

#[tauri::command]
pub async fn list_ngrok_domains(
    state: State<'_, AppState>,
) -> Result<Vec<NgrokDomain>, String> {
    let domains: Vec<NgrokDomain> = sqlx::query_as(
        "SELECT id, domain, created_at FROM ngrok_domains ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(domains)
}

#[tauri::command]
pub async fn delete_ngrok_domain(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM ngrok_domains WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ── Tunnel management ───────────────────────────────────────────────────

#[tauri::command]
pub async fn create_tunnel(
    domain_id: String,
    local_port: i64,
    state: State<'_, AppState>,
) -> Result<NgrokTunnel, String> {
    // Resolve domain from domain_id
    let domain_row: NgrokDomain = sqlx::query_as(
        "SELECT id, domain, created_at FROM ngrok_domains WHERE id = ?",
    )
    .bind(&domain_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Domain not found: {}", e))?;

    // Read authtoken from settings
    let token_row: Option<(String,)> =
        sqlx::query_as("SELECT value FROM settings WHERE key = 'ngrok_authtoken'")
            .fetch_optional(&state.db)
            .await
            .map_err(|e| e.to_string())?;

    let authtoken = token_row
        .map(|r| r.0)
        .ok_or_else(|| "Ngrok authtoken not configured. Set it in Settings.".to_string())?;

    if authtoken.trim().is_empty() {
        return Err("Ngrok authtoken is empty. Set it in Settings.".to_string());
    }

    // Spawn ngrok process
    let child = Command::new("ngrok")
        .arg("http")
        .arg(format!("--domain={}", domain_row.domain))
        .arg(format!("--authtoken={}", authtoken.trim()))
        .arg(local_port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start ngrok: {}", e))?;

    let pid = child.id() as i64;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO ngrok_tunnels (id, domain_id, domain, local_port, pid, status, started_at) VALUES (?, ?, ?, ?, ?, 'running', ?)"
    )
    .bind(&id)
    .bind(&domain_id)
    .bind(&domain_row.domain)
    .bind(local_port)
    .bind(pid)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    // Spawn async task to poll ngrok local API for the tunnel URL
    let pool = state.db.clone();
    let tunnel_id = id.clone();
    let port = local_port;
    tokio::spawn(async move {
        resolve_tunnel_url(&pool, &tunnel_id, port).await;
    });

    Ok(NgrokTunnel {
        id,
        domain_id: Some(domain_id),
        domain: domain_row.domain,
        local_port,
        pid: Some(pid),
        status: "running".to_string(),
        tunnel_url: None,
        started_at: Some(now),
        error_msg: None,
    })
}

/// Poll the ngrok local API up to 10 times (with 1s delay) to discover the tunnel URL.
async fn resolve_tunnel_url(pool: &SqlitePool, tunnel_id: &str, local_port: i64) {
    let client = reqwest::Client::new();
    let addr_suffix = format!(":{}", local_port);

    for _ in 0..10 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let resp = client
            .get("http://127.0.0.1:4040/api/tunnels")
            .send()
            .await;

        if let Ok(resp) = resp {
            if let Ok(api) = resp.json::<NgrokLocalApiResponse>().await {
                // Find the tunnel whose config.addr ends with our local port
                if let Some(t) = api
                    .tunnels
                    .iter()
                    .find(|t| t.config.addr.ends_with(&addr_suffix))
                {
                    let _ = sqlx::query(
                        "UPDATE ngrok_tunnels SET tunnel_url = ? WHERE id = ?",
                    )
                    .bind(&t.public_url)
                    .bind(tunnel_id)
                    .execute(pool)
                    .await;
                    return;
                }
            }
        }
    }
}

#[tauri::command]
pub async fn kill_tunnel(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let row: Option<(Option<i64>,)> =
        sqlx::query_as("SELECT pid FROM ngrok_tunnels WHERE id = ?")
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| e.to_string())?;

    if let Some((Some(pid),)) = row {
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
    }

    sqlx::query("UPDATE ngrok_tunnels SET status = 'stopped', pid = NULL WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn restart_tunnel(id: String, state: State<'_, AppState>) -> Result<NgrokTunnel, String> {
    let tunnel: NgrokTunnel = sqlx::query_as(
        "SELECT id, domain_id, domain, local_port, pid, status, tunnel_url, started_at, error_msg FROM ngrok_tunnels WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Tunnel not found: {}", e))?;

    // Kill old process if still alive
    if let Some(pid) = tunnel.pid {
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
    }

    // Remove old record
    sqlx::query("DELETE FROM ngrok_tunnels WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    // Resolve domain_id: use existing one, or look up by domain name
    let domain_id = match tunnel.domain_id {
        Some(ref did) => {
            let exists: Option<(String,)> =
                sqlx::query_as("SELECT id FROM ngrok_domains WHERE id = ?")
                    .bind(did)
                    .fetch_optional(&state.db)
                    .await
                    .map_err(|e| e.to_string())?;
            if exists.is_some() {
                did.clone()
            } else {
                // domain_id was deleted, look up by name
                let row: Option<(String,)> =
                    sqlx::query_as("SELECT id FROM ngrok_domains WHERE domain = ?")
                        .bind(&tunnel.domain)
                        .fetch_optional(&state.db)
                        .await
                        .map_err(|e| e.to_string())?;
                row.map(|r| r.0).ok_or_else(|| format!("Domain '{}' not found. Add it first.", tunnel.domain))?
            }
        }
        None => {
            let row: Option<(String,)> =
                sqlx::query_as("SELECT id FROM ngrok_domains WHERE domain = ?")
                    .bind(&tunnel.domain)
                    .fetch_optional(&state.db)
                    .await
                    .map_err(|e| e.to_string())?;
            row.map(|r| r.0).ok_or_else(|| format!("Domain '{}' not found. Add it first.", tunnel.domain))?
        }
    };

    create_tunnel(domain_id, tunnel.local_port, state).await
}

#[tauri::command]
pub async fn list_tunnels(state: State<'_, AppState>) -> Result<Vec<NgrokTunnel>, String> {
    let mut tunnels: Vec<NgrokTunnel> = sqlx::query_as(
        "SELECT id, domain_id, domain, local_port, pid, status, tunnel_url, started_at, error_msg FROM ngrok_tunnels ORDER BY started_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    // Check liveness of running tunnels via kill(pid, 0)
    let mut dead_ids: Vec<String> = Vec::new();
    for tunnel in &mut tunnels {
        if tunnel.status == "running" {
            if let Some(pid) = tunnel.pid {
                let alive = unsafe { libc::kill(pid as i32, 0) } == 0;
                if !alive {
                    tunnel.status = "stopped".to_string();
                    tunnel.pid = None;
                    dead_ids.push(tunnel.id.clone());
                }
            } else {
                tunnel.status = "stopped".to_string();
                dead_ids.push(tunnel.id.clone());
            }
        }
    }

    // Update dead tunnels in DB
    for dead_id in &dead_ids {
        let _ = sqlx::query(
            "UPDATE ngrok_tunnels SET status = 'stopped', pid = NULL WHERE id = ?",
        )
        .bind(dead_id)
        .execute(&state.db)
        .await;
    }

    Ok(tunnels)
}

// ── Sync domains from ngrok cloud API ───────────────────────────────────

#[tauri::command]
pub async fn sync_ngrok_domains(
    state: State<'_, AppState>,
) -> Result<Vec<NgrokDomain>, String> {
    // Read API key from settings
    let key_row: Option<(String,)> =
        sqlx::query_as("SELECT value FROM settings WHERE key = 'ngrok_api_key'")
            .fetch_optional(&state.db)
            .await
            .map_err(|e| e.to_string())?;

    let api_key = key_row
        .map(|r| r.0)
        .ok_or_else(|| "Ngrok API key not configured. Set it in Settings.".to_string())?;

    if api_key.trim().is_empty() {
        return Err("Ngrok API key is empty. Set it in Settings.".to_string());
    }

    // Fetch reserved domains from ngrok cloud API
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.ngrok.com/reserved_domains")
        .bearer_auth(api_key.trim())
        .header("Ngrok-Version", "2")
        .send()
        .await
        .map_err(|e| format!("Failed to call ngrok API: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Ngrok API error ({}): {}", status, body));
    }

    let api_resp: NgrokCloudDomainsResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse ngrok API response: {}", e))?;

    // Upsert each domain into ngrok_domains
    for cloud_domain in &api_resp.reserved_domains {
        let exists: Option<(String,)> =
            sqlx::query_as("SELECT id FROM ngrok_domains WHERE domain = ?")
                .bind(&cloud_domain.domain)
                .fetch_optional(&state.db)
                .await
                .map_err(|e| e.to_string())?;

        if exists.is_none() {
            let id = uuid::Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO ngrok_domains (id, domain) VALUES (?, ?)")
                .bind(&id)
                .bind(&cloud_domain.domain)
                .execute(&state.db)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    // Return updated list
    let domains: Vec<NgrokDomain> = sqlx::query_as(
        "SELECT id, domain, created_at FROM ngrok_domains ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(domains)
}

// ── Detect running tunnels from local ngrok agent ───────────────────────

#[tauri::command]
pub async fn detect_running_tunnels(
    state: State<'_, AppState>,
) -> Result<Vec<NgrokTunnel>, String> {
    let client = reqwest::Client::new();

    let resp = client
        .get("http://127.0.0.1:4040/api/tunnels")
        .send()
        .await
        .map_err(|_| "Cannot reach ngrok local API at 127.0.0.1:4040. Is ngrok running?".to_string())?;

    let api: NgrokLocalApiResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse local ngrok API: {}", e))?;

    let now = chrono::Utc::now().to_rfc3339();

    for tunnel in &api.tunnels {
        // Only process https tunnels (skip http duplicates)
        if !tunnel.public_url.starts_with("https://") {
            continue;
        }

        // Extract domain from public_url: "https://xxx.ngrok-free.app" -> "xxx.ngrok-free.app"
        let domain = tunnel.public_url.trim_start_matches("https://");

        // Extract local port from config.addr: "http://localhost:3000" or "localhost:3000"
        let local_port: i64 = tunnel
            .config
            .addr
            .rsplit(':')
            .next()
            .and_then(|p| p.parse().ok())
            .unwrap_or(0);

        if local_port == 0 {
            continue;
        }

        // Check if we already track a running tunnel for this domain + port
        let existing: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM ngrok_tunnels WHERE domain = ? AND local_port = ? AND status = 'running'",
        )
        .bind(domain)
        .bind(local_port)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?;

        if existing.is_some() {
            continue;
        }

        // Look up domain_id if we have this domain registered
        let domain_row: Option<(String,)> =
            sqlx::query_as("SELECT id FROM ngrok_domains WHERE domain = ?")
                .bind(domain)
                .fetch_optional(&state.db)
                .await
                .map_err(|e| e.to_string())?;

        let domain_id = domain_row.map(|r| r.0);

        // Insert as running tunnel (no PID since we didn't spawn it)
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO ngrok_tunnels (id, domain_id, domain, local_port, pid, status, tunnel_url, started_at) VALUES (?, ?, ?, ?, NULL, 'running', ?, ?)"
        )
        .bind(&id)
        .bind(&domain_id)
        .bind(domain)
        .bind(local_port)
        .bind(&tunnel.public_url)
        .bind(&now)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    }

    // Return updated tunnel list
    list_tunnels_internal(&state.db).await
}

async fn list_tunnels_internal(pool: &SqlitePool) -> Result<Vec<NgrokTunnel>, String> {
    let tunnels: Vec<NgrokTunnel> = sqlx::query_as(
        "SELECT id, domain_id, domain, local_port, pid, status, tunnel_url, started_at, error_msg FROM ngrok_tunnels ORDER BY started_at DESC"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(tunnels)
}

// ── Startup cleanup ─────────────────────────────────────────────────────

pub async fn cleanup_stale_tunnels(pool: &SqlitePool) -> Result<Vec<NgrokTunnel>, String> {
    let stale: Vec<NgrokTunnel> = sqlx::query_as(
        "SELECT id, domain_id, domain, local_port, pid, status, tunnel_url, started_at, error_msg FROM ngrok_tunnels WHERE status = 'running'"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query("UPDATE ngrok_tunnels SET status = 'stopped', pid = NULL WHERE status = 'running'")
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(stale)
}
