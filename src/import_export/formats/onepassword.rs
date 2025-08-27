//! 1Password format import support

use crate::storage::PasswordEntry;
use crate::{Result, TwoPasswordError};

use super::super::ImportResult;

/// Import 1Password format (basic implementation)
pub fn import_onepassword(content: &str, result: &mut ImportResult) -> Result<Vec<PasswordEntry>> {
    // 1Password exports are complex and can be in various formats
    // This is a basic implementation for the most common export format
    
    result.add_warning("1Password import is currently in basic mode".to_string());
    result.add_warning("For best results, export from 1Password to CSV format".to_string());
    
    let entries = if content.contains("***") {
        // 1PIF format (old format)
        import_1pif_format(content, result)?
    } else {
        // Try to parse as CSV
        result.add_error("Unsupported 1Password format. Please export as CSV.".to_string());
        Vec::new()
    };

    Ok(entries)
}

/// Import 1PIF format (simplified)
fn import_1pif_format(content: &str, result: &mut ImportResult) -> Result<Vec<PasswordEntry>> {
    let mut entries = Vec::new();
    
    // 1PIF format uses *** as separators and has complex structure
    // This is a very basic parser - in production, we'd need a full parser
    
    let sections: Vec<&str> = content.split("***").collect();
    
    for (index, section) in sections.iter().enumerate() {
        if section.trim().is_empty() {
            continue;
        }

        // Try to extract basic login information
        if let Some(entry) = parse_1pif_section(section) {
            entries.push(entry);
        } else {
            result.add_warning(format!("Could not parse section {}", index + 1));
        }
    }

    if entries.is_empty() {
        result.add_error("No valid entries found in 1PIF format. Consider exporting as CSV.".to_string());
    } else {
        result.add_warning(format!("Successfully processed {} 1Password entries (basic parsing)", entries.len()));
    }

    Ok(entries)
}

/// Parse a single 1PIF section (very basic)
fn parse_1pif_section(section: &str) -> Option<PasswordEntry> {
    // This is a highly simplified parser
    // Real 1PIF parsing would need to handle JSON within the format
    
    let lines: Vec<&str> = section.lines().collect();
    let mut title = String::new();
    let mut username = String::new();
    let mut password = String::new();
    let mut url = String::new();
    
    for line in lines {
        let line = line.trim();
        if line.contains("\"title\"") {
            if let Some(value) = extract_json_string_value(line, "title") {
                title = value;
            }
        } else if line.contains("\"username\"") {
            if let Some(value) = extract_json_string_value(line, "username") {
                username = value;
            }
        } else if line.contains("\"password\"") {
            if let Some(value) = extract_json_string_value(line, "password") {
                password = value;
            }
        } else if line.contains("\"location\"") {
            if let Some(value) = extract_json_string_value(line, "location") {
                url = value;
            }
        }
    }

    if !username.is_empty() && !password.is_empty() {
        let entry_title = if title.is_empty() {
            if !url.is_empty() {
                extract_domain_from_url(&url)
            } else {
                username.clone()
            }
        } else {
            title
        };

        let tags = vec!["1Password".to_string()];

        Some(PasswordEntry::new_with_fields(
            entry_title,
            username,
            password,
            if url.is_empty() { None } else { Some(url) },
            None,
            tags,
        ))
    } else {
        None
    }
}

/// Extract string value from JSON-like line (very basic)
fn extract_json_string_value(line: &str, key: &str) -> Option<String> {
    let key_pattern = format!("\"{}\"", key);
    if let Some(start) = line.find(&key_pattern) {
        let after_key = &line[start + key_pattern.len()..];
        if let Some(colon_pos) = after_key.find(':') {
            let after_colon = &after_key[colon_pos + 1..].trim();
            if after_colon.starts_with('"') {
                if let Some(end_quote) = after_colon[1..].find('"') {
                    return Some(after_colon[1..end_quote + 1].to_string());
                }
            }
        }
    }
    None
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