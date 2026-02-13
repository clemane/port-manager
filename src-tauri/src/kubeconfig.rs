use serde::Serialize;
use sqlx::sqlite::SqlitePool;
use tauri::State;

use crate::crypto::{encrypt_decrypt, get_encryption_key};
use crate::AppState;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct KubeconfigInfo {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub last_used: Option<String>,
}

/// Import a kubeconfig by encrypting its content and storing it in the database.
/// Returns the generated ID for the new kubeconfig entry.
#[tauri::command]
pub async fn import_kubeconfig(
    name: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let key = get_encryption_key()?;
    let encrypted = encrypt_decrypt(content.as_bytes(), &key);

    sqlx::query("INSERT INTO kubeconfigs (id, name, content) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&name)
        .bind(&encrypted)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to insert kubeconfig: {e}"))?;

    Ok(id)
}

/// List all stored kubeconfigs (metadata only, no content).
#[tauri::command]
pub async fn list_kubeconfigs(
    state: State<'_, AppState>,
) -> Result<Vec<KubeconfigInfo>, String> {
    sqlx::query_as::<_, KubeconfigInfo>(
        "SELECT id, name, created_at, last_used FROM kubeconfigs ORDER BY name",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("Failed to list kubeconfigs: {e}"))
}

/// Delete a kubeconfig entry by ID.
#[tauri::command]
pub async fn delete_kubeconfig(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM kubeconfigs WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to delete kubeconfig: {e}"))?;

    Ok(())
}

/// Retrieve and decrypt the kubeconfig content for a given ID.
///
/// This is an internal helper (not a Tauri command) intended for use by
/// other modules such as `k8s.rs` when establishing port-forwards.
#[allow(dead_code)]
pub async fn get_kubeconfig_content(
    id: &str,
    pool: &SqlitePool,
) -> Result<String, String> {
    let row: (Vec<u8>,) = sqlx::query_as("SELECT content FROM kubeconfigs WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to fetch kubeconfig content: {e}"))?;

    let key = get_encryption_key()?;
    let decrypted = encrypt_decrypt(&row.0, &key);
    String::from_utf8(decrypted)
        .map_err(|e| format!("Decrypted kubeconfig is not valid UTF-8: {e}"))
}
