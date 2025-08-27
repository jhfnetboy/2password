//! LastPass CSV format import support

use crate::storage::PasswordEntry;
use crate::{Result, TwoPasswordError};
use csv::Reader;
use serde::Deserialize;
use std::io::Cursor;

use super::super::ImportResult;

/// LastPass CSV record structure
#[derive(Debug, Clone, Deserialize)]
struct LastPassRecord {
    url: String,
    username: String,
    password: String,
    extra: Option<String>,
    name: Option<String>,
    grouping: Option<String>,
    fav: Option<String>,
}

impl LastPassRecord {
    fn to_password_entry(&self) -> PasswordEntry {
        let title = self.name.as_ref()
            .filter(|n| !n.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| {
                if !self.url.is_empty() {
                    extract_domain_from_url(&self.url)
                } else {
                    self.username.clone()
                }
            });

        let url = if self.url.is_empty() || self.url == "http://" {
            None
        } else {
            Some(self.url.clone())
        };

        let mut tags = Vec::new();
        if let Some(ref grouping) = self.grouping {
            if !grouping.trim().is_empty() && grouping != "LastPass" {
                tags.push(grouping.clone());
            }
        }

        PasswordEntry::new_with_fields(
            title,
            self.username.clone(),
            self.password.clone(),
            url,
            self.extra.clone(),
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
    
    // Fallback: simple regex-like extraction
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

/// Import LastPass CSV format
pub fn import_lastpass_csv(content: &str, result: &mut ImportResult) -> Result<Vec<PasswordEntry>> {
    let mut reader = Reader::from_reader(Cursor::new(content));
    let mut entries = Vec::new();

    // Check if this looks like LastPass format
    if let Ok(headers) = reader.headers() {
        let header_str = headers.iter().collect::<Vec<_>>().join(",").to_lowercase();
        if !header_str.contains("url,username,password") {
            result.add_warning("File does not appear to be in LastPass CSV format".to_string());
        }
    }

    for (line_num, record_result) in reader.deserialize().enumerate() {
        match record_result {
            Ok(record) => {
                let lastpass_record: LastPassRecord = record;
                
                if let Err(validation_error) = lastpass_record.validate() {
                    result.add_error(format!("Line {}: {}", line_num + 2, validation_error));
                    continue;
                }

                // Skip empty or placeholder entries
                if lastpass_record.username == "null" || lastpass_record.password == "null" {
                    result.add_warning(format!("Line {}: Skipping null entry", line_num + 2));
                    continue;
                }

                entries.push(lastpass_record.to_password_entry());
            }
            Err(e) => {
                result.add_error(format!("Line {}: Failed to parse record - {}", line_num + 2, e));
            }
        }
    }

    result.add_warning(format!("Successfully processed {} LastPass entries", entries.len()));
    Ok(entries)
}