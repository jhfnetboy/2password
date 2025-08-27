//! Bitwarden JSON format import/export support

use crate::storage::PasswordEntry;
use crate::{Result, TwoPasswordError};
use serde::{Deserialize, Serialize};

use super::super::{ExportOptions, ImportResult};

/// Bitwarden export format structure
#[derive(Debug, Clone, Deserialize)]
struct BitwardenExport {
    items: Vec<BitwardenItem>,
    folders: Option<Vec<BitwardenFolder>>,
}

#[derive(Debug, Clone, Deserialize)]
struct BitwardenItem {
    #[serde(rename = "type")]
    item_type: u32, // 1 = login, 2 = secure note, 3 = card, 4 = identity
    name: String,
    notes: Option<String>,
    login: Option<BitwardenLogin>,
    #[serde(rename = "folderId")]
    folder_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct BitwardenLogin {
    username: Option<String>,
    password: Option<String>,
    uris: Option<Vec<BitwardenUri>>,
}

#[derive(Debug, Clone, Deserialize)]
struct BitwardenUri {
    uri: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct BitwardenFolder {
    id: String,
    name: String,
}

/// Bitwarden export format for our exports
#[derive(Debug, Clone, Serialize)]
struct BitwardenExportOut {
    encrypted: bool,
    items: Vec<BitwardenItemOut>,
    folders: Vec<BitwardenFolderOut>,
}

#[derive(Debug, Clone, Serialize)]
struct BitwardenItemOut {
    id: String,
    #[serde(rename = "type")]
    item_type: u32,
    name: String,
    notes: Option<String>,
    login: BitwardenLoginOut,
    #[serde(rename = "folderId")]
    folder_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct BitwardenLoginOut {
    username: Option<String>,
    password: Option<String>,
    uris: Vec<BitwardenUriOut>,
}

#[derive(Debug, Clone, Serialize)]
struct BitwardenUriOut {
    #[serde(rename = "match")]
    match_type: Option<u32>,
    uri: String,
}

#[derive(Debug, Clone, Serialize)]
struct BitwardenFolderOut {
    id: String,
    name: String,
}

impl BitwardenItem {
    fn to_password_entry(&self, folders: &std::collections::HashMap<String, String>) -> Option<PasswordEntry> {
        // Only process login items
        if self.item_type != 1 {
            return None;
        }

        let login = self.login.as_ref()?;
        
        let username = login.username.as_deref().unwrap_or("").to_string();
        let password = login.password.as_deref().unwrap_or("").to_string();
        
        if password.is_empty() {
            return None;
        }

        let url = login.uris
            .as_ref()
            .and_then(|uris| uris.first())
            .and_then(|uri| uri.uri.as_ref())
            .filter(|u| !u.trim().is_empty())
            .map(|u| u.clone());

        let mut tags = vec!["Bitwarden".to_string()];
        
        // Add folder as tag if present
        if let Some(ref folder_id) = self.folder_id {
            if let Some(folder_name) = folders.get(folder_id) {
                tags.push(folder_name.clone());
            }
        }

        Some(PasswordEntry::new_with_fields(
            self.name.clone(),
            username,
            password,
            url,
            self.notes.clone(),
            tags,
        ))
    }
}

/// Import Bitwarden JSON format
pub fn import_bitwarden_json(content: &str, result: &mut ImportResult) -> Result<Vec<PasswordEntry>> {
    let export: BitwardenExport = serde_json::from_str(content)
        .map_err(|e| TwoPasswordError::ImportExportError(format!("Invalid Bitwarden JSON: {}", e)))?;

    // Build folder lookup map
    let folders: std::collections::HashMap<String, String> = export
        .folders
        .unwrap_or_default()
        .into_iter()
        .map(|f| (f.id, f.name))
        .collect();

    let mut entries = Vec::new();
    let mut skipped_types = std::collections::HashMap::new();

    for (index, item) in export.items.into_iter().enumerate() {
        if let Some(entry) = item.to_password_entry(&folders) {
            entries.push(entry);
        } else {
            // Count skipped item types
            *skipped_types.entry(item.item_type).or_insert(0) += 1;
        }
    }

    // Add warnings for skipped items
    for (item_type, count) in skipped_types {
        let type_name = match item_type {
            1 => "Login",
            2 => "Secure Note", 
            3 => "Card",
            4 => "Identity",
            _ => "Unknown",
        };
        result.add_warning(format!("Skipped {} {} items (not supported)", count, type_name));
    }

    result.add_warning(format!("Successfully processed {} Bitwarden login entries", entries.len()));
    Ok(entries)
}

/// Export to Bitwarden JSON format
pub fn export_bitwarden_json(entries: &[PasswordEntry], options: &ExportOptions) -> Result<String> {
    let mut items = Vec::new();
    let mut folders = std::collections::HashMap::new();
    let mut folder_counter = 1;

    // Create folders for tags and collect items
    for (index, entry) in entries.iter().enumerate() {
        let mut folder_id = None;
        
        // Use first tag as folder
        if let Some(first_tag) = entry.tags.first() {
            if !folders.contains_key(first_tag) {
                folders.insert(first_tag.clone(), format!("folder-{}", folder_counter));
                folder_counter += 1;
            }
            folder_id = folders.get(first_tag).cloned();
        }

        let uris = if let Some(ref url) = entry.url {
            vec![BitwardenUriOut {
                match_type: None,
                uri: url.clone(),
            }]
        } else {
            Vec::new()
        };

        let item = BitwardenItemOut {
            id: format!("item-{}", index + 1),
            item_type: 1, // Login
            name: entry.title.clone(),
            notes: if options.include_notes { entry.notes.clone() } else { None },
            login: BitwardenLoginOut {
                username: Some(entry.username.clone()),
                password: if options.include_passwords { 
                    Some(entry.password.clone()) 
                } else { 
                    Some("***HIDDEN***".to_string()) 
                },
                uris,
            },
            folder_id,
        };

        items.push(item);
    }

    let folder_list: Vec<BitwardenFolderOut> = folders
        .into_iter()
        .map(|(name, id)| BitwardenFolderOut { id, name })
        .collect();

    let export = BitwardenExportOut {
        encrypted: false,
        items,
        folders: folder_list,
    };

    serde_json::to_string_pretty(&export)
        .map_err(|e| TwoPasswordError::ImportExportError(format!("Failed to serialize Bitwarden JSON: {}", e)))
}