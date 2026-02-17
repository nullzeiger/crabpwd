use crate::error::{PasswordManagerError, Result};
use crate::models::Password;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use argon2::Argon2;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use getrandom;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Name of the encrypted file where passwords are stored.
const ENC_FILE: &str = ".pwd.enc";

/// Salt size for Argon2 (16 bytes = 128 bits).
const SALT_LEN: usize = 16;

/// Nonce size for AES-256-GCM (12 bytes = 96 bits).
const NONCE_LEN: usize = 12;

/// Wrapper written to disk as JSON.
/// All binary fields (salt, nonce, ciphertext) are base64-encoded so the
/// file remains valid UTF-8 and human-inspectable.
#[derive(Serialize, Deserialize)]
struct EncryptedStore {
    /// Argon2id key-derivation salt, base64-encoded.
    salt: String,
    /// AES-GCM nonce, base64-encoded.
    nonce: String,
    /// Encrypted password list (JSON ciphertext), base64-encoded.
    data: String,
}

/// In-process cache of the master password: asked only once per session.
/// Uses `once_cell::sync::OnceCell` because `std::sync::OnceLock::get_or_try_init`
/// is still behind an unstable feature gate (tracking issue #109737).
static MASTER_PASSWORD: OnceCell<String> = OnceCell::new();

/// Returns the full path to the encrypted storage file in the user's home directory.
fn get_file_path() -> Result<PathBuf> {
    let home_dir = env::var("HOME").map_err(|_| {
        PasswordManagerError::NotFound("HOME environment variable not set".to_string())
    })?;
    Ok(PathBuf::from(home_dir).join(ENC_FILE))
}

/// Returns the master password, prompting the user at most once per process.
///
/// Resolution order:
/// 1. `CRABPWD_MASTER_PASSWORD` environment variable (useful in scripts / CI).
/// 2. Interactive terminal prompt (input is not echoed).
fn get_master_password() -> Result<&'static str> {
    MASTER_PASSWORD
        .get_or_try_init(|| {
            if let Ok(pwd) = env::var("CRABPWD_MASTER_PASSWORD") {
                return Ok(pwd);
            }
            rpassword::prompt_password("Master password: ").map_err(PasswordManagerError::Io)
        })
        .map(|s| s.as_str())
}

/// Derives a 256-bit AES key from `password` and `salt` using Argon2id.
///
/// Argon2id is memory-hard and resistant to both brute-force and
/// side-channel attacks, making it suitable for password-based key derivation.
fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| PasswordManagerError::Crypto(e.to_string()))?;
    Ok(key)
}

/// Reads, decrypts, and deserializes the password list from disk.
///
/// Returns an empty vector if the storage file does not yet exist.
/// If the master password is wrong, AES-GCM authentication fails and
/// a `Crypto` error is returned — the file is never modified.
pub fn load_passwords() -> Result<Vec<Password>> {
    let path = get_file_path()?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let master_pwd = get_master_password()?;

    // Read and parse the JSON wrapper from disk.
    let content = fs::read_to_string(&path)?;
    let store: EncryptedStore =
        serde_json::from_str(&content).map_err(|e| PasswordManagerError::Parse(e.to_string()))?;

    // Decode each base64-encoded field back to raw bytes.
    let salt = BASE64
        .decode(&store.salt)
        .map_err(|e| PasswordManagerError::Crypto(format!("Invalid salt: {}", e)))?;
    let nonce_bytes = BASE64
        .decode(&store.nonce)
        .map_err(|e| PasswordManagerError::Crypto(format!("Invalid nonce: {}", e)))?;
    let ciphertext = BASE64
        .decode(&store.data)
        .map_err(|e| PasswordManagerError::Crypto(format!("Invalid ciphertext: {}", e)))?;

    // Sanity-check the nonce length before passing it to AES-GCM.
    if nonce_bytes.len() != NONCE_LEN {
        return Err(PasswordManagerError::Crypto(
            "Nonce has unexpected length".to_string(),
        ));
    }

    // Derive the encryption key and decrypt. The GCM tag guarantees both
    // confidentiality and integrity: any tampering or wrong password is caught here.
    let key = derive_key(master_pwd, &salt)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| PasswordManagerError::Crypto(e.to_string()))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|_| {
        PasswordManagerError::Crypto(
            "Decryption failed: wrong master password or corrupted file".to_string(),
        )
    })?;

    // The plaintext is the JSON-serialized Vec<Password>.
    serde_json::from_slice(&plaintext).map_err(|e| PasswordManagerError::Parse(e.to_string()))
}

