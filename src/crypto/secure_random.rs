//! Secure random number generation

use crate::{Result, TwoPasswordError};
use rand::RngCore;

/// Fill buffer with cryptographically secure random bytes
pub fn fill_random(buffer: &mut [u8]) -> Result<()> {
    let mut rng = rand::thread_rng();
    rng.try_fill_bytes(buffer)
        .map_err(|e| TwoPasswordError::crypto(format!("Random generation failed: {}", e)))?;
    Ok(())
}

/// Generate cryptographically secure random bytes
pub fn generate_bytes(len: usize) -> Result<Vec<u8>> {
    let mut buffer = vec![0u8; len];
    fill_random(&mut buffer)?;
    Ok(buffer)
}

/// Generate a nonce for AES-GCM encryption
pub fn generate_nonce() -> Result<Vec<u8>> {
    generate_bytes(crate::config::NONCE_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_random() {
        let mut buffer = [0u8; 32];
        assert!(fill_random(&mut buffer).is_ok());
        // Very unlikely to be all zeros if random
        assert_ne!(buffer, [0u8; 32]);
    }

    #[test]
    fn test_generate_bytes() {
        let bytes = generate_bytes(16).unwrap();
        assert_eq!(bytes.len(), 16);
        // Test multiple generations are different
        let bytes2 = generate_bytes(16).unwrap();
        assert_ne!(bytes, bytes2);
    }
}
