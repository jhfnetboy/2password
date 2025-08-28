//! Key derivation functions using Argon2id
//!
//! Implements multi-factor key derivation as per 2Password design:
//! 主密钥 = Argon2id(简单主密码 + Passkey 认证令牌 + 用户 iCloud ID 哈希值 + 随机盐值)

use crate::{Result, TwoPasswordError};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, Params};
use sha2::{Sha256, Digest};
use base64::Engine;

/// Multi-factor key derivation input
#[derive(Debug)]
pub struct MultiFactorInput {
    /// Simple user password
    pub simple_password: String,
    /// Passkey authentication token (from Touch ID/Face ID)
    pub passkey_auth_token: Vec<u8>,
    /// iCloud ID hash (user identity factor)
    pub icloud_id_hash: Vec<u8>,
    /// Random salt for additional entropy
    pub random_salt: Vec<u8>,
}

/// Argon2id configuration following 2Password security requirements
pub struct Argon2Config {
    /// Memory usage (64MB as per design)
    pub memory: u32,
    /// Number of iterations (3 as per design)
    pub iterations: u32,
    /// Parallelism (4 threads as per design)
    pub parallelism: u32,
    /// Output length (32 bytes for AES-256)
    pub output_length: usize,
}

impl Default for Argon2Config {
    fn default() -> Self {
        Self {
            memory: 65536, // 64MB in KB
            iterations: 3,
            parallelism: 4,
            output_length: 32,
        }
    }
}

/// Derive master key using multi-factor input and Argon2id
/// This is the core security function implementing the design:
/// 主密钥 = Argon2id(简单主密码 + Passkey 认证令牌 + 用户 iCloud ID 哈希值 + 随机盐值)
pub fn derive_master_key(input: &MultiFactorInput, config: Option<Argon2Config>) -> Result<[u8; 32]> {
    let config = config.unwrap_or_default();
    
    // Step 1: Combine all factors into a single input
    let combined_input = combine_factors(input)?;
    
    // Step 2: Create Argon2 configuration
    let params = Params::new(
        config.memory,
        config.iterations,
        config.parallelism,
        Some(config.output_length),
    ).map_err(|e| TwoPasswordError::crypto(format!("Invalid Argon2 parameters: {}", e)))?;
    
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    );
    
    // Step 3: Use the random salt for Argon2
    let salt_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(&input.random_salt);
    let salt_string = SaltString::from_b64(&salt_b64)
        .map_err(|e| TwoPasswordError::crypto(format!("Salt conversion failed: {}", e)))?;
    
    // Step 4: Derive the key
    let password_hash = argon2
        .hash_password(&combined_input, &salt_string)
        .map_err(|e| TwoPasswordError::crypto(format!("Multi-factor key derivation failed: {}", e)))?;
    
    // Step 5: Extract the hash bytes
    let hash_bytes = password_hash
        .hash
        .ok_or_else(|| TwoPasswordError::crypto("No hash in password hash"))?;
    
    // Convert to fixed-size array
    let mut master_key = [0u8; 32];
    let hash_slice = hash_bytes.as_bytes();
    let copy_len = std::cmp::min(32, hash_slice.len());
    master_key[..copy_len].copy_from_slice(&hash_slice[..copy_len]);
    
    tracing::info!("Multi-factor master key derived successfully using Argon2id");
    Ok(master_key)
}

/// Combine all factors into a single byte array for Argon2 input
fn combine_factors(input: &MultiFactorInput) -> Result<Vec<u8>> {
    let mut hasher = Sha256::new();
    
    // Add simple password
    hasher.update(input.simple_password.as_bytes());
    hasher.update(b"\x00SIMPLE_PASSWORD\x00"); // Separator
    
    // Add Passkey authentication token
    hasher.update(&input.passkey_auth_token);
    hasher.update(b"\x00PASSKEY_TOKEN\x00"); // Separator
    
    // Add iCloud ID hash
    hasher.update(&input.icloud_id_hash);
    hasher.update(b"\x00ICLOUD_ID\x00"); // Separator
    
    // Add random salt (but also use it as Argon2 salt)
    hasher.update(&input.random_salt);
    hasher.update(b"\x00RANDOM_SALT\x00"); // Separator
    
    // Final hash to create combined input
    let combined = hasher.finalize();
    Ok(combined.to_vec())
}

