//! Storage management for TwoPassword
//!
//! Handles encrypted vault storage and password entry management

use crate::crypto::{CryptoManager, EncryptedData, Salt};
use crate::{Result, TwoPasswordError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub mod entry;
pub mod vault;

/// A single password entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntry {
    pub id: Uuid,
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl PasswordEntry {
    /// Create a new password entry
    pub fn new(title: String, username: String, password: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            username,
            password,
            url: None,
            notes: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the password entry
    pub fn update(&mut self) {
        self.updated_at = chrono::Utc::now();
    }
}

/// Vault metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMetadata {
    pub format_version: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub entry_count: usize,
}

/// Encrypted vault structure stored on disk
#[derive(Debug, Serialize, Deserialize)]
pub struct VaultFile {
    pub metadata: VaultMetadata,
    pub salt: Salt,
    pub encrypted_data: EncryptedData,
    pub integrity_hash: Vec<u8>,
}

/// In-memory vault with decrypted entries
#[derive(Debug)]
pub struct Vault {
    pub metadata: VaultMetadata,
    pub entries: HashMap<Uuid, PasswordEntry>,
    pub vault_path: PathBuf,
    pub is_modified: bool,
}

impl Vault {
    /// Create a new empty vault
    pub fn new<P: AsRef<Path>>(vault_path: P) -> Self {
        let metadata = VaultMetadata {
            format_version: crate::config::FORMAT_VERSION,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            entry_count: 0,
        };

        Self {
            metadata,
            entries: HashMap::new(),
            vault_path: vault_path.as_ref().to_path_buf(),
            is_modified: false,
        }
    }

    /// Add a new entry to the vault
    pub fn add_entry(&mut self, entry: PasswordEntry) {
        self.entries.insert(entry.id, entry);
        self.metadata.entry_count = self.entries.len();
        self.metadata.updated_at = chrono::Utc::now();
        self.is_modified = true;
    }

    /// Remove an entry from the vault
    pub fn remove_entry(&mut self, id: &Uuid) -> Result<PasswordEntry> {
        let entry = self
            .entries
            .remove(id)
            .ok_or_else(|| TwoPasswordError::EntryNotFound(id.to_string()))?;

        self.metadata.entry_count = self.entries.len();
        self.metadata.updated_at = chrono::Utc::now();
        self.is_modified = true;

        Ok(entry)
    }

    /// Get an entry by ID
    pub fn get_entry(&self, id: &Uuid) -> Option<&PasswordEntry> {
        self.entries.get(id)
    }

    /// Get a mutable entry by ID
    pub fn get_entry_mut(&mut self, id: &Uuid) -> Option<&mut PasswordEntry> {
        if let Some(entry) = self.entries.get_mut(id) {
            self.is_modified = true;
            self.metadata.updated_at = chrono::Utc::now();
            Some(entry)
        } else {
            None
        }
    }

    /// Search entries by title
    pub fn search_by_title(&self, query: &str) -> Vec<&PasswordEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .values()
            .filter(|entry| entry.title.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Get all entries
    pub fn get_all_entries(&self) -> Vec<&PasswordEntry> {
        self.entries.values().collect()
    }
}

/// Main vault manager
pub struct VaultManager {
    crypto: CryptoManager,
    current_vault: Option<Vault>,
    current_salt: Option<Salt>,
}

impl VaultManager {
    /// Create a new vault manager
    pub fn new() -> Self {
        Self {
            crypto: CryptoManager::new(),
            current_vault: None,
            current_salt: None,
        }
    }

    /// Create a new vault at the specified path
    pub fn create_vault<P: AsRef<Path>>(&mut self, path: P, password: &str) -> Result<()> {
        let salt = Salt::generate()?;
        self.crypto.derive_key(password, &salt)?;

        let vault = Vault::new(path);
        self.current_vault = Some(vault);
        self.current_salt = Some(salt);
        self.save_vault()?;

        Ok(())
    }

    /// Load an existing vault
    pub fn load_vault<P: AsRef<Path>>(&mut self, path: P, password: &str) -> Result<()> {
        let (vault, salt) = vault::load_vault_with_salt(path, password, &mut self.crypto)?;
        self.current_vault = Some(vault);
        self.current_salt = Some(salt);
        Ok(())
    }

    /// Save the current vault
    pub fn save_vault(&mut self) -> Result<()> {
        if let (Some(ref vault), Some(ref salt)) = (&self.current_vault, &self.current_salt) {
            vault::save_vault_with_salt(vault, &self.crypto, salt)?;
            // Mark as saved
            if let Some(ref mut vault) = self.current_vault {
                vault.is_modified = false;
            }
        }
        Ok(())
    }

    /// Get the current vault
    pub fn get_vault(&self) -> Option<&Vault> {
        self.current_vault.as_ref()
    }

    /// Get the current vault mutably
    pub fn get_vault_mut(&mut self) -> Option<&mut Vault> {
        self.current_vault.as_mut()
    }

    /// Check if a vault is loaded
    pub fn is_vault_loaded(&self) -> bool {
        self.current_vault.is_some()
    }

    /// Close the current vault
    pub fn close_vault(&mut self) {
        self.current_vault = None;
        self.current_salt = None;
        self.crypto.clear_master_key();
    }
}

impl Default for VaultManager {
    fn default() -> Self {
        Self::new()
    }
}
