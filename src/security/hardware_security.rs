//! Hardware security key support and management
//!
//! Provides integration with hardware security keys (FIDO2/WebAuthn),
//! TPM modules, and secure enclaves for enhanced authentication security.

use crate::security::{SecurityEvent, SecurityEventType, SecuritySeverity, AuthenticationLevel};
use crate::{Result, TwoPasswordError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hardware security key types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HardwareKeyType {
    Fido2,      // FIDO2/WebAuthn security keys
    Tpm,        // Trusted Platform Module
    SecureEnclave, // Hardware secure enclave
    SmartCard,  // Smart card authentication
    Yubikey,    // YubiKey devices
}

/// Hardware security key status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyStatus {
    Available,
    InUse,
    Locked,
    Error,
    Disconnected,
}

/// Hardware security key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSecurityKey {
    pub id: String,
    pub name: String,
    pub key_type: HardwareKeyType,
    pub status: KeyStatus,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub firmware_version: Option<String>,
    pub serial_number: Option<String>,
    pub capabilities: Vec<String>,
    pub registered_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub use_count: u64,
    pub is_primary: bool,
}

/// Hardware authentication challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationChallenge {
    pub id: String,
    pub challenge_data: Vec<u8>,
    pub key_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub origin: String,
    pub user_verification: bool,
}

/// Hardware authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationResponse {
    pub challenge_id: String,
    pub signature: Vec<u8>,
    pub authenticator_data: Vec<u8>,
    pub client_data_json: String,
    pub user_handle: Option<Vec<u8>>,
}

/// Hardware security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSecurityConfig {
    pub enable_fido2: bool,
    pub enable_tpm: bool,
    pub enable_secure_enclave: bool,
    pub require_user_verification: bool,
    pub timeout_seconds: u32,
    pub max_registered_keys: u32,
    pub allow_multiple_keys: bool,
    pub backup_key_required: bool,
}

impl Default for HardwareSecurityConfig {
    fn default() -> Self {
        Self {
            enable_fido2: true,
            enable_tpm: true,
            enable_secure_enclave: true,
            require_user_verification: true,
            timeout_seconds: 60,
            max_registered_keys: 5,
            allow_multiple_keys: true,
            backup_key_required: true,
        }
    }
}

/// Hardware security statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSecurityStats {
    pub total_keys_registered: u32,
    pub active_keys: u32,
    pub successful_authentications: u64,
    pub failed_authentications: u64,
    pub keys_by_type: HashMap<HardwareKeyType, u32>,
    pub last_authentication: Option<DateTime<Utc>>,
}

/// Hardware security manager
pub struct HardwareSecurityManager {
    config: HardwareSecurityConfig,
    registered_keys: HashMap<String, HardwareSecurityKey>,
    active_challenges: HashMap<String, AuthenticationChallenge>,
    stats: HardwareSecurityStats,
}

impl HardwareSecurityManager {
    /// Create new hardware security manager
    pub fn new() -> Self {
        Self {
            config: HardwareSecurityConfig::default(),
            registered_keys: HashMap::new(),
            active_challenges: HashMap::new(),
            stats: HardwareSecurityStats {
                total_keys_registered: 0,
                active_keys: 0,
                successful_authentications: 0,
                failed_authentications: 0,
                keys_by_type: HashMap::new(),
                last_authentication: None,
            },
        }
    }
    
    /// Create manager with custom configuration
    pub fn with_config(config: HardwareSecurityConfig) -> Self {
        let mut manager = Self::new();
        manager.config = config;
        manager
    }
    
    /// Discover available hardware security keys
    pub async fn discover_keys(&mut self) -> Result<Vec<HardwareSecurityKey>> {
        let mut discovered_keys = Vec::new();
        
        // Check for FIDO2 keys
        if self.config.enable_fido2 {
            if let Ok(fido2_keys) = self.discover_fido2_keys().await {
                discovered_keys.extend(fido2_keys);
            }
        }
        
        // Check for TPM
        if self.config.enable_tpm {
            if let Ok(tpm_info) = self.discover_tpm().await {
                discovered_keys.push(tpm_info);
            }
        }
        
        // Check for Secure Enclave (macOS/iOS)
        if self.config.enable_secure_enclave {
            if let Ok(enclave_info) = self.discover_secure_enclave().await {
                discovered_keys.push(enclave_info);
            }
        }
        
        // Update active keys count
        self.stats.active_keys = discovered_keys.len() as u32;
        
        Ok(discovered_keys)
    }
    
