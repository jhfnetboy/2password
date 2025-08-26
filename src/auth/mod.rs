//! Authentication module for TwoPassword
//!
//! Provides Touch ID integration and fallback password authentication

use crate::{Result, TwoPasswordError};

#[cfg(target_os = "macos")]
pub mod touchid;

pub mod password;
pub mod recovery;

/// Authentication result
#[derive(Debug)]
pub enum AuthResult {
    /// Authentication successful with Touch ID
    TouchIdSuccess,
    /// Authentication successful with password
    PasswordSuccess,
    /// Authentication failed
    Failed(String),
}

/// Main authentication manager
pub struct AuthManager {
    #[cfg(target_os = "macos")]
    touch_id_available: bool,
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "macos")]
            touch_id_available: touchid::is_available(),
        }
    }

    /// Check if Touch ID is available
    #[cfg(target_os = "macos")]
    pub fn is_touch_id_available(&self) -> bool {
        self.touch_id_available
    }

    /// Check if Touch ID is available (always false on non-macOS)
    #[cfg(not(target_os = "macos"))]
    pub fn is_touch_id_available(&self) -> bool {
        false
    }

    /// Authenticate using Touch ID
    #[cfg(target_os = "macos")]
    pub fn authenticate_touch_id(&self, reason: &str) -> Result<AuthResult> {
        if !self.touch_id_available {
            return Ok(AuthResult::Failed("Touch ID not available".to_string()));
        }

        match touchid::authenticate(reason) {
            Ok(true) => Ok(AuthResult::TouchIdSuccess),
            Ok(false) => Ok(AuthResult::Failed(
                "Touch ID authentication failed".to_string(),
            )),
            Err(e) => Ok(AuthResult::Failed(format!("Touch ID error: {}", e))),
        }
    }

    /// Authenticate using Touch ID (always fails on non-macOS)
    #[cfg(not(target_os = "macos"))]
    pub fn authenticate_touch_id(&self, _reason: &str) -> Result<AuthResult> {
        Ok(AuthResult::Failed(
            "Touch ID not available on this platform".to_string(),
        ))
    }

    /// Authenticate using password
    pub fn authenticate_password(&self, password: &str, stored_hash: &str) -> Result<AuthResult> {
        match password::verify_password(password, stored_hash) {
            Ok(true) => Ok(AuthResult::PasswordSuccess),
            Ok(false) => Ok(AuthResult::Failed("Invalid password".to_string())),
            Err(e) => Ok(AuthResult::Failed(format!(
                "Password verification error: {}",
                e
            ))),
        }
    }

    /// Generate password hash for storage
    pub fn hash_password(&self, password: &str) -> Result<String> {
        password::hash_password(password)
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}
