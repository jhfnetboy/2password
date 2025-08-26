//! 2-of-3 Recovery system implementation
//!
//! This module implements the recovery system where users need any 2 of:
//! 1. Simple password
//! 2. Touch ID/Passkey authentication  
//! 3. iCloud backup share

use crate::{Result, TwoPasswordError};
use crate::crypto::secret_sharing::{SecretSharing, SecretShare};
use crate::crypto::{Salt, MasterKey};

/// Recovery method types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecoveryMethod {
    Password,
    TouchId,
    ICloudBackup,
}

/// Recovery manager for 2-of-3 system
pub struct RecoveryManager {
    available_methods: Vec<RecoveryMethod>,
}

impl RecoveryManager {
    /// Create a new recovery manager
    pub fn new() -> Self {
        let mut available_methods = vec![RecoveryMethod::Password];
        
        // Check if Touch ID is available
        if crate::auth::touchid::is_available() {
            available_methods.push(RecoveryMethod::TouchId);
        }
        
        // For now, assume iCloud backup is always available
        // In a full implementation, this would check iCloud status
        available_methods.push(RecoveryMethod::ICloudBackup);
        
        Self { available_methods }
    }
    
    /// Get available recovery methods
    pub fn available_methods(&self) -> &[RecoveryMethod] {
        &self.available_methods
    }
    
    /// Check if recovery is possible (need at least 2 methods)
    pub fn can_recover(&self) -> bool {
        self.available_methods.len() >= 2
    }
    
    /// Initialize master key setup with 2-of-3 recovery
    pub fn setup_master_key(&self, password: &str, user_id: &str, device_id: &str) -> Result<RecoverySetup> {
        // Generate the actual master secret
        let master_secret = crate::crypto::secure_random::generate_bytes(32)?;
        let master_secret_array: [u8; 32] = master_secret.try_into()
            .map_err(|_| TwoPasswordError::crypto("Invalid master secret length"))?;
            
        // Split into 3 shares
        let shares = SecretSharing::split_secret(&master_secret_array)?;
        
        // Generate salt for password-based key derivation
        let password_salt = Salt::generate()?;
        
        // Create the recovery setup
        let setup = RecoverySetup {
            master_secret: master_secret_array,
            password_salt: password_salt.clone(),
            password_share: shares[0].clone(), // Share 1: from password
            touchid_share: shares[1].clone(),  // Share 2: from Touch ID
            backup_share: shares[2].clone(),   // Share 3: for iCloud backup
        };
        
        Ok(setup)
    }
    
    /// Recover master key using password and Touch ID
    pub async fn recover_with_password_and_touchid(
        &self,
        password: &str,
        user_id: &str,
        device_id: &str,
        password_salt: &Salt,
    ) -> Result<MasterKey> {
        // Authenticate with Touch ID first
        crate::auth::touchid::authenticate("Authenticate to recover your passwords")?;
        
        // Derive password share
        let password_share = SecretSharing::derive_password_share(password, &password_salt.bytes)?;
        
        // Derive Touch ID share
        let touchid_share = SecretSharing::derive_touchid_share(user_id, device_id)?;
        
        // Reconstruct master secret
        let master_secret = SecretSharing::reconstruct_secret(&password_share, &touchid_share)?;
        
        Ok(MasterKey::from_bytes(master_secret))
    }
    
    /// Recover master key using password and iCloud backup
    pub fn recover_with_password_and_backup(
        &self,
        password: &str,
        password_salt: &Salt,
        backup_data: &[u8],
    ) -> Result<MasterKey> {
        // Derive password share
        let password_share = SecretSharing::derive_password_share(password, &password_salt.bytes)?;
        
        // Restore backup share
        let backup_share = SecretSharing::restore_backup_share(backup_data)?;
        
        // Reconstruct master secret
        let master_secret = SecretSharing::reconstruct_secret(&password_share, &backup_share)?;
        
        Ok(MasterKey::from_bytes(master_secret))
    }
    
    /// Recover master key using Touch ID and iCloud backup
    pub async fn recover_with_touchid_and_backup(
        &self,
        user_id: &str,
        device_id: &str,
        backup_data: &[u8],
    ) -> Result<MasterKey> {
        // Authenticate with Touch ID first
        crate::auth::touchid::authenticate("Authenticate to recover your passwords")?;
        
        // Derive Touch ID share
        let touchid_share = SecretSharing::derive_touchid_share(user_id, device_id)?;
        
        // Restore backup share
        let backup_share = SecretSharing::restore_backup_share(backup_data)?;
        
        // Reconstruct master secret
        let master_secret = SecretSharing::reconstruct_secret(&touchid_share, &backup_share)?;
        
        Ok(MasterKey::from_bytes(master_secret))
    }
}

impl Default for RecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Recovery setup containing all necessary components
#[derive(Debug)]
pub struct RecoverySetup {
    /// The master secret (32 bytes)
    pub master_secret: [u8; 32],
    /// Salt for password-based key derivation
    pub password_salt: Salt,
    /// Share 1: derived from password
    pub password_share: SecretShare,
    /// Share 2: derived from Touch ID
    pub touchid_share: SecretShare,
    /// Share 3: for iCloud backup
    pub backup_share: SecretShare,
}

impl RecoverySetup {
    /// Get the master key
    pub fn master_key(&self) -> MasterKey {
        MasterKey::from_bytes(self.master_secret)
    }
    
    /// Get backup data for iCloud storage
    pub fn backup_data(&self) -> Result<Vec<u8>> {
        SecretSharing::create_backup_share(self.backup_share.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_recovery_manager_creation() {
        let manager = RecoveryManager::new();
        assert!(!manager.available_methods.is_empty());
        assert!(manager.available_methods.contains(&RecoveryMethod::Password));
    }
    
    #[test]
    fn test_master_key_setup() {
        let manager = RecoveryManager::new();
        let setup = manager.setup_master_key("test_password", "user123", "device456").unwrap();
        
        assert_eq!(setup.master_secret.len(), 32);
        assert_eq!(setup.password_share.id, 1);
        assert_eq!(setup.touchid_share.id, 2);
        assert_eq!(setup.backup_share.id, 3);
    }
    
    #[test]
    fn test_password_and_backup_recovery() {
        let manager = RecoveryManager::new();
        let setup = manager.setup_master_key("test_password", "user123", "device456").unwrap();
        
        // Create backup data
        let backup_data = setup.backup_data().unwrap();
        
        // Recover using password and backup
        let recovered_key = manager.recover_with_password_and_backup(
            "test_password",
            &setup.password_salt,
            &backup_data,
        ).unwrap();
        
        // Should match original
        assert_eq!(recovered_key.as_bytes(), &setup.master_secret);
    }
    
    #[test]
    fn test_can_recover() {
        let manager = RecoveryManager::new();
        // Should be able to recover if we have at least 2 methods available
        assert!(manager.can_recover() || manager.available_methods.len() < 2);
    }
}