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

// ── Backward-compatible wrappers (used by kubeconfig.rs, pgmanager.rs) ──

/// Thin wrapper that delegates to the legacy keyring implementation.
pub fn get_encryption_key() -> Result<Vec<u8>, String> {
    legacy_get_encryption_key()
}

/// Thin wrapper that delegates to the legacy XOR implementation.
pub fn encrypt_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    legacy_encrypt_decrypt(data, key)
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
