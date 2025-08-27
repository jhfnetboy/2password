//! Generic parsing utilities for import/export

use crate::{Result, TwoPasswordError};

/// Parse progress callback type
pub type ProgressCallback = Box<dyn Fn(usize, usize) + Send>;

/// File size limits for imports
pub const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50MB
pub const MAX_ENTRIES: usize = 100_000; // 100k entries

/// Validate file size and basic content
pub fn validate_import_file(content: &str, filename: &str) -> Result<()> {
    // Check file size
    if content.len() > MAX_FILE_SIZE {
        return Err(TwoPasswordError::ImportExportError(
            format!("File too large: {} bytes (max: {} bytes)", content.len(), MAX_FILE_SIZE)
        ));
    }

    // Check if file appears to be binary
    if content.chars().any(|c| c as u32 > 127 && !c.is_whitespace()) {
        return Err(TwoPasswordError::ImportExportError(
            "File appears to be binary. Please use text-based export formats.".to_string()
        ));
    }

    // Basic empty file check
    if content.trim().is_empty() {
        return Err(TwoPasswordError::ImportExportError(
            "File is empty or contains only whitespace".to_string()
        ));
    }

    Ok(())
}

/// Sanitize and validate field values
pub fn sanitize_field(value: &str, max_length: usize) -> String {
    let sanitized = value
        .trim()
        .chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .take(max_length)
        .collect();

    sanitized
}

/// Extract domain from URL with better error handling
pub fn extract_domain(url: &str) -> String {
    if url.is_empty() {
        return "Unknown".to_string();
    }

    // Try to parse as URL
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(host) = parsed.host_str() {
            let clean_host = host.trim_start_matches("www.");
            if !clean_host.is_empty() {
                return clean_host.to_string();
            }
        }
    }

    // Try to parse URL without protocol
    let url_with_protocol = if !url.starts_with("http") {
        format!("https://{}", url)
    } else {
        url.to_string()
    };

    if let Ok(parsed) = url::Url::parse(&url_with_protocol) {
        if let Some(host) = parsed.host_str() {
            let clean_host = host.trim_start_matches("www.");
            if !clean_host.is_empty() {
                return clean_host.to_string();
            }
        }
    }

    // Fallback: simple text processing
    let clean_url = url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.");
    
    if let Some(slash_pos) = clean_url.find('/') {
        clean_url[..slash_pos].to_string()
    } else {
        clean_url.to_string()
    }
}

/// Validate password strength and warn about weak passwords
pub fn analyze_password_strength(password: &str) -> (u8, Vec<String>) {
    let mut score = 0u8;
    let mut warnings = Vec::new();

    // Length check
    if password.len() >= 12 {
        score += 25;
    } else if password.len() >= 8 {
        score += 15;
        warnings.push("Password shorter than 12 characters".to_string());
    } else {
        warnings.push("Password too short (less than 8 characters)".to_string());
    }

    // Character variety
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    let char_types = [has_lower, has_upper, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();

    match char_types {
        4 => score += 25,
        3 => {
            score += 20;
            warnings.push("Missing one character type (uppercase, lowercase, digits, symbols)".to_string());
        }
        2 => {
            score += 10;
            warnings.push("Password uses only 2 character types".to_string());
        }
        1 => {
            warnings.push("Password uses only 1 character type".to_string());
        }
        0 => {
            warnings.push("Invalid password".to_string());
        }
        _ => {
            // This case should never occur since we filter boolean array with max 4 elements
            warnings.push("Unexpected character type analysis".to_string());
        }
    }

    // Common patterns
    if password.to_lowercase().contains("password") {
        warnings.push("Password contains the word 'password'".to_string());
        score = score.saturating_sub(20);
    }

    if is_sequential_or_repeated(password) {
        warnings.push("Password contains sequential or repeated characters".to_string());
        score = score.saturating_sub(15);
    }

    // Entropy estimate
    let unique_chars = password.chars().collect::<std::collections::HashSet<_>>().len();
    if unique_chars < password.len() / 2 {
        warnings.push("Password has low character diversity".to_string());
        score = score.saturating_sub(10);
    } else {
        score += 25;
    }

    // Cap at 100
    score = score.min(100);

    (score, warnings)
}

/// Check for sequential or repeated character patterns
fn is_sequential_or_repeated(password: &str) -> bool {
    let chars: Vec<char> = password.chars().collect();
    
    if chars.len() < 3 {
        return false;
    }

    for window in chars.windows(3) {
        // Check for sequential (abc, 123)
        let a = window[0] as u32;
        let b = window[1] as u32;
        let c = window[2] as u32;
        
        if (b == a + 1 && c == b + 1) || (b == a - 1 && c == b - 1) {
            return true;
        }

        // Check for repeated (aaa, 111)
        if window[0] == window[1] && window[1] == window[2] {
            return true;
        }
    }

    false
}

/// Format import/export errors with context
pub fn format_parse_error(line: usize, field: &str, value: &str, error: &str) -> String {
    let display_value = if value.len() > 50 {
        format!("{}...", &value[..47])
    } else {
        value.to_string()
    };
    
    format!(
        "Line {}: Error in field '{}' with value '{}': {}",
        line,
        field,
        display_value,
        error
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://www.example.com/path"), "example.com");
        assert_eq!(extract_domain("http://example.com"), "example.com");
        assert_eq!(extract_domain("example.com"), "example.com");
        assert_eq!(extract_domain("www.example.com"), "example.com");
        assert_eq!(extract_domain(""), "Unknown");
    }

    #[test]
    fn test_password_strength() {
        let (score, warnings) = analyze_password_strength("Password123!");
        assert!(score >= 30); // Contains "password" so gets penalized
        assert!(warnings.iter().any(|w| w.contains("password")));

        let (score, _) = analyze_password_strength("Tr0ub4dor&3");
        // This is a decent password but might get some penalties
        assert!(score >= 50);

        let (score, warnings) = analyze_password_strength("123456");
        assert!(score < 30);
        assert!(!warnings.is_empty());
        
        // Test a very strong password - 20+ chars, all types, high entropy, no common words
        let (score, warnings) = analyze_password_strength("MyV3ryStr0ng!P@ssW0rd2024#");
        assert!(score >= 70); // Should be quite high with length 25+ and good characteristics
        assert!(!warnings.iter().any(|w| w.contains("too short")));
    }

    #[test]
    fn test_sequential_patterns() {
        assert!(is_sequential_or_repeated("abc"));
        assert!(is_sequential_or_repeated("123"));
        assert!(is_sequential_or_repeated("aaa"));
        assert!(!is_sequential_or_repeated("random"));
    }
}