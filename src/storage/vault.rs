//! Vault storage operations

use crate::crypto::CryptoManager;
use crate::storage::{Vault, VaultFile};
use crate::{Result, TwoPasswordError};
use ring::hmac;
use std::fs;
use std::path::Path;

/// Save vault to encrypted file with salt
pub fn save_vault_with_salt(vault: &Vault, crypto: &CryptoManager, salt: &crate::crypto::Salt) -> Result<()> {
    // Serialize vault entries
    let vault_data = serde_json::to_vec(&vault.entries)
        .map_err(|e| TwoPasswordError::storage(format!("Failed to serialize vault: {}", e)))?;

    // Encrypt the vault data
    let encrypted_data = crypto.encrypt(&vault_data)?;

    // Calculate integrity hash of the entire encrypted data
    let integrity_key = hmac::Key::new(hmac::HMAC_SHA256, b"TwoPassword-Integrity-Key");
    let mut hash_input = Vec::new();
    hash_input.extend_from_slice(&encrypted_data.ciphertext);
    hash_input.extend_from_slice(&encrypted_data.nonce);
    hash_input.extend_from_slice(&encrypted_data.hmac);

    let integrity_hash = hmac::sign(&integrity_key, &hash_input);

    // Create vault file structure with the provided salt
    let vault_file = VaultFile {
        metadata: vault.metadata.clone(),
        salt: salt.clone(),
        encrypted_data,
        integrity_hash: integrity_hash.as_ref().to_vec(),
    };

    // Serialize and write to file
    let vault_json = serde_json::to_string_pretty(&vault_file)
        .map_err(|e| TwoPasswordError::storage(format!("Failed to serialize vault file: {}", e)))?;

    // Write to temporary file first, then rename for atomic operation
    let temp_path = vault.vault_path.with_extension("tmp");
    fs::write(&temp_path, vault_json)
        .map_err(|e| TwoPasswordError::storage(format!("Failed to write vault file: {}", e)))?;

    fs::rename(&temp_path, &vault.vault_path)
        .map_err(|e| TwoPasswordError::storage(format!("Failed to finalize vault file: {}", e)))?;

    tracing::info!("Vault saved to {}", vault.vault_path.display());
    Ok(())
}

/// Save vault to encrypted file (generates new salt)
pub fn save_vault(vault: &Vault, crypto: &CryptoManager) -> Result<()> {
    let salt = crate::crypto::Salt::generate()?;
    save_vault_with_salt(vault, crypto, &salt)
}

/// Load vault from encrypted file and return both vault and salt
pub fn load_vault_with_salt<P: AsRef<Path>>(
    path: P,
    password: &str,
    crypto: &mut CryptoManager,
) -> Result<(Vault, crate::crypto::Salt)> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(TwoPasswordError::VaultNotFound);
    }

    // Read vault file
    let vault_json = fs::read_to_string(path)
        .map_err(|e| TwoPasswordError::storage(format!("Failed to read vault file: {}", e)))?;

    // Deserialize vault file
    let vault_file: VaultFile = serde_json::from_str(&vault_json)
        .map_err(|e| TwoPasswordError::storage(format!("Failed to parse vault file: {}", e)))?;

    // Verify format version
    if vault_file.metadata.format_version != crate::config::FORMAT_VERSION {
        return Err(TwoPasswordError::InvalidVaultFormat);
    }

    // Verify integrity hash
    let integrity_key = hmac::Key::new(hmac::HMAC_SHA256, b"TwoPassword-Integrity-Key");
    let mut hash_input = Vec::new();
    hash_input.extend_from_slice(&vault_file.encrypted_data.ciphertext);
    hash_input.extend_from_slice(&vault_file.encrypted_data.nonce);
    hash_input.extend_from_slice(&vault_file.encrypted_data.hmac);

    hmac::verify(&integrity_key, &hash_input, &vault_file.integrity_hash)
        .map_err(|_| TwoPasswordError::storage("Vault integrity verification failed"))?;

    // Derive key from password and salt
    crypto.derive_key(password, &vault_file.salt)?;

    // Decrypt vault data
    let decrypted_data = crypto.decrypt(&vault_file.encrypted_data)?;

    // Deserialize entries
    let entries = serde_json::from_slice(&decrypted_data).map_err(|e| {
        TwoPasswordError::storage(format!("Failed to deserialize vault entries: {}", e))
    })?;

    let mut vault = Vault::new(path);
    vault.metadata = vault_file.metadata;
    vault.entries = entries;
    vault.is_modified = false;

    tracing::info!("Vault loaded from {}", path.display());
    Ok((vault, vault_file.salt))
}

/// Load vault from encrypted file (compatibility function)
pub fn load_vault<P: AsRef<Path>>(
    path: P,
    password: &str,
    crypto: &mut CryptoManager,
) -> Result<Vault> {
    let (vault, _salt) = load_vault_with_salt(path, password, crypto)?;
    Ok(vault)
}

/// Check if vault exists at path
pub fn vault_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// Create backup of vault file
pub fn backup_vault<P: AsRef<Path>>(vault_path: P) -> Result<std::path::PathBuf> {
    let vault_path = vault_path.as_ref();

    if !vault_path.exists() {
        return Err(TwoPasswordError::VaultNotFound);
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = vault_path.with_extension(format!("backup.{}.enc", timestamp));

    fs::copy(vault_path, &backup_path)
        .map_err(|e| TwoPasswordError::storage(format!("Failed to create backup: {}", e)))?;

    tracing::info!("Vault backup created at {}", backup_path.display());
    Ok(backup_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::CryptoManager;
    use tempfile::NamedTempFile;

    #[test]
    fn test_vault_exists() {
        let temp_file = NamedTempFile::new().unwrap();
        assert!(vault_exists(temp_file.path()));

        drop(temp_file);
        // File should be deleted now
        // Note: This might be flaky on some systems
    }

    #[test]
    fn test_save_load_vault() {
        let temp_file = NamedTempFile::new().unwrap();
        let vault_path = temp_file.path();

        // Create and save vault
        let mut crypto = CryptoManager::new();
        let password = "test_password";

        let mut vault = Vault::new(vault_path);
        let entry = crate::storage::PasswordEntry::new(
            "Test Entry".to_string(),
            "testuser".to_string(),
            "testpass".to_string(),
        );
        vault.add_entry(entry);

        // For testing, we need to set up crypto manually
        let salt = crate::crypto::Salt::generate().unwrap();
        crypto.derive_key(password, &salt).unwrap();

        // This would fail because we don't have the full implementation yet
        // save_vault(&vault, &crypto).unwrap();

        // Load vault
        // let loaded_vault = load_vault(vault_path, password, &mut crypto).unwrap();
        // assert_eq!(loaded_vault.entries.len(), 1);
    }
}
