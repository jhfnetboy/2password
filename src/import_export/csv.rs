//! Generic CSV import/export functionality

use crate::storage::PasswordEntry;
use crate::{Result, TwoPasswordError};
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

use super::{ExportOptions, ImportResult};

/// CSV record structure for import/export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvRecord {
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "URL")]
    pub url: Option<String>,
    #[serde(rename = "Username")]
    pub username: String,
    #[serde(rename = "Password")]
    pub password: String,
    #[serde(rename = "Notes")]
    pub notes: Option<String>,
    #[serde(rename = "Tags")]
    pub tags: Option<String>,
}

impl From<&PasswordEntry> for CsvRecord {
    fn from(entry: &PasswordEntry) -> Self {
        Self {
            title: entry.title.clone(),
            url: entry.url.clone(),
            username: entry.username.clone(),
            password: entry.password.clone(),
            notes: entry.notes.clone(),
            tags: if entry.tags.is_empty() {
                None
            } else {
                Some(entry.tags.join(", "))
            },
        }
    }
}

impl CsvRecord {
    /// Convert CSV record to PasswordEntry
    pub fn to_password_entry(&self) -> PasswordEntry {
        let tags = self.tags
            .as_ref()
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        PasswordEntry::new_with_fields(
            self.title.clone(),
            self.username.clone(),
            self.password.clone(),
            self.url.clone(),
            self.notes.clone(),
            tags,
        )
    }

    /// Validate CSV record data
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Title is required".to_string());
        }
        if self.username.trim().is_empty() {
            return Err("Username is required".to_string());
        }
        if self.password.trim().is_empty() {
            return Err("Password is required".to_string());
        }
        Ok(())
    }
}

/// Import passwords from CSV format
pub fn import_csv(content: &str, result: &mut ImportResult) -> Result<Vec<PasswordEntry>> {
    let mut reader = Reader::from_reader(Cursor::new(content));
    let mut entries = Vec::new();

    // Try to detect headers
    let headers = reader.headers()
        .map_err(|e| TwoPasswordError::ImportExportError(format!("Failed to read CSV headers: {}", e)))?;

    // Check if this looks like a password CSV
    let header_str = headers.iter().collect::<Vec<_>>().join(",").to_lowercase();
    if !header_str.contains("password") && !header_str.contains("login") {
        result.add_warning("CSV does not appear to contain password data".to_string());
    }

    // Try different parsing strategies
    if let Ok(records) = try_parse_structured_csv(content) {
        // Structured CSV with proper headers
        for (line_num, record_result) in records.into_iter().enumerate() {
            match record_result {
                Ok(record) => {
                    if let Err(validation_error) = record.validate() {
                        result.add_error(format!("Line {}: {}", line_num + 2, validation_error));
                        continue;
                    }
                    entries.push(record.to_password_entry());
                }
                Err(e) => {
                    result.add_error(format!("Line {}: {}", line_num + 2, e));
                }
            }
        }
    } else {
        // Fallback: try to parse as generic CSV
        entries = try_parse_generic_csv(content, result)?;
    }

    if entries.is_empty() && result.errors.is_empty() {
        result.add_error("No valid password entries found in CSV file".to_string());
    }

    Ok(entries)
}

/// Try to parse CSV with structured headers
fn try_parse_structured_csv(content: &str) -> std::result::Result<Vec<std::result::Result<CsvRecord, String>>, csv::Error> {
    let mut reader = Reader::from_reader(Cursor::new(content));
    let mut records = Vec::new();

    for result in reader.deserialize() {
        match result {
            Ok(record) => records.push(Ok(record)),
            Err(e) => records.push(Err(e.to_string())),
        }
    }

    Ok(records)
}

/// Try to parse as generic CSV with flexible column mapping
fn try_parse_generic_csv(content: &str, result: &mut ImportResult) -> Result<Vec<PasswordEntry>> {
    let mut reader = Reader::from_reader(Cursor::new(content));
    let mut entries = Vec::new();

    let headers = reader.headers()
        .map_err(|e| TwoPasswordError::ImportExportError(format!("Failed to read CSV headers: {}", e)))?
        .clone();

    // Map column indices based on common header names
    let column_mapping = map_columns(&headers);
    
    for (line_num, record_result) in reader.records().enumerate() {
        match record_result {
            Ok(record) => {
                match parse_generic_record(&record, &column_mapping) {
                    Ok(entry) => entries.push(entry),
                    Err(e) => result.add_error(format!("Line {}: {}", line_num + 2, e)),
                }
            }
            Err(e) => {
                result.add_error(format!("Line {}: {}", line_num + 2, e));
            }
        }
    }

    Ok(entries)
}

