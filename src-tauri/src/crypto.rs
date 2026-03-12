use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use zeroize::Zeroize;

// ── New crypto primitives ──────────────────────────────────────────────

/// Derive a 32-byte key from a password and salt using Argon2id.
///
/// Params: memory = 64 MiB, time = 3 iterations, parallelism = 4.
pub fn derive_key(password: &[u8], salt: &[u8]) -> Result<[u8; 32], String> {
    let params = Params::new(65536, 3, 4, Some(32))
        .map_err(|e| format!("Argon2 params error: {e}"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password, salt, &mut key)
        .map_err(|e| format!("Argon2 hash error: {e}"))?;

    Ok(key)
}

/// Derive a 32-byte AES key from a legacy UUID keyring key.
///
/// Uses Argon2id with light params (16 MiB, 1 iteration) since the input
/// is already high-entropy random material (UUID v4).
pub fn derive_aes_key(uuid_key: &[u8]) -> Result<[u8; 32], String> {
    let salt = b"port-manager-aes"; // fixed 16-byte salt
    let params = Params::new(16384, 1, 1, Some(32))
        .map_err(|e| format!("Argon2 params error: {e}"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(uuid_key, salt, &mut key)
        .map_err(|e| format!("Key derivation error: {e}"))?;

    Ok(key)
}

/// Generate a random 16-byte salt.
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Encrypt `data` with AES-256-GCM.
///
/// Returns `nonce (12 bytes) || ciphertext+tag`.
pub fn encrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| format!("AES key init error: {e}"))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("AES encrypt error: {e}"))?;

    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt data produced by [`encrypt`]. Expects `nonce (12) || ciphertext+tag`.
pub fn decrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if data.len() < 12 {
        return Err("Ciphertext too short".to_string());
    }

    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| format!("AES key init error: {e}"))?;

    let nonce = Nonce::from_slice(&data[..12]);
    let plaintext = cipher
        .decrypt(nonce, &data[12..])
        .map_err(|e| format!("AES decrypt error: {e}"))?;

    Ok(plaintext)
}

/// Hash a password for storage. Returns `salt (16) || argon2_hash (32)`.
pub fn hash_password(password: &str) -> Result<Vec<u8>, String> {
    let salt = generate_salt();
    let mut hash = derive_key(password.as_bytes(), &salt)?;

    let mut result = Vec::with_capacity(48);
    result.extend_from_slice(&salt);
    result.extend_from_slice(&hash);

    hash.zeroize();
    Ok(result)
}

/// Verify a password against the output of [`hash_password`].
pub fn verify_password(password: &str, stored: &[u8]) -> Result<bool, String> {
    if stored.len() != 48 {
        return Err("Invalid stored hash length".to_string());
    }

    let salt = &stored[..16];
    let expected_hash = &stored[16..48];

    let mut computed = derive_key(password.as_bytes(), salt)?;
    let matches = constant_time_eq(&computed, expected_hash);

    computed.zeroize();
    Ok(matches)
}

/// Generate a human-readable recovery key: 12 random words joined by "-".
pub fn generate_recovery_key() -> String {
    const WORDS: &[&str] = &[
        "alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel", "india",
        "juliet", "kilo", "lima", "mike", "november", "oscar", "papa", "quebec", "romeo",
        "sierra", "tango", "uniform", "victor", "whiskey", "xray", "yankee", "zulu", "anchor",
        "beacon", "castle", "dragon", "ember", "falcon", "glacier", "harbor", "ivory", "jungle",
        "knight", "lantern", "marble", "nebula", "oracle", "phoenix", "quartz", "raven",
        "summit", "thunder", "umbra", "vortex", "willow", "zenith", "atlas", "blaze", "cipher",
        "drift", "epoch", "flame", "glyph", "haze", "ion", "jade", "karma", "lux", "mesa",
        "nova",
    ];

    let mut rng = rand::thread_rng();
    let words: Vec<&str> = (0..12)
        .map(|_| {
            let idx = (rng.next_u32() as usize) % WORDS.len();
            WORDS[idx]
        })
        .collect();

    words.join("-")
}