    /// Discover FIDO2/WebAuthn security keys
    async fn discover_fido2_keys(&self) -> Result<Vec<HardwareSecurityKey>> {
        // In a real implementation, this would use WebAuthn API or CTAP2 protocol
        // For now, we'll simulate discovery
        
        let mut keys = Vec::new();
        
        // Simulated FIDO2 key discovery
        if self.is_fido2_available().await {
            let key = HardwareSecurityKey {
                id: "fido2_key_1".to_string(),
                name: "FIDO2 Security Key".to_string(),
                key_type: HardwareKeyType::Fido2,
                status: KeyStatus::Available,
                manufacturer: Some("Example Corp".to_string()),
                model: Some("SecureKey Pro".to_string()),
                firmware_version: Some("1.0.0".to_string()),
                serial_number: None, // Usually not exposed for privacy
                capabilities: vec![
                    "webauthn".to_string(),
                    "ctap2".to_string(),
                    "resident-key".to_string(),
                    "user-verification".to_string(),
                ],
                registered_at: Utc::now(),
                last_used: None,
                use_count: 0,
                is_primary: false,
            };
            keys.push(key);
        }
        
        Ok(keys)
    }
    
    /// Check if FIDO2 is available on the system
    async fn is_fido2_available(&self) -> bool {
        // In a real implementation, this would check for:
        // - USB HID devices with FIDO2 capability
        // - WebAuthn browser support
        // - Platform authenticator availability
        
        // For now, simulate availability based on platform
        #[cfg(target_os = "windows")]
        return true; // Windows Hello
        
        #[cfg(target_os = "macos")]
        return true; // Touch ID
        
        #[cfg(target_os = "linux")]
        return false; // Depends on hardware and drivers
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return false;
    }
    
    /// Discover TPM (Trusted Platform Module)
    async fn discover_tpm(&self) -> Result<HardwareSecurityKey> {
        // Check for TPM availability
        if !self.is_tpm_available().await {
            return Err(TwoPasswordError::SecurityError("TPM not available".to_string()));
        }
        
        let tpm_key = HardwareSecurityKey {
            id: "tpm_1".to_string(),
            name: "Trusted Platform Module".to_string(),
            key_type: HardwareKeyType::Tpm,
            status: KeyStatus::Available,
            manufacturer: Some("TPM Manufacturer".to_string()),
            model: Some("TPM 2.0".to_string()),
            firmware_version: Some("2.0".to_string()),
            serial_number: None,
            capabilities: vec![
                "key-generation".to_string(),
                "attestation".to_string(),
                "secure-storage".to_string(),
            ],
            registered_at: Utc::now(),
            last_used: None,
            use_count: 0,
            is_primary: false,
        };
        
        Ok(tpm_key)
    }
    
    /// Check if TPM is available
    async fn is_tpm_available(&self) -> bool {
        // In a real implementation, this would check for TPM 2.0 availability
        // For now, simulate availability on Windows
        #[cfg(target_os = "windows")]
        return true;
        
        #[cfg(not(target_os = "windows"))]
        return false;
    }
    
    /// Discover Secure Enclave (Apple devices)
    async fn discover_secure_enclave(&self) -> Result<HardwareSecurityKey> {
        if !self.is_secure_enclave_available().await {
            return Err(TwoPasswordError::SecurityError("Secure Enclave not available".to_string()));
        }
        
        let enclave_key = HardwareSecurityKey {
            id: "secure_enclave_1".to_string(),
            name: "Secure Enclave".to_string(),
            key_type: HardwareKeyType::SecureEnclave,
            status: KeyStatus::Available,
            manufacturer: Some("Apple".to_string()),
            model: Some("Secure Enclave".to_string()),
            firmware_version: None,
            serial_number: None,
            capabilities: vec![
                "biometric-auth".to_string(),
                "key-generation".to_string(),
                "secure-storage".to_string(),
            ],
            registered_at: Utc::now(),
            last_used: None,
            use_count: 0,
            is_primary: false,
        };
        
        Ok(enclave_key)
    }
    