/// Map CSV columns to our fields based on header names
#[derive(Debug)]
struct ColumnMapping {
    title: Option<usize>,
    url: Option<usize>,
    username: Option<usize>,
    password: Option<usize>,
    notes: Option<usize>,
    tags: Option<usize>,
}

fn map_columns(headers: &csv::StringRecord) -> ColumnMapping {
    let mut mapping = ColumnMapping {
        title: None,
        url: None,
        username: None,
        password: None,
        notes: None,
        tags: None,
    };

    for (index, header) in headers.iter().enumerate() {
        let header_lower = header.to_lowercase();
        
        if mapping.title.is_none() && (header_lower.contains("title") || header_lower.contains("name") || header_lower.contains("site")) {
            mapping.title = Some(index);
        } else if mapping.url.is_none() && (header_lower.contains("url") || header_lower.contains("website") || header_lower.contains("site")) {
            mapping.url = Some(index);
        } else if mapping.username.is_none() && (header_lower.contains("username") || header_lower.contains("email") || header_lower.contains("login")) {
            mapping.username = Some(index);
        } else if mapping.password.is_none() && header_lower.contains("password") {
            mapping.password = Some(index);
        } else if mapping.notes.is_none() && (header_lower.contains("notes") || header_lower.contains("comment")) {
            mapping.notes = Some(index);
        } else if mapping.tags.is_none() && (header_lower.contains("tags") || header_lower.contains("categories") || header_lower.contains("folder")) {
            mapping.tags = Some(index);
        }
    }

    mapping
}

/// Parse a generic CSV record using column mapping
fn parse_generic_record(record: &csv::StringRecord, mapping: &ColumnMapping) -> std::result::Result<PasswordEntry, String> {
    let title = mapping.title
        .and_then(|i| record.get(i))
        .unwrap_or("Unknown")
        .trim()
        .to_string();

    let username = mapping.username
        .and_then(|i| record.get(i))
        .ok_or("Username column not found")?
        .trim()
        .to_string();

    let password = mapping.password
        .and_then(|i| record.get(i))
        .ok_or("Password column not found")?
        .trim()
        .to_string();

    if title.is_empty() && username.is_empty() {
        return Err("Both title and username cannot be empty".to_string());
    }

    if password.is_empty() {
        return Err("Password cannot be empty".to_string());
    }

    let url = mapping.url
        .and_then(|i| record.get(i))
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string());

    let notes = mapping.notes
        .and_then(|i| record.get(i))
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string());

    let tags = mapping.tags
        .and_then(|i| record.get(i))
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.split(',').map(|tag| tag.trim().to_string()).collect())
        .unwrap_or_default();

    Ok(PasswordEntry::new_with_fields(
        if title.is_empty() { username.clone() } else { title },
        username,
        password,
        url,
        notes,
        tags,
    ))
}

/// Export passwords to CSV format
pub fn export_csv(entries: &[PasswordEntry], options: &ExportOptions) -> Result<String> {
    let mut output = Vec::new();
    {
        let mut writer = Writer::from_writer(&mut output);

        // Write header
        let mut headers = vec!["Title", "Username"];
        
        if options.include_passwords {
            headers.push("Password");
        }
        
        headers.extend_from_slice(&["URL", "Notes"]);
        
        if options.include_tags {
            headers.push("Tags");
        }
        
        if options.include_metadata {
            headers.extend_from_slice(&["Created", "Modified"]);
        }

        writer.write_record(&headers)
            .map_err(|e| TwoPasswordError::ImportExportError(format!("Failed to write CSV header: {}", e)))?;

        // Write entries
        for entry in entries {
            let mut record = vec![
                entry.title.clone(),
                entry.username.clone(),
            ];

            if options.include_passwords {
                record.push(entry.password.clone());
            }

            record.push(entry.url.as_deref().unwrap_or("").to_string());
            record.push(entry.notes.as_deref().unwrap_or("").to_string());

            if options.include_tags {
                record.push(entry.tags.join(", "));
            }

            if options.include_metadata {
                record.push(entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string());
                record.push(entry.updated_at.format("%Y-%m-%d %H:%M:%S").to_string());
            }

            writer.write_record(&record)
                .map_err(|e| TwoPasswordError::ImportExportError(format!("Failed to write CSV record: {}", e)))?;
        }

        writer.flush()
            .map_err(|e| TwoPasswordError::ImportExportError(format!("Failed to flush CSV writer: {}", e)))?;
    }

    String::from_utf8(output)
        .map_err(|e| TwoPasswordError::ImportExportError(format!("Failed to convert CSV to string: {}", e)))
}