//! Key derivation functions using Argon2id

use crate::{Result, TwoPasswordError};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

/// Derive a key from password and salt using Argon2id
pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let argon2 = Argon2::default();

    // Convert salt bytes to SaltString using base64 encoding without padding
    use base64::engine::{general_purpose, Engine as _};
    let salt_b64 = general_purpose::STANDARD_NO_PAD.encode(salt);
    let salt_string = SaltString::from_b64(&salt_b64)
        .map_err(|e| TwoPasswordError::crypto(format!("Salt conversion failed: {}", e)))?;

    // Hash the password
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| TwoPasswordError::crypto(format!("Key derivation failed: {}", e)))?;

    // Extract the hash bytes
    let hash_bytes = password_hash
        .hash
        .ok_or_else(|| TwoPasswordError::crypto("No hash in password hash"))?;

    // Convert to fixed-size array
    let mut key = [0u8; 32];
    let hash_slice = hash_bytes.as_bytes();
    let copy_len = std::cmp::min(32, hash_slice.len());
    key[..copy_len].copy_from_slice(&hash_slice[..copy_len]);

    Ok(key)
}

/// Hash a password for storage (with random salt)
pub fn hash_password_for_storage(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| TwoPasswordError::crypto(format!("Password hashing failed: {}", e)))?;

    Ok(password_hash.to_string())
}

/// Verify a password against a stored hash
pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(stored_hash)
        .map_err(|e| TwoPasswordError::crypto(format!("Invalid stored hash: {}", e)))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(TwoPasswordError::crypto(format!(
            "Password verification failed: {}",
            e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key() {
        let password = "test_password";
        let salt = vec![0u8; 32]; // Use zeros for reproducibility

        let key = derive_key(password, &salt).unwrap();
        assert_eq!(key.len(), 32);

        // Same inputs should produce same key
        let key2 = derive_key(password, &salt).unwrap();
        assert_eq!(key, key2);

        // Different password should produce different key
        let key3 = derive_key("different", &salt).unwrap();
        assert_ne!(key, key3);
    }

    #[test]
    fn test_hash_and_verify_password() {
        let password = "test_password";

        let hash = hash_password_for_storage(password).unwrap();
        assert!(!hash.is_empty());

        // Correct password should verify
        assert!(verify_password(password, &hash).unwrap());

        // Wrong password should not verify
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }
}
