/// Retrieve or generate the encryption key stored in the system keyring.
///
/// On first call, generates a random UUID-based key and persists it in the
/// platform keyring under service="port-manager", user="encryption-key".
/// Subsequent calls retrieve the same key.
pub fn get_encryption_key() -> Result<Vec<u8>, String> {
    let entry = keyring::Entry::new("port-manager", "encryption-key")
        .map_err(|e| format!("Failed to create keyring entry: {e}"))?;

    match entry.get_password() {
        Ok(key) => Ok(key.into_bytes()),
        Err(keyring::Error::NoEntry) => {
            // Generate a new random key and store it
            let key = uuid::Uuid::new_v4().to_string();
            entry
                .set_password(&key)
                .map_err(|e| format!("Failed to store encryption key in keyring: {e}"))?;
            Ok(key.into_bytes())
        }
        Err(e) => Err(format!("Failed to retrieve encryption key from keyring: {e}")),
    }
}

/// XOR-based encrypt/decrypt (symmetric). Repeats the key across the data.
pub fn encrypt_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect()
}
