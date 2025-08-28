//! Passkey/WebAuthn integration for TwoPassword
//!
//! Simplified Passkey implementation that integrates with macOS Touch ID
//! and provides authentication tokens for multi-factor key derivation

use crate::{Result, TwoPasswordError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Passkey configuration
#[derive(Debug, Clone)]
pub struct PasskeyConfig {
    /// Application identifier
    pub app_id: String,
    /// Application name
    pub app_name: String,
}

impl Default for PasskeyConfig {
    fn default() -> Self {
        Self {
            app_id: "com.twopassword.app".to_string(),
            app_name: "2Password".to_string(),
        }
    }
}

/// Passkey credential information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyCredential {
    /// Credential ID
    pub credential_id: String,
    /// User ID (UUID)
    pub user_id: Uuid,
    /// Username/display name
    pub username: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last used timestamp
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    /// Platform type (Touch ID, Face ID, etc.)
    pub platform_type: String,
}

/// Passkey authentication result
#[derive(Debug, Clone)]
pub struct PasskeyAuthResult {
    /// Authentication successful
    pub success: bool,
    /// User ID if successful
    pub user_id: Option<Uuid>,
    /// Authentication token (for key derivation)
    pub auth_token: Option<Vec<u8>>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Simplified Passkey manager that uses Touch ID/Face ID for authentication
pub struct PasskeyManager {
    config: PasskeyConfig,
    /// Stored credentials (in production, would use secure storage)
    credentials: Vec<PasskeyCredential>,
}

impl PasskeyManager {
    /// Create a new Passkey manager
    pub fn new(config: PasskeyConfig) -> Result<Self> {
        Ok(Self {
            config,
            credentials: Vec::new(),
        })
    }

    /// Register a new Passkey credential using Touch ID
    pub fn register_credential(
        &mut self,
        username: &str,
        display_name: Option<&str>,
    ) -> Result<PasskeyCredential> {
        // Check if Touch ID is available
        #[cfg(target_os = "macos")]
        {
            if !crate::auth::touchid::is_available() {
                return Err(TwoPasswordError::passkey("Touch ID/Face ID not available"));
            }

            // Prompt for Touch ID authentication for registration
            if !crate::auth::touchid::authenticate(&format!("Register {} for 2Password", username))? {
                return Err(TwoPasswordError::passkey("Touch ID authentication failed"));
            }
        }

        let user_id = Uuid::new_v4();
        let credential_id = format!("passkey_{}", user_id);
        let display_name = display_name.unwrap_or(username);

        let credential = PasskeyCredential {
            credential_id,
            user_id,
            username: display_name.to_string(),
            created_at: chrono::Utc::now(),
            last_used: None,
            platform_type: Self::detect_platform_type(),
        };

        // Store the credential
        self.credentials.push(credential.clone());

        tracing::info!("Registered Passkey credential for user: {}", username);
        Ok(credential)
    }

    /// Authenticate using Passkey/Touch ID
    pub fn authenticate(
        &mut self,
        username: Option<&str>,
    ) -> Result<PasskeyAuthResult> {
        // Check if Touch ID is available
        #[cfg(target_os = "macos")]
        {
            if !crate::auth::touchid::is_available() {
                return Ok(PasskeyAuthResult {
                    success: false,
                    user_id: None,
                    auth_token: None,
                    error: Some("Touch ID/Face ID not available".to_string()),
                });
            }

            // Find credential if username provided
            let credential = if let Some(username) = username {
                self.credentials.iter().find(|c| c.username == username)
            } else {
                self.credentials.first()
            };

            let auth_message = if let Some(cred) = credential {
                format!("Authenticate {} with 2Password", cred.username)
            } else {
                "Authenticate with 2Password using Touch ID/Face ID".to_string()
            };

            // Prompt for Touch ID authentication
            match crate::auth::touchid::authenticate(&auth_message) {
                Ok(true) => {
                    // Generate authentication token
                    let auth_token = self.generate_auth_token(credential)?;
                    
                    let result = PasskeyAuthResult {
                        success: true,
                        user_id: credential.map(|c| c.user_id),
                        auth_token: Some(auth_token),
                        error: None,
                    };

                    tracing::info!("Passkey authentication successful");
                    Ok(result)
                }
                Ok(false) => {
                    Ok(PasskeyAuthResult {
                        success: false,
                        user_id: None,
                        auth_token: None,
                        error: Some("Touch ID authentication cancelled".to_string()),
                    })
                }
                Err(e) => {
                    Ok(PasskeyAuthResult {
                        success: false,
                        user_id: None,
                        auth_token: None,
                        error: Some(format!("Touch ID authentication failed: {}", e)),
                    })
                }
            }
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            Ok(PasskeyAuthResult {
                success: false,
                user_id: None,
                auth_token: None,
                error: Some("Passkey not supported on this platform".to_string()),
            })
        }
    }

