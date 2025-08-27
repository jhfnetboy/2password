//! Error handling for TwoPassword

use thiserror::Error;

/// Main error type for TwoPassword operations
#[derive(Error, Debug)]
pub enum TwoPasswordError {
    #[error("Cryptographic operation failed: {0}")]
    CryptoError(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Storage operation failed: {0}")]
    StorageError(String),

    #[error("Vault not found or corrupted")]
    VaultNotFound,

    #[error("Invalid vault format or version")]
    InvalidVaultFormat,

    #[error("Password entry not found: {0}")]
    EntryNotFound(String),

    #[error("Invalid master password")]
    InvalidMasterPassword,

    #[error("Touch ID not available or failed")]
    TouchIdError(String),

    #[error("IO operation failed: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Import/Export error: {0}")]
    ImportExportError(String),

    #[error("Security error: {0}")]
    SecurityError(String),
}

/// Convenience result type
pub type Result<T> = std::result::Result<T, TwoPasswordError>;

impl TwoPasswordError {
    /// Create a crypto error
    pub fn crypto<S: Into<String>>(msg: S) -> Self {
        Self::CryptoError(msg.into())
    }

    /// Create an auth error
    pub fn auth<S: Into<String>>(msg: S) -> Self {
        Self::AuthError(msg.into())
    }

    /// Create a storage error
    pub fn storage<S: Into<String>>(msg: S) -> Self {
        Self::StorageError(msg.into())
    }

    /// Create a config error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::ConfigError(msg.into())
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::ValidationError(msg.into())
    }

    /// Create a Touch ID error
    pub fn touch_id<S: Into<String>>(msg: S) -> Self {
        Self::TouchIdError(msg.into())
    }

    /// Create an import/export error
    pub fn import_export<S: Into<String>>(msg: S) -> Self {
        Self::ImportExportError(msg.into())
    }

    /// Create a security error
    pub fn security<S: Into<String>>(msg: S) -> Self {
        Self::SecurityError(msg.into())
    }
}
