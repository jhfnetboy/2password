//! Password authentication module

use crate::crypto::key_derivation;
use crate::{Result, TwoPasswordError};

/// Hash a password for secure storage
pub fn hash_password(password: &str) -> Result<String> {
    key_derivation::hash_password_for_storage(password)
}

/// Verify a password against a stored hash
pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool> {
    key_derivation::verify_password(password, stored_hash)
}

/// Validate password strength
pub fn validate_password_strength(password: &str) -> Result<()> {
    if password.len() < 8 {
        return Err(TwoPasswordError::validation(
            "Password must be at least 8 characters long",
        ));
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password
        .chars()
        .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

    let mut score = 0;
    if has_uppercase {
        score += 1;
    }
    if has_lowercase {
        score += 1;
    }
    if has_digit {
        score += 1;
    }
    if has_special {
        score += 1;
    }

    if score < 3 {
        return Err(TwoPasswordError::validation(
            "Password must contain at least 3 of: uppercase, lowercase, digit, special character",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_verify_password() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn test_password_strength_validation() {
        assert!(validate_password_strength("SecurePass123!").is_ok());
        assert!(validate_password_strength("short").is_err());
        assert!(validate_password_strength("alllowercase123").is_err()); // only lowercase + digits = 2 categories
        assert!(validate_password_strength("ALLUPPERCASE123").is_err()); // only uppercase + digits = 2 categories
        assert!(validate_password_strength("NoNumbers!").is_ok()); // uppercase + lowercase + special = 3 categories (OK)
        assert!(validate_password_strength("onlylowercase").is_err()); // only lowercase = 1 category
    }
}
