//! AES-GCM encryption and decryption

use crate::crypto::{secure_random, EncryptedData};
use crate::{Result, TwoPasswordError};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use ring::hmac;

/// Encrypt data using AES-256-GCM
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<EncryptedData> {
    // Create cipher
    let cipher_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(cipher_key);

    // Generate random nonce
    let nonce_bytes = secure_random::generate_nonce()?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| TwoPasswordError::crypto(format!("Encryption failed: {}", e)))?;

    // Calculate HMAC for integrity
    let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, key);
    let mut hmac_input = Vec::new();
    hmac_input.extend_from_slice(&nonce_bytes);
    hmac_input.extend_from_slice(&ciphertext);

    let hmac_tag = hmac::sign(&hmac_key, &hmac_input);
    let hmac_bytes = hmac_tag.as_ref().to_vec();

    Ok(EncryptedData {
        ciphertext,
        nonce: nonce_bytes,
        hmac: hmac_bytes,
    })
}

/// Decrypt data using AES-256-GCM
pub fn decrypt(key: &[u8; 32], encrypted: &EncryptedData) -> Result<Vec<u8>> {
    // Verify HMAC first
    let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, key);
    let mut hmac_input = Vec::new();
    hmac_input.extend_from_slice(&encrypted.nonce);
    hmac_input.extend_from_slice(&encrypted.ciphertext);

    hmac::verify(&hmac_key, &hmac_input, &encrypted.hmac).map_err(|_| {
        TwoPasswordError::crypto("HMAC verification failed - data may be corrupted")
    })?;

    // Create cipher
    let cipher_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(cipher_key);

    // Decrypt
    if encrypted.nonce.len() != crate::config::NONCE_SIZE {
        return Err(TwoPasswordError::crypto("Invalid nonce size"));
    }

    let nonce = Nonce::from_slice(&encrypted.nonce);
    let plaintext = cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| TwoPasswordError::crypto(format!("Decryption failed: {}", e)))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32]; // Test key
        let plaintext = b"Hello, secure world!";

        // Encrypt
        let encrypted = encrypt(&key, plaintext).unwrap();
        assert!(!encrypted.ciphertext.is_empty());
        assert_eq!(encrypted.nonce.len(), crate::config::NONCE_SIZE);
        assert_eq!(encrypted.hmac.len(), crate::config::HMAC_SIZE);

        // Decrypt
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_different_keys_produce_different_results() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let plaintext = b"test data";

        let encrypted1 = encrypt(&key1, plaintext).unwrap();
        let encrypted2 = encrypt(&key2, plaintext).unwrap();

        // Should produce different ciphertexts
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);

        // Key1 should not decrypt data encrypted with key2
        assert!(decrypt(&key1, &encrypted2).is_err());
    }

    #[test]
    fn test_tampered_data_fails_verification() {
        let key = [42u8; 32];
        let plaintext = b"test data";

        let mut encrypted = encrypt(&key, plaintext).unwrap();

        // Tamper with ciphertext
        encrypted.ciphertext[0] ^= 1;

        // Should fail HMAC verification
        assert!(decrypt(&key, &encrypted).is_err());
    }
}
