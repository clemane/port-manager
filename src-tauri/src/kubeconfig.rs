use serde::Serialize;
use sqlx::sqlite::SqlitePool;
use tauri::State;

use crate::crypto::{get_or_create_encryption_key, get_encryption_key, secure_encrypt, secure_decrypt};
use crate::AppState;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct KubeconfigInfo {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub last_used: Option<String>,
}

/// Validate that content is valid YAML (specifically a kubeconfig structure).
fn validate_kubeconfig_yaml(content: &str) -> Result<(), String> {
    let value: serde_yaml::Value = serde_yaml::from_str(content)
        .map_err(|e| format!("Invalid YAML: {e}"))?;

    // Basic sanity check: must be a mapping with apiVersion
    let map = value.as_mapping().ok_or("Kubeconfig must be a YAML mapping")?;
    let has_api_version = map.keys().any(|k| {
        k.as_str().map(|s| s == "apiVersion").unwrap_or(false)
    });

    if !has_api_version {
        return Err("Invalid kubeconfig: missing 'apiVersion' field".to_string());
    }

    Ok(())
}

/// Import a kubeconfig by encrypting its content and storing it in the database.
///
/// Validates YAML structure before storing. Uses AES-256-GCM encryption.
/// Returns the generated ID for the new kubeconfig entry.
#[tauri::command]
pub async fn import_kubeconfig(
    name: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Validate YAML before storing
    validate_kubeconfig_yaml(&content)?;

    let id = uuid::Uuid::new_v4().to_string();
    let key = get_or_create_encryption_key()?;
    let encrypted = secure_encrypt(content.as_bytes(), &key)?;

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
/// Uses AES-256-GCM decryption with automatic fallback to legacy XOR for
/// data encrypted before the migration. Legacy data is transparently
/// re-encrypted with AES-256-GCM on successful decryption.
///
/// Validates that the decrypted content is valid UTF-8 and valid YAML.
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
    let (decrypted, needs_migration) = secure_decrypt(&row.0, &key)?;

    let content = String::from_utf8(decrypted)
        .map_err(|_| {
            "Decrypted kubeconfig is not valid UTF-8. \
             The encryption key may have changed — please re-import this kubeconfig."
                .to_string()
        })?;

    // Validate YAML structure
    validate_kubeconfig_yaml(&content).map_err(|e| {
        format!(
            "Decrypted content is not a valid kubeconfig: {e}. \
             The encryption key may have changed — please re-import this kubeconfig."
        )
    })?;

    // Transparently migrate legacy XOR data to AES-256-GCM
    if needs_migration {
        if let Ok(re_encrypted) = secure_encrypt(content.as_bytes(), &key) {
            let _ = sqlx::query("UPDATE kubeconfigs SET content = ? WHERE id = ?")
                .bind(&re_encrypted)
                .bind(id)
                .execute(pool)
                .await;
        }
    }

    Ok(content)
}
