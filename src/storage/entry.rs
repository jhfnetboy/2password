//! Password entry operations and utilities

use crate::storage::PasswordEntry;
use crate::{Result, TwoPasswordError};
use uuid::Uuid;

/// Search and filter operations for password entries
pub struct EntryManager;

impl EntryManager {
    /// Find entries by fuzzy title matching
    pub fn fuzzy_search<'a>(entries: &'a [PasswordEntry], query: &str) -> Vec<&'a PasswordEntry> {
        let query_lower = query.to_lowercase();
        let mut matches: Vec<(&PasswordEntry, i32)> = entries
            .iter()
            .filter_map(|entry| {
                let title_lower = entry.title.to_lowercase();

                // Exact match gets highest score
                if title_lower == query_lower {
                    return Some((entry, 100));
                }

                // Starts with query gets high score
                if title_lower.starts_with(&query_lower) {
                    return Some((entry, 80));
                }

                // Contains query gets medium score
                if title_lower.contains(&query_lower) {
                    return Some((entry, 60));
                }

                // URL matching
                if let Some(ref url) = entry.url {
                    let url_lower = url.to_lowercase();
                    if url_lower.contains(&query_lower) {
                        return Some((entry, 40));
                    }
                }

                // Username matching
                if entry.username.to_lowercase().contains(&query_lower) {
                    return Some((entry, 30));
                }

                None
            })
            .collect();

        // Sort by score (highest first)
        matches.sort_by(|a, b| b.1.cmp(&a.1));

        matches.into_iter().map(|(entry, _)| entry).collect()
    }

    /// Find entries by URL domain
    pub fn find_by_domain<'a>(
        entries: &'a [PasswordEntry],
        domain: &str,
    ) -> Vec<&'a PasswordEntry> {
        let domain_lower = domain.to_lowercase();
        entries
            .iter()
            .filter(|entry| {
                if let Some(ref url) = entry.url {
                    let url_lower = url.to_lowercase();
                    url_lower.contains(&domain_lower)
                } else {
                    false
                }
            })
            .collect()
    }

    /// Find entries by tag
    pub fn find_by_tag<'a>(entries: &'a [PasswordEntry], tag: &str) -> Vec<&'a PasswordEntry> {
        let tag_lower = tag.to_lowercase();
        entries
            .iter()
            .filter(|entry| entry.tags.iter().any(|t| t.to_lowercase() == tag_lower))
            .collect()
    }

    /// Find entry by ID
    pub fn find_by_id<'a>(entries: &'a [PasswordEntry], id: &Uuid) -> Option<&'a PasswordEntry> {
        entries.iter().find(|entry| entry.id == *id)
    }

    /// Validate entry data
    pub fn validate_entry(entry: &PasswordEntry) -> Result<()> {
        if entry.title.trim().is_empty() {
            return Err(TwoPasswordError::validation("Title cannot be empty"));
        }

        if entry.username.trim().is_empty() {
            return Err(TwoPasswordError::validation("Username cannot be empty"));
        }

        if entry.password.trim().is_empty() {
            return Err(TwoPasswordError::validation("Password cannot be empty"));
        }

        // Validate URL if provided
        if let Some(ref url) = entry.url {
            if !url.trim().is_empty() && !is_valid_url(url) {
                return Err(TwoPasswordError::validation("Invalid URL format"));
            }
        }

        Ok(())
    }

    /// Check for duplicate entries
    pub fn find_duplicates(entries: &[PasswordEntry]) -> Vec<Vec<&PasswordEntry>> {
        use std::collections::HashMap;

        let mut groups: HashMap<String, Vec<&PasswordEntry>> = HashMap::new();

        for entry in entries {
            // Create key from title and username (case insensitive)
            let key = format!(
                "{}::{}",
                entry.title.to_lowercase().trim(),
                entry.username.to_lowercase().trim()
            );

            groups.entry(key).or_default().push(entry);
        }

        groups
            .into_values()
            .filter(|group| group.len() > 1)
            .collect()
    }

    /// Get entries created in date range
    pub fn entries_in_date_range(
        entries: &[PasswordEntry],
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Vec<&PasswordEntry> {
        entries
            .iter()
            .filter(|entry| entry.created_at >= start && entry.created_at <= end)
            .collect()
    }

    /// Get entries modified in date range
    pub fn entries_modified_in_range(
        entries: &[PasswordEntry],
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Vec<&PasswordEntry> {
        entries
            .iter()
            .filter(|entry| entry.updated_at >= start && entry.updated_at <= end)
            .collect()
    }
}

/// Check if a string is a valid URL
fn is_valid_url(url_str: &str) -> bool {
    url::Url::parse(url_str).is_ok()
}

/// Generate secure random password
pub fn generate_password(
    length: usize,
    include_uppercase: bool,
    include_lowercase: bool,
    include_numbers: bool,
    include_symbols: bool,
) -> Result<String> {
    use rand::Rng;

    if length == 0 {
        return Err(TwoPasswordError::validation(
            "Password length must be greater than 0",
        ));
    }

    let mut charset = String::new();

    if include_lowercase {
        charset.push_str("abcdefghijklmnopqrstuvwxyz");
    }

    if include_uppercase {
        charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }

    if include_numbers {
        charset.push_str("0123456789");
    }

    if include_symbols {
        charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
    }

    if charset.is_empty() {
        return Err(TwoPasswordError::validation(
            "At least one character set must be included",
        ));
    }

    let charset_chars: Vec<char> = charset.chars().collect();
    let mut rng = rand::thread_rng();

    let password: String = (0..length)
        .map(|_| charset_chars[rng.gen_range(0..charset_chars.len())])
        .collect();

    Ok(password)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(title: &str, username: &str, url: Option<String>) -> PasswordEntry {
        let mut entry = PasswordEntry::new(
            title.to_string(),
            username.to_string(),
            "password".to_string(),
        );
        entry.url = url;
        entry
    }

    #[test]
    fn test_fuzzy_search() {
        let entries = vec![
            create_test_entry("GitHub", "user1", Some("https://github.com".to_string())),
            create_test_entry("Gmail", "user2", Some("https://gmail.com".to_string())),
            create_test_entry("GitLab", "user3", Some("https://gitlab.com".to_string())),
        ];

        let results = EntryManager::fuzzy_search(&entries, "git");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "GitHub"); // Should come first (starts with)
        assert_eq!(results[1].title, "GitLab"); // Should come second
    }

    #[test]
    fn test_find_by_domain() {
        let entries = vec![
            create_test_entry("GitHub", "user1", Some("https://github.com".to_string())),
            create_test_entry(
                "Google Drive",
                "user2",
                Some("https://drive.google.com".to_string()),
            ),
            create_test_entry("Gmail", "user3", Some("https://gmail.com".to_string())),
        ];

        let results = EntryManager::find_by_domain(&entries, "google");
        assert_eq!(results.len(), 1); // Only "drive.google.com" contains "google"
        
        let github_results = EntryManager::find_by_domain(&entries, "github");
        assert_eq!(github_results.len(), 1);
    }

    #[test]
    fn test_validate_entry() {
        let valid_entry =
            create_test_entry("Test", "user", Some("https://example.com".to_string()));
        assert!(EntryManager::validate_entry(&valid_entry).is_ok());

        let invalid_entry = create_test_entry("", "user", None);
        assert!(EntryManager::validate_entry(&invalid_entry).is_err());
    }

    #[test]
    fn test_generate_password() {
        let password = generate_password(16, true, true, true, true).unwrap();
        assert_eq!(password.len(), 16);

        let short_password = generate_password(8, true, false, false, false).unwrap();
        assert_eq!(short_password.len(), 8);
        // Should only contain uppercase letters
        assert!(short_password.chars().all(|c| c.is_ascii_uppercase()));
    }

    #[test]
    fn test_find_duplicates() {
        let entries = vec![
            create_test_entry("GitHub", "user1", None),
            create_test_entry("github", "user1", None), // Duplicate (case insensitive)
            create_test_entry("GitLab", "user2", None),
        ];

        let duplicates = EntryManager::find_duplicates(&entries);
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].len(), 2);
    }
}
