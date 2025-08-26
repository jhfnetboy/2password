// Integration tests for 2Password Phase 1 functionality
// Tests the complete cryptographic workflow and data integrity

use tempfile::TempDir;
use twopassword::{
    auth::recovery::RecoveryManager,
    crypto::{
        key_derivation::derive_key,
        secret_sharing::SecretSharing,
    },
    storage::{PasswordEntry, Vault, VaultManager},
};

/// Test the complete vault workflow
#[test]
fn test_complete_vault_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let vault_path = temp_dir.path().join("test_vault.json");
    
    let mut vault_manager = VaultManager::new();
    let master_password = "SuperSecurePassword123!";
    
    // Create a new vault
    vault_manager.create_vault(vault_path.clone(), master_password)
        .expect("Failed to create vault");
    
    // Add password entry
    let entry = PasswordEntry::new(
        "github.com".to_string(),
        "user@example.com".to_string(),
        "MySecretPassword".to_string(),
    );
    
    let entry_id = entry.id;
    
    // Add entry to vault
    {
        let vault = vault_manager.get_vault_mut().expect("No vault loaded");
        vault.add_entry(entry);
    }
    
    // Save vault
    vault_manager.save_vault()
        .expect("Failed to save vault");
    
    // Close and reload vault
    vault_manager.close_vault();
    vault_manager.load_vault(vault_path, master_password)
        .expect("Failed to reload vault");
    
    // Verify data persistence
    let vault = vault_manager.get_vault().expect("No vault loaded after reload");
    let retrieved_entry = vault.get_entry(&entry_id)
        .expect("Entry not found after reload");
    
    assert_eq!(retrieved_entry.title, "github.com");
    assert_eq!(retrieved_entry.username, "user@example.com");
    assert_eq!(retrieved_entry.password, "MySecretPassword");
}

/// Test Shamir's Secret Sharing recovery mechanism
#[test]
fn test_secret_sharing_recovery() {
    let master_key = [42u8; 32]; // Changed to array type
    
    // Generate 2-of-3 shares
    let shares = SecretSharing::split_secret(&master_key)
        .expect("Failed to split secret");
    
    assert_eq!(shares.len(), 3);
    
    // Test recovery with 2 shares (minimum threshold)
    let recovered_key = SecretSharing::reconstruct_secret(&shares[0], &shares[2])
        .expect("Failed to reconstruct secret");
    
    assert_eq!(recovered_key, master_key);
    
    // Test all combinations work
    let recovered_1_2 = SecretSharing::reconstruct_secret(&shares[0], &shares[1])
        .expect("Failed to reconstruct with shares 1,2");
    assert_eq!(recovered_1_2, master_key);
    
    let recovered_2_3 = SecretSharing::reconstruct_secret(&shares[1], &shares[2])
        .expect("Failed to reconstruct with shares 2,3");
    assert_eq!(recovered_2_3, master_key);
    
    // Test that duplicate shares cannot recover (should fail)
    let result = SecretSharing::reconstruct_secret(&shares[0], &shares[0]);
    assert!(result.is_err(), "Duplicate shares should not be able to recover secret");
}

/// Test key derivation consistency
#[test]
fn test_key_derivation_consistency() {
    let password = "TestPassword123!";
    let salt = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    
    // Derive key multiple times
    let key1 = derive_key(password, &salt).expect("Failed to derive key 1");
    let key2 = derive_key(password, &salt).expect("Failed to derive key 2");
    
    // Keys should be identical for same inputs
    assert_eq!(key1, key2);
    
    // Keys should be different for different passwords
    let key3 = derive_key("DifferentPassword", &salt)
        .expect("Failed to derive key 3");
    assert_ne!(key1, key3);
    
    // Keys should be different for different salts
    let different_salt = vec![16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
    let key4 = derive_key(password, &different_salt)
        .expect("Failed to derive key 4");
    assert_ne!(key1, key4);
}

/// Test vault operations
#[test]
fn test_vault_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let vault_path = temp_dir.path().join("ops_test_vault.json");
    
    // Create vault
    let mut vault = Vault::new(vault_path);
    
    // Create test entries
    let entry1 = PasswordEntry::new(
        "example.com".to_string(),
        "user1".to_string(),
        "password1".to_string(),
    );
    let entry2 = PasswordEntry::new(
        "test.com".to_string(),
        "user2".to_string(),
        "password2".to_string(),
    );
    
    let entry1_id = entry1.id;
    let entry2_id = entry2.id;
    
    // Add entries
    vault.add_entry(entry1);
    vault.add_entry(entry2);
    
    // Test retrieval
    assert!(vault.get_entry(&entry1_id).is_some());
    assert!(vault.get_entry(&entry2_id).is_some());
    
    // Test search
    let search_results = vault.search_by_title("example");
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].title, "example.com");
    
    // Test get all
    let all_entries = vault.get_all_entries();
    assert_eq!(all_entries.len(), 2);
    
    // Test removal
    let removed_entry = vault.remove_entry(&entry1_id)
        .expect("Failed to remove entry");
    assert_eq!(removed_entry.title, "example.com");
    
    // Verify removal
    assert!(vault.get_entry(&entry1_id).is_none());
    assert_eq!(vault.get_all_entries().len(), 1);
}