    /// Check if Secure Enclave is available
    async fn is_secure_enclave_available(&self) -> bool {
        #[cfg(target_os = "macos")]
        return true;
        
        #[cfg(not(target_os = "macos"))]
        return false;
    }
    
    /// Register a hardware security key
    pub async fn register_key(&mut self, mut key: HardwareSecurityKey) -> Result<String> {
        // Check if maximum keys reached
        if self.registered_keys.len() >= self.config.max_registered_keys as usize {
            return Err(TwoPasswordError::SecurityError(
                "Maximum number of hardware keys registered".to_string()
            ));
        }
        
        // Ensure key is available
        if key.status != KeyStatus::Available {
            return Err(TwoPasswordError::SecurityError(
                "Hardware key is not available for registration".to_string()
            ));
        }
        
        // Set as primary if no other keys registered
        if self.registered_keys.is_empty() {
            key.is_primary = true;
        }
        
        let key_id = key.id.clone();
        key.registered_at = Utc::now();
        
        // Update statistics
        self.stats.total_keys_registered += 1;
        *self.stats.keys_by_type.entry(key.key_type.clone()).or_insert(0) += 1;
        
        // Store the key
        self.registered_keys.insert(key_id.clone(), key);
        
        tracing::info!("Hardware security key registered: {}", key_id);
        
        Ok(key_id)
    }
    
    /// Unregister a hardware security key
    pub async fn unregister_key(&mut self, key_id: &str) -> Result<()> {
        if let Some(key) = self.registered_keys.remove(key_id) {
            // Update statistics
            if let Some(count) = self.stats.keys_by_type.get_mut(&key.key_type) {
                *count = count.saturating_sub(1);
            }
            
            tracing::info!("Hardware security key unregistered: {}", key_id);
            Ok(())
        } else {
            Err(TwoPasswordError::SecurityError("Key not found".to_string()))
        }
    }
    
    /// Create authentication challenge
    pub async fn create_authentication_challenge(
        &mut self,
        key_id: &str,
        origin: String,
        user_verification: bool,
    ) -> Result<AuthenticationChallenge> {
        let key = self.registered_keys.get(key_id)
            .ok_or_else(|| TwoPasswordError::SecurityError("Key not found".to_string()))?;
            
        if key.status != KeyStatus::Available {
            return Err(TwoPasswordError::SecurityError("Key not available".to_string()));
        }
        
        // Generate challenge data
        let challenge_data = self.generate_challenge_data()?;
        let challenge_id = format!("challenge_{}", Utc::now().timestamp_nanos());
        
        let challenge = AuthenticationChallenge {
            id: challenge_id.clone(),
            challenge_data,
            key_id: key_id.to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::seconds(self.config.timeout_seconds as i64),
            origin,
            user_verification,
        };
        
        self.active_challenges.insert(challenge_id.clone(), challenge.clone());
        
        Ok(challenge)
    }
    