/// Encrypts and writes the password list to disk.
///
/// A fresh random salt and nonce are generated on every save, so encrypting
/// the same plaintext twice always produces a different ciphertext
/// (semantic security / IND-CPA).
pub fn save_passwords(passwords: &[Password]) -> Result<()> {
    let path = get_file_path()?;
    let master_pwd = get_master_password()?;

    // Generate cryptographically secure random bytes for the salt and nonce
    // directly from the OS CSPRNG via the `getrandom` crate.
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    getrandom::fill(&mut salt)
        .map_err(|e| PasswordManagerError::Crypto(format!("RNG error: {}", e)))?;
    getrandom::fill(&mut nonce_bytes)
        .map_err(|e| PasswordManagerError::Crypto(format!("RNG error: {}", e)))?;

    // Derive the key from the master password and the fresh salt, then encrypt.
    let key = derive_key(master_pwd, &salt)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| PasswordManagerError::Crypto(e.to_string()))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext =
        serde_json::to_vec(passwords).map_err(|e| PasswordManagerError::Parse(e.to_string()))?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| PasswordManagerError::Crypto(e.to_string()))?;

    // Build the JSON wrapper with all binary fields base64-encoded, then
    // write it atomically via `fs::write` (single syscall, no partial writes).
    let store = EncryptedStore {
        salt: BASE64.encode(salt),
        nonce: BASE64.encode(nonce_bytes),
        data: BASE64.encode(&ciphertext),
    };

    let content = serde_json::to_string_pretty(&store)
        .map_err(|e| PasswordManagerError::Parse(e.to_string()))?;

    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Encrypting then decrypting with the same key/nonce must recover the original plaintext.
    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let password = "test_master_password_123!";
        let mut salt = [0u8; SALT_LEN];
        let mut nonce_bytes = [0u8; NONCE_LEN];
        getrandom::fill(&mut salt).unwrap();
        getrandom::fill(&mut nonce_bytes).unwrap();

        let key = derive_key(password, &salt).unwrap();
        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = b"hello encrypted world";
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).unwrap();
        let decrypted = cipher.decrypt(nonce, ciphertext.as_ref()).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    /// Decrypting with a different key must fail (GCM authentication tag mismatch).
    #[test]
    fn test_wrong_key_fails() {
        let mut salt = [0u8; SALT_LEN];
        let mut nonce_bytes = [0u8; NONCE_LEN];
        getrandom::fill(&mut salt).unwrap();
        getrandom::fill(&mut nonce_bytes).unwrap();

        let key_good = derive_key("correct_password", &salt).unwrap();
        let key_bad = derive_key("wrong_password", &salt).unwrap();

        let cipher_enc = Aes256Gcm::new_from_slice(&key_good).unwrap();
        let cipher_dec = Aes256Gcm::new_from_slice(&key_bad).unwrap();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher_enc.encrypt(nonce, b"secret".as_ref()).unwrap();
        assert!(cipher_dec.decrypt(nonce, ciphertext.as_ref()).is_err());
    }

    /// `derive_key` must always return exactly 32 bytes regardless of input.
    #[test]
    fn test_key_derivation_length() {
        let salt = [42u8; SALT_LEN];
        let key = derive_key("any_password", &salt).unwrap();
        assert_eq!(key.len(), 32);
    }

    /// The same password with different salts must produce different keys
    /// (ensures that the salt is actually fed into the KDF).
    #[test]
    fn test_different_salts_produce_different_keys() {
        let salt1 = [1u8; SALT_LEN];
        let salt2 = [2u8; SALT_LEN];
        let key1 = derive_key("same_password", &salt1).unwrap();
        let key2 = derive_key("same_password", &salt2).unwrap();
        assert_ne!(key1, key2);
    }
}
