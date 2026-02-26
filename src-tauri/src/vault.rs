use crate::auth::VaultState;
use crate::file_activator;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSecret {
    pub id: String,
    pub name: String,
    pub category: String,
    pub file_path: Option<String>,
    pub notes: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// List all secrets in the vault (without the encrypted content blobs).
#[tauri::command]
pub fn list_vault_secrets(
    state: tauri::State<'_, VaultState>,
) -> Result<Vec<VaultSecret>, String> {
    state.vault.with_conn(|conn| {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, category, file_path, notes, is_active, created_at, updated_at
                 FROM vault_secrets ORDER BY created_at DESC",
            )
            .map_err(|e| format!("Query error: {e}"))?;

        let secrets = stmt
            .query_map([], |row| {
                Ok(VaultSecret {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    file_path: row.get(3)?,
                    notes: row.get(4)?,
                    is_active: row.get::<_, i32>(5)? != 0,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Query error: {e}"))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(secrets)
    })
}

/// Add a new secret to the vault. Returns the generated ID.
#[tauri::command]
pub fn add_vault_secret(
    name: String,
    category: String,
    content: Vec<u8>,
    file_path: Option<String>,
    notes: Option<String>,
    state: tauri::State<'_, VaultState>,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();

    state.vault.with_conn(|conn| {
        conn.execute(
            "INSERT INTO vault_secrets (id, name, category, content, file_path, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, name, category, content, file_path, notes],
        )
        .map_err(|e| format!("Insert error: {e}"))?;

        Ok(id.clone())
    })
}

/// Update an existing secret. Only non-`None` fields are updated.
#[tauri::command]
pub fn update_vault_secret(
    id: String,
    name: Option<String>,
    category: Option<String>,
    content: Option<Vec<u8>>,
    file_path: Option<String>,
    notes: Option<String>,
    state: tauri::State<'_, VaultState>,
) -> Result<(), String> {
    state.vault.with_conn(|conn| {
        let mut sets: Vec<String> = vec![];
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

        if let Some(ref v) = name {
            sets.push("name = ?".to_string());
            params.push(Box::new(v.clone()));
        }
        if let Some(ref v) = category {
            sets.push("category = ?".to_string());
            params.push(Box::new(v.clone()));
        }
        if let Some(ref v) = content {
            sets.push("content = ?".to_string());
            params.push(Box::new(v.clone()));
        }
        if let Some(ref v) = file_path {
            sets.push("file_path = ?".to_string());
            params.push(Box::new(v.clone()));
        }
        if let Some(ref v) = notes {
            sets.push("notes = ?".to_string());
            params.push(Box::new(v.clone()));
        }

        if sets.is_empty() {
            return Ok(());
        }

        sets.push("updated_at = datetime('now')".to_string());
        params.push(Box::new(id.clone()));

        let sql = format!(
            "UPDATE vault_secrets SET {} WHERE id = ?",
            sets.join(", ")
        );

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        conn.execute(&sql, param_refs.as_slice())
            .map_err(|e| format!("Update error: {e}"))?;

        Ok(())
    })
}

/// Delete a secret. If it's currently active, deactivate (secure-delete) first.
#[tauri::command]
pub fn delete_vault_secret(
    id: String,
    state: tauri::State<'_, VaultState>,
) -> Result<(), String> {
    state.vault.with_conn(|conn| {
        // Check if active and has a file path
        let row: Option<(i32, Option<String>)> = conn
            .query_row(
                "SELECT is_active, file_path FROM vault_secrets WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        if let Some((is_active, Some(file_path))) = row {
            if is_active != 0 {
                file_activator::secure_delete(&file_path);
            }
        }

        conn.execute(
            "DELETE FROM vault_secrets WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| format!("Delete error: {e}"))?;

        Ok(())
    })
}

/// Activate a secret: read its content from the DB, write it to `file_path`, mark `is_active = 1`.
#[tauri::command]
pub fn activate_secret(
    id: String,
    state: tauri::State<'_, VaultState>,
) -> Result<(), String> {
    state.vault.with_conn(|conn| {
        let (content, file_path): (Vec<u8>, Option<String>) = conn
            .query_row(
                "SELECT content, file_path FROM vault_secrets WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| format!("Query error: {e}"))?;

        let path = file_path.ok_or_else(|| "Secret has no file_path set".to_string())?;

        file_activator::activate_file(&path, &content)?;

        conn.execute(
            "UPDATE vault_secrets SET is_active = 1, updated_at = datetime('now') WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| format!("Update error: {e}"))?;

        Ok(())
    })
}

/// Deactivate a secret: securely delete the file, mark `is_active = 0`.
#[tauri::command]
pub fn deactivate_secret(
    id: String,
    state: tauri::State<'_, VaultState>,
) -> Result<(), String> {
    state.vault.with_conn(|conn| {
        let file_path: Option<String> = conn
            .query_row(
                "SELECT file_path FROM vault_secrets WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Query error: {e}"))?;

        if let Some(ref path) = file_path {
            file_activator::secure_delete(path);
        }

        conn.execute(
            "UPDATE vault_secrets SET is_active = 0, updated_at = datetime('now') WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| format!("Update error: {e}"))?;

        Ok(())
    })
}

/// Deactivate all active secrets: secure-delete every file, mark all `is_active = 0`.
#[tauri::command]
pub fn deactivate_all_secrets(
    state: tauri::State<'_, VaultState>,
) -> Result<(), String> {
    state.vault.with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, file_path FROM vault_secrets WHERE is_active = 1 AND file_path IS NOT NULL")
            .map_err(|e| format!("Query error: {e}"))?;

        let pairs: Vec<(String, String)> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| format!("Query error: {e}"))?
            .filter_map(|r| r.ok())
            .collect();

        file_activator::deactivate_all(&pairs);

        conn.execute(
            "UPDATE vault_secrets SET is_active = 0, updated_at = datetime('now') WHERE is_active = 1",
            [],
        )
        .map_err(|e| format!("Update error: {e}"))?;

        Ok(())
    })
}