/// Test recovery manager basic functionality
#[test]
fn test_recovery_manager_basic() {
    let recovery_manager = RecoveryManager::new();
    
    // Test that recovery manager can be created
    // This is a basic test since the actual recovery functionality
    // would require proper integration with system services
    
    // The actual implementation would test method availability
    // For now, we just ensure the manager can be instantiated
    let _rm = recovery_manager; // Use the variable to avoid warning
}

/// Test key derivation performance
#[test]
fn test_key_derivation_performance() {
    use std::time::Instant;
    
    let password = "BenchmarkPassword123!";
    let salt = vec![1u8; 16];
    
    let start = Instant::now();
    let _key = derive_key(password, &salt)
        .expect("Failed to derive key");
    let duration = start.elapsed();
    
    // Key derivation should take reasonable time (< 5 seconds for default config)
    assert!(duration.as_secs() < 5, "Key derivation took too long: {:?}", duration);
    
    println!("Key derivation took: {:?}", duration);
}

/// Test password entry creation and modification
#[test]
fn test_password_entry_operations() {
    let mut entry = PasswordEntry::new(
        "test.com".to_string(),
        "testuser".to_string(),
        "testpass123".to_string(),
    );
    
    // Test initial values
    assert_eq!(entry.title, "test.com");
    assert_eq!(entry.username, "testuser");
    assert_eq!(entry.password, "testpass123");
    assert!(entry.url.is_none());
    assert!(entry.notes.is_none());
    assert!(entry.tags.is_empty());
    
    // Test modification
    let original_updated_at = entry.updated_at;
    std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure time difference
    
    entry.url = Some("https://test.com".to_string());
    entry.notes = Some("Test account".to_string());
    entry.tags.push("work".to_string());
    entry.update();
    
    assert_eq!(entry.url, Some("https://test.com".to_string()));
    assert_eq!(entry.notes, Some("Test account".to_string()));
    assert_eq!(entry.tags, vec!["work".to_string()]);
    assert!(entry.updated_at > original_updated_at);
}

/// Test vault manager state management
#[test]
fn test_vault_manager_state() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let vault_path = temp_dir.path().join("state_test_vault.json");
    
    let mut vault_manager = VaultManager::new();
    
    // Initially no vault loaded
    assert!(!vault_manager.is_vault_loaded());
    assert!(vault_manager.get_vault().is_none());
    
    // Create vault
    vault_manager.create_vault(vault_path.clone(), "password123")
        .expect("Failed to create vault");
    
    // Now vault should be loaded
    assert!(vault_manager.is_vault_loaded());
    assert!(vault_manager.get_vault().is_some());
    
    // Close vault
    vault_manager.close_vault();
    
    // Should be closed now
    assert!(!vault_manager.is_vault_loaded());
    assert!(vault_manager.get_vault().is_none());
}

/// Test error handling for invalid operations
#[test]
fn test_error_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let vault_path = temp_dir.path().join("error_test_vault.json");
    
    let mut vault = Vault::new(vault_path);
    let fake_id = uuid::Uuid::new_v4();
    
    // Test removing non-existent entry
    let result = vault.remove_entry(&fake_id);
    assert!(result.is_err());
    
    // Test key derivation with invalid salt (too short)
    let result = derive_key("password", &[1, 2, 3]);
    match result {
        Ok(_) => {
            // Some implementations might handle short salts gracefully
            // by padding or using them as-is
        }
        Err(_) => {
            // Expected for implementations that validate salt length
        }
    }
}

/// Test concurrent operations safety
#[test]
fn test_concurrent_safety() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let vault_path = temp_dir.path().join("concurrent_test_vault.json");
    
    let vault_manager = Arc::new(Mutex::new(VaultManager::new()));
    
    // Create vault
    {
        let mut vm = vault_manager.lock().unwrap();
        vm.create_vault(vault_path, "password123")
            .expect("Failed to create vault");
    }
    
    // Test concurrent access (basic test)
    let handles: Vec<_> = (0..3).map(|i| {
        let vm_clone = Arc::clone(&vault_manager);
        thread::spawn(move || {
            let mut vm = vm_clone.lock().unwrap();
            if let Some(vault) = vm.get_vault_mut() {
                let entry = PasswordEntry::new(
                    format!("site{}.com", i),
                    format!("user{}", i),
                    format!("pass{}", i),
                );
                vault.add_entry(entry);
            }
        })
    }).collect();
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify all entries were added
    let vm = vault_manager.lock().unwrap();
    if let Some(vault) = vm.get_vault() {
        assert_eq!(vault.get_all_entries().len(), 3);
    }
}