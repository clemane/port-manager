use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// Tracks a secret that was written to disk so we can clean up on lock.
pub struct ActiveSecretPath {
    pub id: String,
    pub file_path: String,
}

/// Wrapper around a SQLCipher-encrypted database (`vault.db`).
pub struct VaultDb {
    conn: Mutex<Option<Connection>>,
    db_path: PathBuf,
}

impl VaultDb {
    /// Create a new VaultDb handle. Does NOT open or create the database yet.
    pub fn new(app_dir: PathBuf) -> Self {
        Self {
            conn: Mutex::new(None),
            db_path: app_dir.join("vault.db"),
        }
    }

    /// Returns `true` if `vault.db` already exists on disk.
    pub fn exists(&self) -> bool {
        self.db_path.exists()
    }

    /// Create a brand-new vault database with the given SQLCipher key.
    ///
    /// * `password_hash` -- output of `crypto::hash_password`
    /// * `recovery_hash` -- output of `crypto::hash_password` for the recovery key
    /// * `salt` -- the raw 16-byte salt
    /// * `derived_key_hex` -- hex-encoded 32-byte key for SQLCipher PRAGMA
    pub fn create(
        &self,
        password_hash: &[u8],
        recovery_hash: &[u8],
        salt: &[u8],
        derived_key_hex: &str,
    ) -> Result<(), String> {
        // Ensure parent directory exists
        if let Some(parent) = self.db_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create vault directory: {e}"))?;
        }

        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to create vault.db: {e}"))?;

        // Set SQLCipher encryption key
        conn.execute_batch(&format!("PRAGMA key = \"x'{derived_key_hex}'\";"))
            .map_err(|e| format!("Failed to set SQLCipher key: {e}"))?;

        // Run migrations
        Self::run_migrations(&conn)?;

        // Insert auth row
        conn.execute(
            "INSERT INTO vault_auth (id, password_hash, recovery_hash, salt) VALUES (1, ?1, ?2, ?3)",
            rusqlite::params![password_hash, recovery_hash, salt],
        )
        .map_err(|e| format!("Failed to insert auth row: {e}"))?;

