//! Chrome password CSV format import support

use crate::storage::PasswordEntry;
use crate::{Result, TwoPasswordError};
use csv::Reader;
use serde::Deserialize;
use std::io::Cursor;

use super::super::ImportResult;

/// Chrome password CSV record structure
#[derive(Debug, Clone, Deserialize)]
struct ChromeRecord {
    name: String,
    url: String,
    username: String,
    password: String,
}

impl ChromeRecord {
    fn to_password_entry(&self) -> PasswordEntry {
        let title = if self.name.trim().is_empty() {
            extract_domain_from_url(&self.url)
        } else {
            self.name.clone()
        };

        let url = if self.url.is_empty() || self.url == "http://" {
            None
        } else {
            Some(self.url.clone())
        };

        let tags = vec!["Chrome".to_string()];

        PasswordEntry::new_with_fields(
            title,
            self.username.clone(),
            self.password.clone(),
            url,
            None, // Chrome exports don't typically include notes
            tags,
        )
    }

    fn validate(&self) -> std::result::Result<(), String> {
        if self.username.trim().is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if self.password.trim().is_empty() {
            return Err("Password cannot be empty".to_string());
        }
        if self.url.trim().is_empty() {
            return Err("URL cannot be empty for Chrome entries".to_string());
        }
        Ok(())
    }
}

/// Extract domain name from URL for title
fn extract_domain_from_url(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(host) = parsed.host_str() {
            return host.trim_start_matches("www.").to_string();
        }
    }
    
    // Fallback: simple extraction
    if let Some(start) = url.find("://") {
        let after_protocol = &url[start + 3..];
        if let Some(end) = after_protocol.find('/') {
            after_protocol[..end].trim_start_matches("www.").to_string()
        } else {
            after_protocol.trim_start_matches("www.").to_string()
        }
    } else {
        url.to_string()
    }
}

/// Import Chrome password CSV format
pub fn import_chrome_csv(content: &str, result: &mut ImportResult) -> Result<Vec<PasswordEntry>> {
    let mut reader = Reader::from_reader(Cursor::new(content));
    let mut entries = Vec::new();

    // Check if this looks like Chrome format
    if let Ok(headers) = reader.headers() {
        let header_str = headers.iter().collect::<Vec<_>>().join(",").to_lowercase();
        if !header_str.contains("name,url,username,password") {
            result.add_warning("File does not appear to be in Chrome CSV format".to_string());
        }
    }

    for (line_num, record_result) in reader.deserialize().enumerate() {
        match record_result {
            Ok(record) => {
                let chrome_record: ChromeRecord = record;
                
                if let Err(validation_error) = chrome_record.validate() {
                    result.add_error(format!("Line {}: {}", line_num + 2, validation_error));
                    continue;
                }

                entries.push(chrome_record.to_password_entry());
            }
            Err(e) => {
                result.add_error(format!("Line {}: Failed to parse record - {}", line_num + 2, e));
            }
        }
    }

    result.add_warning(format!("Successfully processed {} Chrome entries", entries.len()));
    Ok(entries)
}