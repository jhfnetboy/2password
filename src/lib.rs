//! TwoPassword - A secure password manager with Touch ID integration
//!
//! This library provides a zero-knowledge password manager that integrates with
//! macOS Touch ID for secure authentication and uses industry-standard encryption.

pub mod auth;
pub mod cli;
pub mod crypto;
pub mod error;
pub mod import_export;
pub mod password_health;
pub mod security;
pub mod storage;

// Re-export common types
pub use error::{Result, TwoPasswordError};

/// Application configuration and constants
pub mod config {
    /// Default vault file name
    pub const VAULT_FILE_NAME: &str = "vault.enc";

    /// Application name for system integration
    pub const APP_NAME: &str = "TwoPassword";

    /// Version for file format compatibility
    pub const FORMAT_VERSION: u32 = 1;

    /// Key derivation iteration count
    pub const PBKDF2_ITERATIONS: u32 = 100_000;

    /// AES-GCM key size in bytes
    pub const KEY_SIZE: usize = 32;

    /// AES-GCM nonce size in bytes
    pub const NONCE_SIZE: usize = 12;

    /// Salt size for key derivation
    pub const SALT_SIZE: usize = 32;

    /// HMAC size for integrity verification
    pub const HMAC_SIZE: usize = 32;
}

/// Initialize the application with proper logging
pub fn init() -> Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt::init();

    tracing::info!("TwoPassword initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }

    #[test]
    fn test_constants() {
        assert_eq!(config::FORMAT_VERSION, 1);
        assert_eq!(config::KEY_SIZE, 32);
        assert_eq!(config::NONCE_SIZE, 12);
    }
}