        let mut guard = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        *guard = Some(conn);
        Ok(())
    }

    /// Open an existing vault database with the given SQLCipher key.
    pub fn open(&self, derived_key_hex: &str) -> Result<(), String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open vault.db: {e}"))?;

        // Set SQLCipher encryption key
        conn.execute_batch(&format!("PRAGMA key = \"x'{derived_key_hex}'\";"))
            .map_err(|e| format!("Failed to set SQLCipher key: {e}"))?;

        // Verify the key is correct by querying sqlite_master
        conn.query_row("SELECT count(*) FROM sqlite_master;", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|_| "Invalid vault password".to_string())?;

        // Run any pending migrations
        Self::run_migrations(&conn)?;

        let mut guard = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        *guard = Some(conn);
        Ok(())
    }

    /// Lock the vault: collect active secret file paths, mark them inactive, close the connection.
    ///
    /// Returns the list of active secrets (id, file_path) so the caller can securely delete
    /// the files.
    pub fn lock(&self) -> Result<Vec<ActiveSecretPath>, String> {
        let mut guard = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;

        let paths = {
            let conn = match guard.as_ref() {
                Some(c) => c,
                None => return Ok(vec![]),
            };

            // Collect active secrets that have file paths
            let mut stmt = conn
                .prepare("SELECT id, file_path FROM vault_secrets WHERE is_active = 1 AND file_path IS NOT NULL")
                .map_err(|e| format!("Query error: {e}"))?;

            let paths: Vec<ActiveSecretPath> = stmt
                .query_map([], |row| {
                    Ok(ActiveSecretPath {
                        id: row.get(0)?,
                        file_path: row.get(1)?,
                    })
                })
                .map_err(|e| format!("Query error: {e}"))?
                .filter_map(|r| r.ok())
                .collect();

            // Mark all as inactive
            conn.execute("UPDATE vault_secrets SET is_active = 0", [])
                .map_err(|e| format!("Update error: {e}"))?;

            paths
        };

        // Close the connection (immutable borrows are now dropped)
        *guard = None;

        Ok(paths)
    }

    /// Execute a closure with a reference to the open database connection.
    ///
    /// Returns an error if the vault is locked (not open).
    pub fn with_conn<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&Connection) -> Result<T, String>,
    {
        let guard = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        match guard.as_ref() {
            Some(conn) => f(conn),
            None => Err("Vault is locked".to_string()),
        }
    }

    /// Write the raw salt bytes to a `vault.salt` file alongside the DB.
    pub fn write_salt(&self, salt: &[u8]) -> Result<(), String> {
        let salt_path = self.salt_path();
        if let Some(parent) = salt_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory for salt file: {e}"))?;
        }
        fs::write(&salt_path, salt).map_err(|e| format!("Failed to write salt file: {e}"))
    }

    /// Read the raw salt bytes from the `vault.salt` file.
    pub fn read_salt(&self) -> Result<Vec<u8>, String> {
        let salt_path = self.salt_path();
        fs::read(&salt_path).map_err(|e| format!("Failed to read salt file: {e}"))
    }

    /// Write recovery data to `vault.recovery` (unencrypted file).
    ///
    /// Format: recovery_salt (16) || recovery_hash (48) || encrypted_db_key (variable)
    pub fn write_recovery(
        &self,
        recovery_salt: &[u8],
        recovery_hash: &[u8],
        encrypted_db_key: &[u8],
    ) -> Result<(), String> {
        let recovery_path = self.recovery_path();
        let mut data = Vec::new();
        data.extend_from_slice(recovery_salt);   // 16 bytes
        data.extend_from_slice(recovery_hash);   // 48 bytes
        data.extend_from_slice(encrypted_db_key); // variable
        fs::write(&recovery_path, &data)
            .map_err(|e| format!("Failed to write recovery file: {e}"))
    }

    /// Read recovery data from `vault.recovery`.
    ///
    /// Returns (recovery_salt, recovery_hash, encrypted_db_key).
    pub fn read_recovery(&self) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), String> {
        let recovery_path = self.recovery_path();
        let data = fs::read(&recovery_path)
            .map_err(|e| format!("Failed to read recovery file: {e}"))?;
        if data.len() < 64 {
            return Err("Recovery file is corrupted".to_string());
        }
        let recovery_salt = data[..16].to_vec();
        let recovery_hash = data[16..64].to_vec();
        let encrypted_db_key = data[64..].to_vec();
        Ok((recovery_salt, recovery_hash, encrypted_db_key))
    }

    /// Destroy the vault: close the connection, delete vault.db and vault.salt.
    pub fn destroy(&self) -> Result<(), String> {
        // Close connection if open
        let mut guard = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        *guard = None;

        // Delete vault.db
        if self.db_path.exists() {
            fs::remove_file(&self.db_path)
                .map_err(|e| format!("Failed to delete vault.db: {e}"))?;
        }

        // Delete vault.salt
        let salt_path = self.salt_path();
        if salt_path.exists() {
            fs::remove_file(&salt_path)
                .map_err(|e| format!("Failed to delete vault.salt: {e}"))?;
        }

        // Delete vault.recovery
        let recovery_path = self.recovery_path();
        if recovery_path.exists() {
            fs::remove_file(&recovery_path)
                .map_err(|e| format!("Failed to delete vault.recovery: {e}"))?;
        }

        Ok(())
    }

    // ── Private helpers ────────────────────────────────────────────────

    fn salt_path(&self) -> PathBuf {
        self.db_path.with_file_name("vault.salt")
    }

    fn recovery_path(&self) -> PathBuf {
        self.db_path.with_file_name("vault.recovery")
    }

    fn run_migrations(conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS vault_auth (
                id INTEGER PRIMARY KEY DEFAULT 1,
                password_hash BLOB NOT NULL,
                recovery_hash BLOB NOT NULL,
                salt BLOB NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS vault_secrets (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                category TEXT NOT NULL CHECK(category IN ('kubeconfig','ssh_key','token','certificate','password','other')),
                content BLOB NOT NULL,
                file_path TEXT,
                notes TEXT,
                is_active INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            ",
        )
        .map_err(|e| format!("Migration error: {e}"))
    }
}