    /// Generate random challenge data
    fn generate_challenge_data(&self) -> Result<Vec<u8>> {
        use rand::RngCore;
        let mut challenge = vec![0u8; 32];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut challenge);
        Ok(challenge)
    }
    
    /// Verify authentication response
    pub async fn verify_authentication(
        &mut self,
        response: AuthenticationResponse,
    ) -> Result<bool> {
        let challenge = self.active_challenges.remove(&response.challenge_id)
            .ok_or_else(|| TwoPasswordError::SecurityError("Challenge not found or expired".to_string()))?;
            
        // Check if challenge has expired
        if Utc::now() > challenge.expires_at {
            return Err(TwoPasswordError::SecurityError("Challenge expired".to_string()));
        }
        
        // First verify the signature
        let is_valid = self.simulate_signature_verification(&challenge, &response).await?;
        
        if is_valid {
            // Then update the key - get mutable reference after verification
            let key = self.registered_keys.get_mut(&challenge.key_id)
                .ok_or_else(|| TwoPasswordError::SecurityError("Key not found".to_string()))?;
            
            // Update key usage
            key.last_used = Some(Utc::now());
            key.use_count += 1;
            
            // Update statistics
            self.stats.successful_authentications += 1;
            self.stats.last_authentication = Some(Utc::now());
            
            tracing::info!("Hardware key authentication successful: {}", challenge.key_id);
        } else {
            self.stats.failed_authentications += 1;
            tracing::warn!("Hardware key authentication failed: {}", challenge.key_id);
        }
        
        Ok(is_valid)
    }
    
    /// Simulate signature verification (placeholder for real cryptographic verification)
    async fn simulate_signature_verification(
        &self,
        _challenge: &AuthenticationChallenge,
        response: &AuthenticationResponse,
    ) -> Result<bool> {
        // In a real implementation, this would:
        // 1. Parse the authenticator data
        // 2. Verify the signature using the key's public key
        // 3. Check counter values for replay protection
        // 4. Validate client data JSON
        
        // For simulation, consider valid if response contains data
        Ok(!response.signature.is_empty() && !response.authenticator_data.is_empty())
    }
    
    /// Get list of registered keys
    pub fn get_registered_keys(&self) -> Vec<&HardwareSecurityKey> {
        self.registered_keys.values().collect()
    }
    
    /// Get hardware security statistics
    pub fn get_statistics(&self) -> &HardwareSecurityStats {
        &self.stats
    }
    
    /// Set primary authentication key
    pub async fn set_primary_key(&mut self, key_id: &str) -> Result<()> {
        // Clear existing primary status
        for key in self.registered_keys.values_mut() {
            key.is_primary = false;
        }
        
        // Set new primary key
        if let Some(key) = self.registered_keys.get_mut(key_id) {
            key.is_primary = true;
            tracing::info!("Primary hardware key set: {}", key_id);
            Ok(())
        } else {
            Err(TwoPasswordError::SecurityError("Key not found".to_string()))
        }
    }
    
    /// Check if hardware authentication is available
    pub fn is_hardware_auth_available(&self) -> bool {
        !self.registered_keys.is_empty() && 
        self.registered_keys.values().any(|key| key.status == KeyStatus::Available)
    }
    
    /// Get required authentication level based on available hardware
    pub fn get_authentication_level(&self) -> AuthenticationLevel {
        if self.is_hardware_auth_available() {
            AuthenticationLevel::HardwareKey
        } else {
            AuthenticationLevel::Password
        }
    }
    
    /// Clean up expired challenges
    pub async fn cleanup_expired_challenges(&mut self) -> Result<()> {
        let now = Utc::now();
        self.active_challenges.retain(|_, challenge| challenge.expires_at > now);
        Ok(())
    }
    
    /// Generate hardware security report
    pub async fn generate_security_report(&self) -> Result<HardwareSecurityReport> {
        let total_authentications = self.stats.successful_authentications + self.stats.failed_authentications;
        let success_rate = if total_authentications > 0 {
            (self.stats.successful_authentications as f64 / total_authentications as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(HardwareSecurityReport {
            generated_at: Utc::now(),
            total_keys_registered: self.stats.total_keys_registered,
            active_keys: self.stats.active_keys,
            keys_by_type: self.stats.keys_by_type.clone(),
            total_authentications,
            successful_authentications: self.stats.successful_authentications,
            failed_authentications: self.stats.failed_authentications,
            success_rate,
            last_authentication: self.stats.last_authentication,
            active_challenges_count: self.active_challenges.len() as u32,
            configuration: self.config.clone(),
        })
    }
}

/// Hardware security report
#[derive(Debug, Serialize, Deserialize)]
pub struct HardwareSecurityReport {
    pub generated_at: DateTime<Utc>,
    pub total_keys_registered: u32,
    pub active_keys: u32,
    pub keys_by_type: HashMap<HardwareKeyType, u32>,
    pub total_authentications: u64,
    pub successful_authentications: u64,
    pub failed_authentications: u64,
    pub success_rate: f64,
    pub last_authentication: Option<DateTime<Utc>>,
    pub active_challenges_count: u32,
    pub configuration: HardwareSecurityConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hardware_security_manager_creation() {
        let manager = HardwareSecurityManager::new();
        assert_eq!(manager.registered_keys.len(), 0);
        assert_eq!(manager.stats.total_keys_registered, 0);
    }
    
    #[tokio::test]
    async fn test_key_registration() {
        let mut manager = HardwareSecurityManager::new();
        
        let key = HardwareSecurityKey {
            id: "test_key_1".to_string(),
            name: "Test Security Key".to_string(),
            key_type: HardwareKeyType::Fido2,
            status: KeyStatus::Available,
            manufacturer: Some("Test Corp".to_string()),
            model: Some("TestKey 1.0".to_string()),
            firmware_version: Some("1.0.0".to_string()),
            serial_number: None,
            capabilities: vec!["webauthn".to_string()],
            registered_at: Utc::now(),
            last_used: None,
            use_count: 0,
            is_primary: false,
        };
        
        let key_id = manager.register_key(key).await.unwrap();
        assert_eq!(key_id, "test_key_1");
        assert_eq!(manager.registered_keys.len(), 1);
        assert_eq!(manager.stats.total_keys_registered, 1);
        
        // First key should be set as primary
        assert!(manager.registered_keys.get(&key_id).unwrap().is_primary);
    }
    
    #[tokio::test]
    async fn test_authentication_challenge_creation() {
        let mut manager = HardwareSecurityManager::new();
        
        // Register a key first
        let key = HardwareSecurityKey {
            id: "test_key_1".to_string(),
            name: "Test Security Key".to_string(),
            key_type: HardwareKeyType::Fido2,
            status: KeyStatus::Available,
            manufacturer: None,
            model: None,
            firmware_version: None,
            serial_number: None,
            capabilities: vec![],
            registered_at: Utc::now(),
            last_used: None,
            use_count: 0,
            is_primary: true,
        };
        
        manager.register_key(key).await.unwrap();
        
        // Create authentication challenge
        let challenge = manager.create_authentication_challenge(
            "test_key_1",
            "https://example.com".to_string(),
            true,
        ).await.unwrap();
        
        assert!(!challenge.challenge_data.is_empty());
        assert_eq!(challenge.key_id, "test_key_1");
        assert_eq!(challenge.origin, "https://example.com");
        assert!(challenge.user_verification);
        assert_eq!(manager.active_challenges.len(), 1);
    }
    
    #[tokio::test]
    async fn test_authentication_verification() {
        let mut manager = HardwareSecurityManager::new();
        
        // Register a key
        let key = HardwareSecurityKey {
            id: "test_key_1".to_string(),
            name: "Test Security Key".to_string(),
            key_type: HardwareKeyType::Fido2,
            status: KeyStatus::Available,
            manufacturer: None,
            model: None,
            firmware_version: None,
            serial_number: None,
            capabilities: vec![],
            registered_at: Utc::now(),
            last_used: None,
            use_count: 0,
            is_primary: true,
        };
        
        manager.register_key(key).await.unwrap();
        
        // Create challenge
        let challenge = manager.create_authentication_challenge(
            "test_key_1",
            "https://example.com".to_string(),
            true,
        ).await.unwrap();
        
        // Create response
        let response = AuthenticationResponse {
            challenge_id: challenge.id,
            signature: vec![1, 2, 3, 4], // Mock signature
            authenticator_data: vec![5, 6, 7, 8], // Mock authenticator data
            client_data_json: "{}".to_string(),
            user_handle: None,
        };
        
        // Verify authentication
        let is_valid = manager.verify_authentication(response).await.unwrap();
        assert!(is_valid);
        assert_eq!(manager.stats.successful_authentications, 1);
        assert_eq!(manager.active_challenges.len(), 0); // Challenge should be removed
    }
    
    #[tokio::test]
    async fn test_hardware_security_report() {
        let mut manager = HardwareSecurityManager::new();
        
        // Register a key
        let key = HardwareSecurityKey {
            id: "test_key_1".to_string(),
            name: "Test Security Key".to_string(),
            key_type: HardwareKeyType::Fido2,
            status: KeyStatus::Available,
            manufacturer: None,
            model: None,
            firmware_version: None,
            serial_number: None,
            capabilities: vec![],
            registered_at: Utc::now(),
            last_used: None,
            use_count: 0,
            is_primary: true,
        };
        
        manager.register_key(key).await.unwrap();
        
        let report = manager.generate_security_report().await.unwrap();
        
        assert_eq!(report.total_keys_registered, 1);
        assert_eq!(report.active_keys, 0); // No discovery run
        assert!(report.keys_by_type.contains_key(&HardwareKeyType::Fido2));
        assert_eq!(report.success_rate, 0.0); // No authentications yet
    }
}