    /// Generate authentication token for key derivation
    fn generate_auth_token(&self, credential: Option<&PasskeyCredential>) -> Result<Vec<u8>> {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        
        // Include app identifier
        hasher.update(self.config.app_id.as_bytes());
        
        // Include credential information if available
        if let Some(cred) = credential {
            hasher.update(cred.credential_id.as_bytes());
            hasher.update(cred.user_id.as_bytes());
        }
        
        // Include current timestamp to ensure uniqueness
        let timestamp = chrono::Utc::now().timestamp().to_be_bytes();
        hasher.update(&timestamp);
        
        // Include platform-specific data
        #[cfg(target_os = "macos")]
        {
            // Use system-specific entropy
            if let Ok(system_id) = std::process::Command::new("ioreg")
                .args(&["-rd1", "-c", "IOPlatformExpertDevice"])
                .output()
            {
                hasher.update(&system_id.stdout);
            }
        }
        
        // Add some randomness
        use crate::crypto::secure_random;
        let random_bytes = secure_random::generate_bytes(32)?;
        hasher.update(&random_bytes);
        
        let auth_token = hasher.finalize().to_vec();
        Ok(auth_token)
    }

    /// Detect platform type for authenticator info
    fn detect_platform_type() -> String {
        #[cfg(target_os = "macos")]
        {
            // Check if Touch ID or Face ID is available
            if crate::auth::touchid::is_available() {
                "Touch ID / Face ID".to_string()
            } else {
                "Platform Authenticator".to_string()
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            "Platform Authenticator".to_string()
        }
    }

    /// List all registered credentials
    pub fn list_credentials(&self) -> Vec<&PasskeyCredential> {
        self.credentials.iter().collect()
    }

    /// Remove a credential
    pub fn remove_credential(&mut self, credential_id: &str) -> Result<bool> {
        let initial_len = self.credentials.len();
        self.credentials.retain(|c| c.credential_id != credential_id);
        Ok(self.credentials.len() < initial_len)
    }

    /// Generate iCloud ID hash for key derivation
    pub fn get_icloud_id_hash() -> Result<Vec<u8>> {
        #[cfg(target_os = "macos")]
        {
            // In a real implementation, we would get the actual iCloud account ID
            // For now, use a system-specific identifier
            use std::process::Command;
            
            let output = Command::new("dscacheutil")
                .args(&["-q", "user", "-a", "name", &whoami::username()])
                .output()
                .map_err(|e| TwoPasswordError::passkey(format!("Failed to get user info: {}", e)))?;
            
            let user_info = String::from_utf8_lossy(&output.stdout);
            
            // Hash the user information to create a stable identifier
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(user_info.as_bytes());
            hasher.update(b"twopassword-icloud-id"); // Salt with app name
            
            Ok(hasher.finalize().to_vec())
        }
        #[cfg(not(target_os = "macos"))]
        {
            // For non-macOS platforms, use username + hostname
            let username = whoami::username();
            let hostname = whoami::hostname();
            
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(username.as_bytes());
            hasher.update(hostname.as_bytes());
            hasher.update(b"twopassword-user-id");
            
            Ok(hasher.finalize().to_vec())
        }
    }

    /// Check if platform supports Passkeys
    pub fn is_platform_supported() -> bool {
        #[cfg(target_os = "macos")]
        {
            crate::auth::touchid::is_available()
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }
}

impl Default for PasskeyManager {
    fn default() -> Self {
        Self::new(PasskeyConfig::default()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passkey_manager_creation() {
        let config = PasskeyConfig::default();
        let manager = PasskeyManager::new(config);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_platform_support() {
        let supported = PasskeyManager::is_platform_supported();
        // Note: This test may fail in CI/test environments without biometric hardware
        // In such cases, it's expected that Touch ID is not available
        #[cfg(target_os = "macos")]
        {
            // Just ensure the function returns a boolean without panicking
            println!("Platform support on macOS: {}", supported);
            // Don't assert true since Touch ID may not be available in test environments
        }
        #[cfg(not(target_os = "macos"))]
        assert!(!supported);
    }

    #[test]
    fn test_icloud_id_hash() {
        let hash = PasskeyManager::get_icloud_id_hash();
        assert!(hash.is_ok());
        let hash_bytes = hash.unwrap();
        assert_eq!(hash_bytes.len(), 32); // SHA-256 output size
    }
}