// ── Legacy functions (kept for migration / backward compatibility) ──────

/// Legacy XOR-based encrypt/decrypt. Kept for migrating existing data.
pub fn legacy_encrypt_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect()
}

/// Legacy keyring-based key retrieval. Kept for migrating existing data.
pub fn legacy_get_encryption_key() -> Result<Vec<u8>, String> {
    let entry = keyring::Entry::new("port-manager", "encryption-key")
        .map_err(|e| format!("Failed to create keyring entry: {e}"))?;

    match entry.get_password() {
        Ok(key) => Ok(key.into_bytes()),
        Err(keyring::Error::NoEntry) => {
            let key = uuid::Uuid::new_v4().to_string();
            entry
                .set_password(&key)
                .map_err(|e| format!("Failed to store encryption key in keyring: {e}"))?;
            Ok(key.into_bytes())
        }
        Err(e) => Err(format!(
            "Failed to retrieve encryption key from keyring: {e}"
        )),
    }
}

// ── Secure wrappers (used by kubeconfig.rs, pgmanager.rs) ───────────────

/// Retrieve the encryption key from the keyring.
///
/// **Does NOT auto-generate** a new key if none exists — returns an error
/// instead. This prevents silent data corruption when the keyring is cleared
/// and a new key would fail to decrypt existing data.
///
/// Use [`get_or_create_encryption_key`] for operations that store new data.
pub fn get_encryption_key() -> Result<Vec<u8>, String> {
    let entry = keyring::Entry::new("port-manager", "encryption-key")
        .map_err(|e| format!("Failed to create keyring entry: {e}"))?;

    match entry.get_password() {
        Ok(key) => Ok(key.into_bytes()),
        Err(keyring::Error::NoEntry) => Err(
            "No encryption key found in keyring. Your stored data cannot be decrypted. \
             Please re-import your kubeconfigs and credentials."
                .to_string(),
        ),
        Err(e) => Err(format!(
            "Failed to retrieve encryption key from keyring: {e}"
        )),
    }
}

/// Retrieve or create the encryption key.
///
/// Creates a new UUID key in the keyring if none exists. Use this for
/// operations that store **new** data (import, save).
pub fn get_or_create_encryption_key() -> Result<Vec<u8>, String> {
    legacy_get_encryption_key()
}

/// Encrypt data using AES-256-GCM with a key derived from the UUID keyring key.
///
/// Returns encrypted bytes suitable for storage.
pub fn secure_encrypt(data: &[u8], uuid_key: &[u8]) -> Result<Vec<u8>, String> {
    let aes_key = derive_aes_key(uuid_key)?;
    encrypt(data, &aes_key)
}

/// Decrypt data, trying AES-256-GCM first, then falling back to legacy XOR.
///
/// Returns `(plaintext, needs_migration)`:
/// - `needs_migration = false`: data was AES-encrypted (current format)
/// - `needs_migration = true`: data was XOR-encrypted (legacy format, should be re-encrypted)
///
/// AES-GCM is authenticated, so if the key is wrong, decryption fails with an
/// error instead of silently producing garbage (unlike XOR).
pub fn secure_decrypt(data: &[u8], uuid_key: &[u8]) -> Result<(Vec<u8>, bool), String> {
    // Try AES-256-GCM first (current format)
    let aes_key = derive_aes_key(uuid_key)?;
    if data.len() >= 12 {
        if let Ok(plaintext) = decrypt(data, &aes_key) {
            return Ok((plaintext, false));
        }
    }

    // Fallback to legacy XOR
    let plaintext = legacy_encrypt_decrypt(data, uuid_key);
    Ok((plaintext, true))
}

// ── Helpers ─────────────────────────────────────────────────────────────

/// Constant-time byte-slice comparison.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
