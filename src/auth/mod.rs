//! Authentication module for TwoPassword
//!
//! Provides Passkey/WebAuthn, Touch ID integration and fallback password authentication

use crate::{Result, TwoPasswordError};

#[cfg(target_os = "macos")]
pub mod touchid;

pub mod password;
pub mod recovery;
pub mod passkey;

/// Authentication result
#[derive(Debug)]
pub enum AuthResult {
    /// Authentication successful with Touch ID
    TouchIdSuccess,
    /// Authentication successful with password
    PasswordSuccess,
    /// Authentication successful with Passkey
    PasskeySuccess {
        user_id: uuid::Uuid,
        auth_token: Vec<u8>,
    },
    /// Authentication failed
    Failed(String),
}

/// Main authentication manager
pub struct AuthManager {
    #[cfg(target_os = "macos")]
    touch_id_available: bool,
    passkey_manager: Option<passkey::PasskeyManager>,
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new() -> Self {
        let passkey_manager = if passkey::PasskeyManager::is_platform_supported() {
            match passkey::PasskeyManager::new(passkey::PasskeyConfig::default()) {
                Ok(manager) => Some(manager),
                Err(e) => {
                    tracing::warn!("Failed to initialize Passkey manager: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            #[cfg(target_os = "macos")]
            touch_id_available: touchid::is_available(),
            passkey_manager,
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

    /// Check if Passkey authentication is available
    pub fn is_passkey_available(&self) -> bool {
        self.passkey_manager.is_some()
    }

    /// Register a new Passkey credential
    pub fn register_passkey_credential(
        &mut self,
        username: &str,
        display_name: Option<&str>,
    ) -> Result<passkey::PasskeyCredential> {
        match &mut self.passkey_manager {
            Some(manager) => manager.register_credential(username, display_name),
            None => Err(TwoPasswordError::passkey("Passkey not available")),
        }
    }

    /// Authenticate with Passkey
    pub fn authenticate_passkey(
        &mut self,
        username: Option<&str>,
    ) -> Result<AuthResult> {
        match &mut self.passkey_manager {
            Some(manager) => {
                let result = manager.authenticate(username)?;
                if result.success {
                    if let (Some(user_id), Some(auth_token)) = (result.user_id, result.auth_token) {
                        Ok(AuthResult::PasskeySuccess { user_id, auth_token })
                    } else {
                        Ok(AuthResult::Failed("Incomplete Passkey authentication".to_string()))
                    }
                } else {
                    Ok(AuthResult::Failed(
                        result.error.unwrap_or_else(|| "Passkey authentication failed".to_string()),
                    ))
                }
            }
            None => Ok(AuthResult::Failed("Passkey not available".to_string())),
        }
    }

    /// List all Passkey credentials
    pub fn list_passkey_credentials(&self) -> Vec<&passkey::PasskeyCredential> {
        match &self.passkey_manager {
            Some(manager) => manager.list_credentials(),
            None => Vec::new(),
        }
    }

    /// Remove a Passkey credential
    pub fn remove_passkey_credential(&mut self, credential_id: &str) -> Result<bool> {
        match &mut self.passkey_manager {
            Some(manager) => manager.remove_credential(credential_id),
            None => Err(TwoPasswordError::passkey("Passkey not available")),
        }
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}
