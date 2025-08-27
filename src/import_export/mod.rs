//! Import/Export functionality for TwoPassword
//!
//! Supports multiple password manager formats including CSV, 1Password, LastPass, Bitwarden

use crate::storage::PasswordEntry;
use crate::{Result, TwoPasswordError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod csv;
pub mod formats;
pub mod parser;

/// Supported import/export formats
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportFormat {
    CSV,
    OnePassword,
    LastPass,
    Bitwarden,
    Chrome,
    Firefox,
}

impl ImportFormat {
    /// Get format from file extension or content
    pub fn detect_from_filename(filename: &str) -> Option<Self> {
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())?
            .to_lowercase();

        match ext.as_str() {
            "csv" => Some(ImportFormat::CSV),
            "1pif" => Some(ImportFormat::OnePassword),
            "json" => Some(ImportFormat::Bitwarden), // Bitwarden exports as JSON
            _ => None,
        }
    }

    /// Get human-readable format name
    pub fn display_name(&self) -> &'static str {
        match self {
            ImportFormat::CSV => "CSV",
            ImportFormat::OnePassword => "1Password",
            ImportFormat::LastPass => "LastPass",
            ImportFormat::Bitwarden => "Bitwarden",
            ImportFormat::Chrome => "Chrome",
            ImportFormat::Firefox => "Firefox",
        }
    }
}

/// Import/export result with statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success: bool,
    pub total_entries: usize,
    pub imported_entries: usize,
    pub skipped_entries: usize,
    pub error_entries: usize,
    pub duplicate_entries: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ImportResult {
    pub fn new() -> Self {
        Self {
            success: false,
            total_entries: 0,
            imported_entries: 0,
            skipped_entries: 0,
            error_entries: 0,
            duplicate_entries: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.error_entries += 1;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn finalize(&mut self) {
        self.success = self.error_entries == 0 || self.imported_entries > 0;
        self.skipped_entries = self.total_entries - self.imported_entries - self.error_entries;
    }
}

/// Export options for customizing output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: ImportFormat,
    pub include_passwords: bool,
    pub include_notes: bool,
    pub include_tags: bool,
    pub include_metadata: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ImportFormat::CSV,
            include_passwords: true,
            include_notes: true,
            include_tags: true,
            include_metadata: false,
        }
    }
}

/// Main import/export service
pub struct ImportExportService {
    duplicate_check: bool,
}

impl ImportExportService {
    pub fn new() -> Self {
        Self {
            duplicate_check: true,
        }
    }

    /// Set whether to check for duplicates during import
    pub fn set_duplicate_check(&mut self, enabled: bool) {
        self.duplicate_check = enabled;
    }

    /// Auto-detect format from file content
    pub fn detect_format(&self, filename: &str, content: &str) -> Result<ImportFormat> {
        // First try filename detection
        if let Some(format) = ImportFormat::detect_from_filename(filename) {
            // Validate format matches content
            if self.validate_format_content(&format, content) {
                return Ok(format);
            }
        }

        // Try content-based detection
        self.detect_format_from_content(content)
    }

    /// Validate that content matches expected format
    fn validate_format_content(&self, format: &ImportFormat, content: &str) -> bool {
        match format {
            ImportFormat::CSV => self.is_valid_csv(content),
            ImportFormat::Bitwarden => self.is_valid_json(content),
            ImportFormat::OnePassword => content.contains("***") || content.contains("1Password"),
            ImportFormat::LastPass => content.contains("url,username,password") || content.contains("LastPass"),
            ImportFormat::Chrome => content.contains("name,url,username,password"),
            ImportFormat::Firefox => self.is_valid_json(content) && content.contains("logins"),
        }
    }

    /// Detect format from content analysis
    fn detect_format_from_content(&self, content: &str) -> Result<ImportFormat> {
        let content_lower = content.to_lowercase();

        // JSON-based formats
        if self.is_valid_json(content) {
            if content_lower.contains("bitwarden") || content_lower.contains("encrypted") {
                return Ok(ImportFormat::Bitwarden);
            }
            if content_lower.contains("logins") && content_lower.contains("hostname") {
                return Ok(ImportFormat::Firefox);
            }
        }

        // CSV-based formats - check headers
        if let Some(first_line) = content.lines().next() {
            let headers = first_line.to_lowercase();
            
            if headers.contains("url,username,password") || headers.contains("website,username,password") {
                if headers.contains("lastpass") {
                    return Ok(ImportFormat::LastPass);
                }
                return Ok(ImportFormat::CSV);
            }
            
            if headers.contains("name,url,username,password") {
                return Ok(ImportFormat::Chrome);
            }

            // Generic CSV detection
            if headers.contains(',') && (headers.contains("password") || headers.contains("login")) {
                return Ok(ImportFormat::CSV);
            }
        }

        // 1Password format
        if content.contains("***") && (content.contains("1Password") || content.contains("webforms.WebForm")) {
            return Ok(ImportFormat::OnePassword);
        }

        Err(TwoPasswordError::ImportExportError(
            "Unable to detect file format. Please specify format manually.".to_string()
        ))
    }

