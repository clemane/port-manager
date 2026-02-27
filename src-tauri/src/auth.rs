use crate::crypto;
use crate::file_activator;
use crate::vault_db::VaultDb;
use serde::Serialize;
use std::sync::Mutex;

/// Shared application state for the vault subsystem.
pub struct VaultState {
    pub vault: VaultDb,
    pub session_key: Mutex<Option<String>>,
}

#[derive(Serialize)]
pub struct VaultStatus {
    pub exists: bool,
    pub unlocked: bool,
}

/// Check whether the vault exists and whether it is currently unlocked.
#[tauri::command]
pub fn vault_status(state: tauri::State<'_, VaultState>) -> VaultStatus {
    let unlocked = state
        .session_key
        .lock()
        .map(|g| g.is_some())
        .unwrap_or(false);

    VaultStatus {
        exists: state.vault.exists(),
        unlocked,
    }
}

/// Create a new master password for the vault.
///
/// Returns the recovery key (must be shown to the user once).
#[tauri::command]
pub fn create_master_password(
    password: String,
    state: tauri::State<'_, VaultState>,
) -> Result<String, String> {
    // 1. Generate salt and derive SQLCipher key
    let salt = crypto::generate_salt();
    let derived = crypto::derive_key(password.as_bytes(), &salt)?;
    let derived_hex = hex::encode(derived);

    // 2. Hash the password for verification storage
    let password_hash = crypto::hash_password(&password)?;

    // 3. Generate recovery key
    let recovery_key = crypto::generate_recovery_key();
    let recovery_hash = crypto::hash_password(&recovery_key)?;

    // 4. Encrypt the SQLCipher key with the recovery key (for recovery flow)
    let recovery_salt = crypto::generate_salt();
    let recovery_enc_key = crypto::derive_key(recovery_key.as_bytes(), &recovery_salt)?;
    let encrypted_db_key = crypto::encrypt(derived_hex.as_bytes(), &recovery_enc_key)?;

    // 5. Write salt file (needed before we can open the DB later)
    state.vault.write_salt(&salt)?;

    // 6. Write recovery file (outside encrypted DB)
    let recovery_verify_hash = crypto::hash_password(&recovery_key)?;
    state.vault.write_recovery(&recovery_salt, &recovery_verify_hash, &encrypted_db_key)?;

    // 7. Create the encrypted vault database
    state
        .vault
        .create(&password_hash, &recovery_hash, &salt, &derived_hex)?;

    // 8. Store the session key so the vault is "unlocked"
    let mut session = state
        .session_key
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    *session = Some(derived_hex);

    Ok(recovery_key)
}

/// Attempt to unlock the vault with the given password.
///
/// Returns `true` on success.
#[tauri::command]
pub fn login(password: String, state: tauri::State<'_, VaultState>) -> Result<bool, String> {
    // 1. Read salt
    let salt = state.vault.read_salt()?;

    // 2. Derive the SQLCipher key
    let derived = crypto::derive_key(password.as_bytes(), &salt)?;
    let derived_hex = hex::encode(derived);

    // 3. Try to open the database
    match state.vault.open(&derived_hex) {
        Ok(()) => {
            let mut session = state
                .session_key
                .lock()
                .map_err(|e| format!("Lock error: {e}"))?;
            *session = Some(derived_hex);
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

/// Recover the vault using the recovery key.
///
/// Verifies the recovery key, decrypts the stored DB key, and unlocks the vault.
#[tauri::command]
pub fn recover_vault(
    recovery_key: String,
    state: tauri::State<'_, VaultState>,
) -> Result<bool, String> {
    // 1. Read recovery data
    let (recovery_salt, recovery_hash, encrypted_db_key) = state.vault.read_recovery()?;

    // 2. Verify the recovery key
    if !crypto::verify_password(&recovery_key, &recovery_hash)? {
        return Ok(false);
    }

    // 3. Derive the recovery encryption key and decrypt the DB key
    let recovery_enc_key = crypto::derive_key(recovery_key.as_bytes(), &recovery_salt)?;
    let db_key_bytes = crypto::decrypt(&encrypted_db_key, &recovery_enc_key)?;
    let derived_hex = String::from_utf8(db_key_bytes)
        .map_err(|_| "Failed to decode DB key".to_string())?;

    // 4. Open the vault
    state.vault.open(&derived_hex)?;

    // 5. Store session key
    let mut session = state.session_key.lock().map_err(|e| format!("Lock error: {e}"))?;
    *session = Some(derived_hex);

    Ok(true)
}

/// Lock the vault: securely delete all active secret files and close the DB.
#[tauri::command]
pub fn lock_vault(state: tauri::State<'_, VaultState>) -> Result<(), String> {
    // 1. Lock vault DB (returns list of active file paths)
    let active_paths = state.vault.lock()?;

    // 2. Securely delete each active file
    let pairs: Vec<(String, String)> = active_paths
        .into_iter()
        .map(|a| (a.id, a.file_path))
        .collect();
    file_activator::deactivate_all(&pairs);

    // 3. Clear session key
    let mut session = state
        .session_key
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?;
    *session = None;

    Ok(())
}

/// Expand `~` to `$HOME` in a path string.
pub fn expand_path(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}/{}", home, &path[2..]);
        }
    }
    path.to_string()
}
