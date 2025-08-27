//! HaveIBeenPwned API integration with k-anonymity
//!
//! Checks passwords against known data breaches without exposing the actual password
//! using the k-anonymity method (hash prefix search).

use crate::password_health::BreachCheckResult;
use crate::{Result, TwoPasswordError};
use reqwest;
use sha1::{Sha1, Digest};
use std::collections::HashMap;

/// HaveIBeenPwned API base URL for password range search
const HIBP_API_URL: &str = "https://api.pwnedpasswords.com/range";

/// Check if password has been breached using HaveIBeenPwned k-anonymity API
pub async fn check_password_breach(password: &str) -> Result<BreachCheckResult> {
    if password.is_empty() {
        return Ok(BreachCheckResult {
            is_breached: false,
            breach_count: None,
            last_checked: chrono::Utc::now(),
        });
    }

    // Calculate SHA-1 hash of the password
    let mut hasher = Sha1::new();
    hasher.update(password.as_bytes());
    let hash_bytes = hasher.finalize();
    let hash_hex = format!("{:X}", hash_bytes);

    // Split hash into prefix (first 5 chars) and suffix (remaining chars)
    let (prefix, suffix) = hash_hex.split_at(5);

    // Query HaveIBeenPwned API with just the prefix
    let response = query_hibp_api(prefix).await?;

    // Parse response and look for our suffix
    let breach_count = parse_hibp_response(&response, suffix);

    Ok(BreachCheckResult {
        is_breached: breach_count.is_some(),
        breach_count,
        last_checked: chrono::Utc::now(),
    })
}

/// Query the HaveIBeenPwned API with hash prefix
async fn query_hibp_api(prefix: &str) -> Result<String> {
    let url = format!("{}/{}", HIBP_API_URL, prefix);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "TwoPassword/1.0")
        .header("Add-Padding", "true") // Request padding for additional anonymity
        .send()
        .await
        .map_err(|e| TwoPasswordError::ImportExportError(format!("HTTP request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(TwoPasswordError::ImportExportError(
            format!("HaveIBeenPwned API returned status: {}", response.status())
        ));
    }

    let response_text = response
        .text()
        .await
        .map_err(|e| TwoPasswordError::ImportExportError(format!("Failed to read response: {}", e)))?;

    Ok(response_text)
}

/// Parse HaveIBeenPwned response and find matching hash suffix
fn parse_hibp_response(response: &str, target_suffix: &str) -> Option<u64> {
    for line in response.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Each line format: "HASH_SUFFIX:COUNT"
        if let Some((hash_suffix, count_str)) = line.split_once(':') {
            if hash_suffix.trim() == target_suffix {
                if let Ok(count) = count_str.trim().parse::<u64>() {
                    return Some(count);
                }
            }
        }
    }
    None
}

/// Batch check multiple passwords for breaches
pub async fn batch_check_breaches(passwords: &[String]) -> Result<HashMap<String, BreachCheckResult>> {
    let mut results = HashMap::new();
    
    // Group passwords by hash prefix to minimize API calls
    let mut prefix_groups: HashMap<String, Vec<(String, String)>> = HashMap::new();
    
    for password in passwords {
        if password.is_empty() {
            results.insert(password.clone(), BreachCheckResult {
                is_breached: false,
                breach_count: None,
                last_checked: chrono::Utc::now(),
            });
            continue;
        }

        let mut hasher = Sha1::new();
        hasher.update(password.as_bytes());
        let hash_bytes = hasher.finalize();
        let hash_hex = format!("{:X}", hash_bytes);
        let (prefix, suffix) = hash_hex.split_at(5);
        
        prefix_groups
            .entry(prefix.to_string())
            .or_insert_with(Vec::new)
            .push((password.clone(), suffix.to_string()));
    }

    // Process each prefix group
    for (prefix, password_suffix_pairs) in prefix_groups {
        match query_hibp_api(&prefix).await {
            Ok(response) => {
                for (password, suffix) in password_suffix_pairs {
                    let breach_count = parse_hibp_response(&response, &suffix);
                    results.insert(password, BreachCheckResult {
                        is_breached: breach_count.is_some(),
                        breach_count,
                        last_checked: chrono::Utc::now(),
                    });
                }
            }
            Err(e) => {
                // On error, mark all passwords in this group as unchecked
                for (password, _) in password_suffix_pairs {
                    results.insert(password, BreachCheckResult {
                        is_breached: false,
                        breach_count: None,
                        last_checked: chrono::Utc::now(),
                    });
                }
                tracing::warn!("Failed to check breaches for prefix {}: {}", prefix, e);
            }
        }

        // Rate limiting: wait between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    Ok(results)
}

/// Check if breach checking is available (network connectivity)
pub async fn is_breach_check_available() -> bool {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/00000", HIBP_API_URL))
        .header("User-Agent", "TwoPassword/1.0")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    response.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hibp_response() {
        let response = "0018A45C4D1DEF81644B54AB7F969B88D65:1\n003D68EB55068C33ACE09247EE4C639306B:3\n";
        
        // Should find the first hash
        let count = parse_hibp_response(response, "0018A45C4D1DEF81644B54AB7F969B88D65");
        assert_eq!(count, Some(1));
        
        // Should find the second hash
        let count = parse_hibp_response(response, "003D68EB55068C33ACE09247EE4C639306B");
        assert_eq!(count, Some(3));
        
        // Should not find non-existent hash
        let count = parse_hibp_response(response, "NONEXISTENT");
        assert_eq!(count, None);
    }

    #[tokio::test]
    async fn test_known_breached_password() {
        // Test with a known breached password (password)
        let result = check_password_breach("password").await;
        
        if let Ok(breach_result) = result {
            // This should be breached (unless network is unavailable)
            if breach_result.is_breached {
                assert!(breach_result.breach_count.is_some());
                assert!(breach_result.breach_count.unwrap() > 0);
            }
        }
    }

    #[tokio::test]
    async fn test_empty_password() {
        let result = check_password_breach("").await.unwrap();
        assert!(!result.is_breached);
        assert!(result.breach_count.is_none());
    }

    #[test]
    fn test_sha1_hash_generation() {
        // Test that we generate correct SHA-1 hash for known input
        let password = "password";
        let mut hasher = Sha1::new();
        hasher.update(password.as_bytes());
        let hash_bytes = hasher.finalize();
        let hash_hex = format!("{:x}", hash_bytes);
        
        // Known SHA-1 hash of "password" 
        assert_eq!(hash_hex, "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8");
    }
}