/// Derive a key from password and salt using Argon2id (legacy function)
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

/// Create multi-factor input by combining user password with Passkey authentication
pub fn create_multifactor_input(
    simple_password: String,
    passkey_auth_token: Vec<u8>,
    icloud_id_hash: Vec<u8>,
) -> Result<MultiFactorInput> {
    // Generate a cryptographically secure random salt
    let random_salt = crate::crypto::secure_random::generate_bytes(32)?;
    
    Ok(MultiFactorInput {
        simple_password,
        passkey_auth_token,
        icloud_id_hash,
        random_salt,
    })
}

/// Create multi-factor input with a provided salt (for loading existing vaults)
pub fn create_multifactor_input_with_salt(
    simple_password: String,
    passkey_auth_token: Vec<u8>,
    icloud_id_hash: Vec<u8>,
    random_salt: Vec<u8>,
) -> MultiFactorInput {
    MultiFactorInput {
        simple_password,
        passkey_auth_token,
        icloud_id_hash,
        random_salt,
    }
}

/// Verify a password against a multi-factor derived key
pub fn verify_multifactor_password(
    input: &MultiFactorInput,
    stored_hash: &str,
    config: Option<Argon2Config>,
) -> Result<bool> {
    // Derive the key using multi-factor input
    let derived_key = derive_master_key(input, config)?;
    
    // For verification, we need to compare against the stored hash
    // In practice, this would involve deriving a verification hash
    // For now, we'll use the standard verification method
    let parsed_hash = PasswordHash::new(stored_hash)
        .map_err(|e| TwoPasswordError::crypto(format!("Invalid stored hash: {}", e)))?;

    let combined_input = combine_factors(input)?;
    let argon2 = Argon2::default();

    match argon2.verify_password(&combined_input, &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(TwoPasswordError::crypto(format!(
            "Multi-factor password verification failed: {}",
            e
        ))),
    }
}

#[cfg(test)]
mod multifactor_tests {
    use super::*;

    #[test]
    fn test_multifactor_key_derivation() {
        let simple_password = "123456".to_string();
        let passkey_token = vec![0x01, 0x02, 0x03, 0x04]; // Mock Passkey token
        let icloud_id = vec![0x05, 0x06, 0x07, 0x08]; // Mock iCloud ID hash
        let salt = vec![0x09; 32]; // Mock salt

        let input = create_multifactor_input_with_salt(
            simple_password.clone(),
            passkey_token.clone(),
            icloud_id.clone(),
            salt.clone(),
        );

        // Test key derivation
        let key1 = derive_master_key(&input, None).unwrap();
        assert_eq!(key1.len(), 32);

        // Same inputs should produce same key
        let key2 = derive_master_key(&input, None).unwrap();
        assert_eq!(key1, key2);

        // Different password should produce different key
        let input2 = create_multifactor_input_with_salt(
            "different".to_string(),
            passkey_token,
            icloud_id,
            salt,
        );
        let key3 = derive_master_key(&input2, None).unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_factor_combination() {
        let input = MultiFactorInput {
            simple_password: "test".to_string(),
            passkey_auth_token: vec![1, 2, 3],
            icloud_id_hash: vec![4, 5, 6],
            random_salt: vec![7, 8, 9],
        };

        let combined1 = combine_factors(&input).unwrap();
        let combined2 = combine_factors(&input).unwrap();
        
        // Same input should produce same combination
        assert_eq!(combined1, combined2);
        assert_eq!(combined1.len(), 32); // SHA-256 output
    }
}