    /// Check if content is valid JSON
    fn is_valid_json(&self, content: &str) -> bool {
        serde_json::from_str::<serde_json::Value>(content).is_ok()
    }

    /// Check if content is valid CSV
    fn is_valid_csv(&self, content: &str) -> bool {
        ::csv::Reader::from_reader(content.as_bytes()).headers().is_ok()
    }

    /// Import passwords from file content
    pub fn import_passwords(
        &self,
        content: &str,
        format: ImportFormat,
        existing_entries: &[PasswordEntry],
    ) -> Result<(Vec<PasswordEntry>, ImportResult)> {
        let mut result = ImportResult::new();
        
        let entries = match format {
            ImportFormat::CSV => self.import_csv(content, &mut result)?,
            ImportFormat::LastPass => self.import_lastpass_csv(content, &mut result)?,
            ImportFormat::Chrome => self.import_chrome_csv(content, &mut result)?,
            ImportFormat::Bitwarden => self.import_bitwarden_json(content, &mut result)?,
            ImportFormat::Firefox => self.import_firefox_json(content, &mut result)?,
            ImportFormat::OnePassword => self.import_onepassword(content, &mut result)?,
        };

        result.total_entries = entries.len();
        
        // Filter duplicates if enabled
        let filtered_entries = if self.duplicate_check {
            self.filter_duplicates(entries, existing_entries, &mut result)
        } else {
            result.imported_entries = entries.len();
            entries
        };

        result.finalize();
        Ok((filtered_entries, result))
    }

    /// Export passwords to specified format
    pub fn export_passwords(
        &self,
        entries: &[PasswordEntry],
        options: &ExportOptions,
    ) -> Result<String> {
        match options.format {
            ImportFormat::CSV => self.export_csv(entries, options),
            ImportFormat::Bitwarden => self.export_bitwarden_json(entries, options),
            _ => Err(TwoPasswordError::ImportExportError(
                format!("Export format {} not yet implemented", options.format.display_name())
            ))
        }
    }

    /// Filter out duplicate entries
    fn filter_duplicates(
        &self,
        entries: Vec<PasswordEntry>,
        existing_entries: &[PasswordEntry],
        result: &mut ImportResult,
    ) -> Vec<PasswordEntry> {
        let existing_keys: std::collections::HashSet<_> = existing_entries
            .iter()
            .map(|e| (e.title.to_lowercase(), e.username.to_lowercase()))
            .collect();

        let mut unique_entries = Vec::new();
        let mut seen_keys = std::collections::HashSet::new();

        for entry in entries {
            let key = (entry.title.to_lowercase(), entry.username.to_lowercase());
            
            if existing_keys.contains(&key) || seen_keys.contains(&key) {
                result.duplicate_entries += 1;
                result.add_warning(format!("Duplicate entry skipped: {} ({})", entry.title, entry.username));
            } else {
                seen_keys.insert(key);
                unique_entries.push(entry);
            }
        }

        result.imported_entries = unique_entries.len();
        unique_entries
    }

    /// Import CSV format (generic)
    fn import_csv(&self, content: &str, result: &mut ImportResult) -> Result<Vec<PasswordEntry>> {
        csv::import_csv(content, result)
    }

    /// Import LastPass CSV format
    fn import_lastpass_csv(&self, content: &str, result: &mut ImportResult) -> Result<Vec<PasswordEntry>> {
        formats::lastpass::import_lastpass_csv(content, result)
    }

    /// Import Chrome CSV format
    fn import_chrome_csv(&self, content: &str, result: &mut ImportResult) -> Result<Vec<PasswordEntry>> {
        formats::chrome::import_chrome_csv(content, result)
    }

    /// Import Bitwarden JSON format
    fn import_bitwarden_json(&self, content: &str, result: &mut ImportResult) -> Result<Vec<PasswordEntry>> {
        formats::bitwarden::import_bitwarden_json(content, result)
    }

    /// Import Firefox JSON format
    fn import_firefox_json(&self, content: &str, result: &mut ImportResult) -> Result<Vec<PasswordEntry>> {
        formats::firefox::import_firefox_json(content, result)
    }

    /// Import 1Password format
    fn import_onepassword(&self, content: &str, result: &mut ImportResult) -> Result<Vec<PasswordEntry>> {
        formats::onepassword::import_onepassword(content, result)
    }

    /// Export to CSV format
    fn export_csv(&self, entries: &[PasswordEntry], options: &ExportOptions) -> Result<String> {
        csv::export_csv(entries, options)
    }

    /// Export to Bitwarden JSON format
    fn export_bitwarden_json(&self, entries: &[PasswordEntry], options: &ExportOptions) -> Result<String> {
        formats::bitwarden::export_bitwarden_json(entries, options)
    }
}

impl Default for ImportExportService {
    fn default() -> Self {
        Self::new()
    }
}