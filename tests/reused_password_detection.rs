//! Test for reused password detection functionality in Task 17.2

use twopassword::password_health::PasswordHealthService;
use twopassword::storage::PasswordEntry;
use uuid::Uuid;

#[test]
fn test_reused_password_detection() {
    let health_service = PasswordHealthService::new();
    
    // Create test entries with some reused passwords
    let entries = vec![
        PasswordEntry {
            id: Uuid::new_v4(),
            title: "Gmail".to_string(),
            username: "user@gmail.com".to_string(),
            password: "password123".to_string(),
            url: Some("https://gmail.com".to_string()),
            notes: None,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        PasswordEntry {
            id: Uuid::new_v4(),
            title: "Facebook".to_string(),
            username: "user@example.com".to_string(),
            password: "password123".to_string(), // Same as Gmail - should be detected as reused
            url: Some("https://facebook.com".to_string()),
            notes: None,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        PasswordEntry {
            id: Uuid::new_v4(),
            title: "Bank Account".to_string(),
            username: "user123".to_string(),
            password: "uniquePassword456".to_string(), // Unique password
            url: Some("https://mybank.com".to_string()),
            notes: None,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        PasswordEntry {
            id: Uuid::new_v4(),
            title: "Amazon".to_string(),
            username: "user@example.com".to_string(),
            password: "password123".to_string(), // Same as Gmail and Facebook - makes it a group of 3
            url: Some("https://amazon.com".to_string()),
            notes: None,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];

    // Run reused password detection
    let reused_groups = health_service.detect_reused_passwords(&entries).unwrap();
    
    // Should find exactly one group with 3 entries (Gmail, Facebook, Amazon)
    assert_eq!(reused_groups.len(), 1, "Should find exactly one group of reused passwords");
    
    let reused_group = &reused_groups[0];
    assert_eq!(reused_group.entries.len(), 3, "The reused group should contain 3 entries");
    
    // Check that the risk level is appropriate (Critical due to Amazon being high-value)
    assert_eq!(reused_group.risk_level, twopassword::password_health::RiskLevel::Critical);
    
    println!("✅ Reused password detection working correctly!");
    println!("Found {} reused password group with {} entries", 
             reused_groups.len(), reused_group.entries.len());
    println!("Risk level: {:?}", reused_group.risk_level);
}

#[test] 
fn test_no_reused_passwords() {
    let health_service = PasswordHealthService::new();
    
    // Create test entries with all unique passwords
    let entries = vec![
        PasswordEntry {
            id: Uuid::new_v4(),
            title: "Gmail".to_string(),
            username: "user@gmail.com".to_string(),
            password: "uniquePassword1".to_string(),
            url: Some("https://gmail.com".to_string()),
            notes: None,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        PasswordEntry {
            id: Uuid::new_v4(),
            title: "Facebook".to_string(),
            username: "user@example.com".to_string(),
            password: "uniquePassword2".to_string(),
            url: Some("https://facebook.com".to_string()),
            notes: None,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];

    // Run reused password detection
    let reused_groups = health_service.detect_reused_passwords(&entries).unwrap();
    
    // Should find no reused password groups
    assert_eq!(reused_groups.len(), 0, "Should find no reused password groups");
    
    println!("✅ No reused passwords detected correctly when all passwords are unique!");
}

#[test]
fn test_risk_level_assessment() {
    let health_service = PasswordHealthService::new();
    
    // Test Critical risk (financial site)
    let critical_entries = vec![
        PasswordEntry {
            id: Uuid::new_v4(),
            title: "Bank".to_string(),
            username: "user".to_string(),
            password: "shared123".to_string(),
            url: Some("https://mybank.com".to_string()),
            notes: None,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        PasswordEntry {
            id: Uuid::new_v4(),
            title: "PayPal".to_string(),
            username: "user".to_string(),
            password: "shared123".to_string(),
            url: Some("https://paypal.com".to_string()),
            notes: None,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];
    
    let critical_groups = health_service.detect_reused_passwords(&critical_entries).unwrap();
    assert_eq!(critical_groups.len(), 1);
    assert_eq!(critical_groups[0].risk_level, twopassword::password_health::RiskLevel::Critical);
    
    // Test Medium risk (few non-critical sites)  
    let medium_entries = vec![
        PasswordEntry {
            id: Uuid::new_v4(),
            title: "Forum1".to_string(),
            username: "user".to_string(),
            password: "shared123".to_string(),
            url: Some("https://forum1.com".to_string()),
            notes: None,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        PasswordEntry {
            id: Uuid::new_v4(),
            title: "Forum2".to_string(),
            username: "user".to_string(),
            password: "shared123".to_string(),
            url: Some("https://forum2.com".to_string()),
            notes: None,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        PasswordEntry {
            id: Uuid::new_v4(),
            title: "Forum3".to_string(),
            username: "user".to_string(),
            password: "shared123".to_string(),
            url: Some("https://forum3.com".to_string()),
            notes: None,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        PasswordEntry {
            id: Uuid::new_v4(),
            title: "Forum4".to_string(),
            username: "user".to_string(),
            password: "shared123".to_string(),
            url: Some("https://forum4.com".to_string()),
            notes: None,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];
    
    let medium_groups = health_service.detect_reused_passwords(&medium_entries).unwrap();
    assert_eq!(medium_groups.len(), 1);
    assert_eq!(medium_groups[0].risk_level, twopassword::password_health::RiskLevel::Medium);
    
    println!("✅ Risk level assessment working correctly!");
    println!("Critical risk for financial sites: {:?}", critical_groups[0].risk_level);
    println!("Medium risk for multiple non-critical sites: {:?}", medium_groups[0].risk_level);
}