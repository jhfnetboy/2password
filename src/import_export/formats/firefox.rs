//! Firefox JSON format import support

use crate::storage::PasswordEntry;
use crate::{Result, TwoPasswordError};
use serde::Deserialize;

use super::super::ImportResult;

/// Firefox password export structure
#[derive(Debug, Clone, Deserialize)]
struct FirefoxExport {
    logins: Vec<FirefoxLogin>,
}

#[derive(Debug, Clone, Deserialize)]
struct FirefoxLogin {
    hostname: String,
    #[serde(rename = "httpRealm")]
    http_realm: Option<String>,
    #[serde(rename = "formSubmitURL")]
    form_submit_url: Option<String>,
    #[serde(rename = "usernameField")]
    username_field: Option<String>,
    #[serde(rename = "passwordField")]
    password_field: Option<String>,
    #[serde(rename = "encryptedUsername")]
    encrypted_username: Option<String>,
    #[serde(rename = "encryptedPassword")]
    encrypted_password: Option<String>,
    username: Option<String>,
    password: Option<String>,
    guid: Option<String>,
    #[serde(rename = "timeCreated")]
    time_created: Option<u64>,
    #[serde(rename = "timeLastUsed")]
    time_last_used: Option<u64>,
    #[serde(rename = "timePasswordChanged")]
    time_password_changed: Option<u64>,
    #[serde(rename = "timesUsed")]
    times_used: Option<u32>,
}

impl FirefoxLogin {
    fn to_password_entry(&self) -> Option<PasswordEntry> {
        let username = self.username.as_ref()?.clone();
        let password = self.password.as_ref()?.clone();
        
        if password.is_empty() {
            return None;
        }

        let title = extract_domain_from_url(&self.hostname);
        let url = if self.hostname.is_empty() {
            None
        } else {
            Some(self.hostname.clone())
        };

        let tags = vec!["Firefox".to_string()];

        Some(PasswordEntry::new_with_fields(
            title,
            username,
            password,
            url,
            None, // Firefox exports don't include notes
            tags,
        ))
    }
}

/// Extract domain name from URL for title
fn extract_domain_from_url(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(host) = parsed.host_str() {
            return host.trim_start_matches("www.").to_string();
        }
    }
    
    url.to_string()
}

/// Import Firefox JSON format
pub fn import_firefox_json(content: &str, result: &mut ImportResult) -> Result<Vec<PasswordEntry>> {
    let export: FirefoxExport = serde_json::from_str(content)
        .map_err(|e| TwoPasswordError::ImportExportError(format!("Invalid Firefox JSON: {}", e)))?;

    let mut entries = Vec::new();
    let mut skipped_encrypted = 0;

    for login in export.logins {
        // Skip encrypted entries (would need Firefox to decrypt)
        if login.encrypted_username.is_some() || login.encrypted_password.is_some() {
            skipped_encrypted += 1;
            continue;
        }

        if let Some(entry) = login.to_password_entry() {
            entries.push(entry);
        }
    }

    if skipped_encrypted > 0 {
        result.add_warning(format!("Skipped {} encrypted entries - Firefox must export unencrypted passwords", skipped_encrypted));
    }

    result.add_warning(format!("Successfully processed {} Firefox entries", entries.len()));
    Ok(entries)
}