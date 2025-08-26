//! Cryptographic operations for TwoPassword
//!
//! This module provides secure encryption, decryption, and key derivation
//! operations using industry-standard algorithms.

use crate::{Result, TwoPasswordError};
use zeroize::ZeroizeOnDrop;

pub mod aes_gcm;
pub mod key_derivation;
pub mod secure_random;
pub mod secret_sharing;

/// Master key for encryption operations
#[derive(Clone, ZeroizeOnDrop)]
pub struct MasterKey {
    key: [u8; 32],
}

impl MasterKey {
    /// Create a new master key from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { key: bytes }
    }

    /// Get the key bytes (should be used carefully)
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

/// Encrypted data container
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptedData {
    /// The encrypted ciphertext
    pub ciphertext: Vec<u8>,
    /// The nonce used for encryption
    pub nonce: Vec<u8>,
    /// HMAC for integrity verification
    pub hmac: Vec<u8>,
}

/// Salt for key derivation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Salt {
    pub bytes: Vec<u8>,
}

impl Salt {
    /// Generate a new random salt
    pub fn generate() -> Result<Self> {
        let mut bytes = vec![0u8; crate::config::SALT_SIZE];
        secure_random::fill_random(&mut bytes)?;
        Ok(Self { bytes })
    }

    /// Create salt from existing bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

/// Main cryptographic manager
pub struct CryptoManager {
    master_key: Option<MasterKey>,
}

impl CryptoManager {
    /// Create a new crypto manager
    pub fn new() -> Self {
        Self { master_key: None }
    }

    /// Set the master key for encryption operations
    pub fn set_master_key(&mut self, key: MasterKey) {
        self.master_key = Some(key);
    }

    /// Check if master key is set
    pub fn has_master_key(&self) -> bool {
        self.master_key.is_some()
    }

    /// Clear the master key from memory
    pub fn clear_master_key(&mut self) {
        self.master_key = None;
    }

    /// Encrypt data using the current master key
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        let key = self
            .master_key
            .as_ref()
            .ok_or_else(|| TwoPasswordError::crypto("Master key not set"))?;

        aes_gcm::encrypt(key.as_bytes(), plaintext)
    }

    /// Decrypt data using the current master key
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>> {
        let key = self
            .master_key
            .as_ref()
            .ok_or_else(|| TwoPasswordError::crypto("Master key not set"))?;

        aes_gcm::decrypt(key.as_bytes(), encrypted)
    }

    /// Derive master key from password and salt
    pub fn derive_key(&mut self, password: &str, salt: &Salt) -> Result<()> {
        let key_bytes = key_derivation::derive_key(password, &salt.bytes)?;
        self.master_key = Some(MasterKey::from_bytes(key_bytes));
        Ok(())
    }
}

impl Default for CryptoManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Securely clear sensitive data from memory on drop
impl Drop for CryptoManager {
    fn drop(&mut self) {
        self.clear_master_key();
    }
}
