//! Advanced Security Infrastructure Tests
//!
//! Comprehensive tests for the security infrastructure modules including
//! audit logging, event monitoring, hardware security, zero trust, and compliance.

use twopassword::security::{
    SecurityEvent, SecurityEventType, SecuritySeverity, SecurityInfrastructure,
    audit_log::AuditLogger,
    event_monitor::SecurityEventMonitor,
    hardware_security::HardwareSecurityManager,
    zero_trust::ZeroTrustManager,
    compliance::ComplianceManager,
};
use twopassword::Result;
use chrono::Utc;
use std::collections::HashMap;

#[tokio::test]
async fn test_security_infrastructure_creation() -> Result<()> {
    let security_infra = SecurityInfrastructure::new()?;
    
    // Test that all components are initialized
    assert_eq!(security_infra.policy.max_login_attempts, 5);
    assert_eq!(security_infra.policy.min_password_length, 12);
    assert!(security_infra.policy.enable_audit_logging);
    
    println!("✅ Security infrastructure created successfully");
    Ok(())
}

#[tokio::test]
async fn test_audit_logger_functionality() -> Result<()> {
    let mut audit_logger = AuditLogger::new()?;
    
    let event = SecurityEvent::new(
        SecurityEventType::LoginSuccess,
        SecuritySeverity::Info,
        "Test login successful".to_string(),
    );
    
    audit_logger.log_event(&event).await?;
    
    let stats = audit_logger.get_statistics().await?;
    assert_eq!(stats.total_events, 1);
    assert_eq!(stats.successful_logins, 1);
    
    println!("✅ Audit logger functionality verified");
    Ok(())
}

#[tokio::test]
async fn test_event_monitor_threat_detection() -> Result<()> {
    let monitor = SecurityEventMonitor::new();
    
    // Simulate brute force attack
    for i in 0..6 {
        let event = SecurityEvent::new(
            SecurityEventType::LoginFailure,
            SecuritySeverity::Medium,
            format!("Failed login attempt {}", i + 1),
        );
        monitor.process_event(&event).await?;
    }
    
    let anomalies = monitor.get_recent_anomalies(None).await?;
    assert!(!anomalies.is_empty(), "Should detect brute force pattern");
    
    let stats = monitor.get_statistics().await?;
    assert!(stats.anomalies_detected > 0);
    
    println!("✅ Event monitor threat detection working correctly");
    Ok(())
}

#[tokio::test]
async fn test_hardware_security_key_management() -> Result<()> {
    let mut hw_manager = HardwareSecurityManager::new();
    
    // Discover available keys
    let discovered_keys = hw_manager.discover_keys().await?;
    println!("Discovered {} hardware security keys", discovered_keys.len());
    
    // Test key registration (simulate)
    if !discovered_keys.is_empty() {
        let key = discovered_keys[0].clone();
        let key_id = hw_manager.register_key(key).await?;
        
        let registered_keys = hw_manager.get_registered_keys();
        assert_eq!(registered_keys.len(), 1);
        assert_eq!(registered_keys[0].id, key_id);
        
        println!("✅ Hardware security key management verified");
    } else {
        println!("⚠️  No hardware security keys available for testing");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_zero_trust_access_evaluation() -> Result<()> {
    let mut zt_manager = ZeroTrustManager::new();
    
    let context = twopassword::security::SecurityContext {
        user_id: Some("test_user".to_string()),
        session_id: Some("test_session".to_string()),
        device_id: "test_device".to_string(),
        source_ip: Some("192.168.1.100".to_string()),
        user_agent: Some("TestAgent/1.0".to_string()),
        permissions: vec!["read".to_string()],
        authentication_level: twopassword::security::AuthenticationLevel::Password,
    };
    
    let decision = zt_manager.evaluate_access_request(&context, "passwords").await?;
    
    assert!(decision.trust_score >= 0.0 && decision.trust_score <= 1.0);
    assert!(decision.risk_score >= 0.0 && decision.risk_score <= 1.0);
    assert!(!decision.reason.is_empty());
    
    println!("✅ Zero trust access evaluation completed");
    println!("   Trust Score: {:.2}", decision.trust_score);
    println!("   Risk Score: {:.2}", decision.risk_score);
    println!("   Access Granted: {}", decision.granted);
    Ok(())
}

#[tokio::test]
async fn test_compliance_assessment() -> Result<()> {
    let mut compliance_manager = ComplianceManager::new();
    
    // Run SOC2 compliance assessment
    let assessment = compliance_manager.run_assessment(
        twopassword::security::compliance::ComplianceStandard::SOC2
    ).await?;
    
    assert_eq!(assessment.standard, twopassword::security::compliance::ComplianceStandard::SOC2);
    assert!(assessment.total_requirements > 0);
    assert!(assessment.overall_score >= 0.0 && assessment.overall_score <= 1.0);
    
    // Run GDPR compliance assessment
    let gdpr_assessment = compliance_manager.run_assessment(
        twopassword::security::compliance::ComplianceStandard::GDPR
    ).await?;
    
    assert_eq!(gdpr_assessment.standard, twopassword::security::compliance::ComplianceStandard::GDPR);
    assert!(gdpr_assessment.total_requirements > 0);
    
    let report = compliance_manager.generate_compliance_report().await?;
    assert_eq!(report.enabled_standards.len(), 2);
    
    println!("✅ Compliance assessments completed");
    println!("   SOC2 Score: {:.2} ({} requirements)", assessment.overall_score, assessment.total_requirements);
    println!("   GDPR Score: {:.2} ({} requirements)", gdpr_assessment.overall_score, gdpr_assessment.total_requirements);
    println!("   Overall Compliance: {:.1}%", report.overall_compliance_percentage);
    Ok(())
}

#[tokio::test]
async fn test_integrated_security_workflow() -> Result<()> {
    let mut security_infra = SecurityInfrastructure::new()?;
    
    // Create a high-severity security event
    let security_event = SecurityEvent::new(
        SecurityEventType::IntrusionDetected,
        SecuritySeverity::Critical,
        "Potential intrusion attempt detected".to_string(),
    ).with_user("user123".to_string(), Some("session456".to_string()))
     .with_network(Some("192.168.1.1".to_string()), Some("MaliciousAgent/1.0".to_string()))
     .with_resource("vault123".to_string(), "password_vault".to_string())
     .with_metadata("attack_type".to_string(), "brute_force".to_string());
    
    // Process through integrated security infrastructure
    security_infra.log_security_event(security_event).await?;
    
    // Get security metrics
    let metrics = security_infra.get_security_metrics().await?;
    assert!(metrics.security_violations > 0);
    
    println!("✅ Integrated security workflow completed");
    println!("   Security Violations: {}", metrics.security_violations);
    Ok(())
}

#[tokio::test]
async fn test_security_policy_validation() -> Result<()> {
    let security_infra = SecurityInfrastructure::new()?;
    
    let context = twopassword::security::SecurityContext {
        user_id: Some("test_user".to_string()),
        session_id: Some("test_session".to_string()),
        device_id: "test_device".to_string(),
        source_ip: Some("192.168.1.100".to_string()),
        user_agent: None,
        permissions: vec!["admin".to_string()],
        authentication_level: twopassword::security::AuthenticationLevel::MultiFactor,
    };
    
    // Test different authentication levels
    let levels = vec![
        twopassword::security::AuthenticationLevel::Password,
        twopassword::security::AuthenticationLevel::Biometric,
        twopassword::security::AuthenticationLevel::MultiFactor,
        twopassword::security::AuthenticationLevel::HardwareKey,
    ];
    
    for level in levels {
        let is_valid = security_infra.validate_security_context(&context, level.clone());
        println!("   Authentication level {:?}: {}", level, if is_valid { "✅ Valid" } else { "❌ Invalid" });
    }
    
    println!("✅ Security policy validation completed");
    Ok(())
}

#[tokio::test]
async fn test_comprehensive_security_metrics() -> Result<()> {
    let mut security_infra = SecurityInfrastructure::new()?;
    
    // Generate various security events
    let events = vec![
        (SecurityEventType::LoginSuccess, SecuritySeverity::Info, "User login"),
        (SecurityEventType::LoginFailure, SecuritySeverity::Medium, "Failed login"),
        (SecurityEventType::PasswordBreachDetected, SecuritySeverity::High, "Breached password found"),
        (SecurityEventType::WeakPasswordDetected, SecuritySeverity::Medium, "Weak password detected"),
        (SecurityEventType::SecurityViolation, SecuritySeverity::Critical, "Security policy violation"),
    ];
    
    for (event_type, severity, description) in events {
        let event = SecurityEvent::new(event_type, severity, description.to_string());
        security_infra.log_security_event(event).await?;
    }
    
    let metrics = security_infra.get_security_metrics().await?;
    
    assert_eq!(metrics.successful_logins, 1);
    assert_eq!(metrics.failed_login_attempts, 1);
    assert_eq!(metrics.breach_count, 1);
    assert_eq!(metrics.weak_password_count, 1);
    assert_eq!(metrics.security_violations, 1);
    assert!(metrics.compliance_score >= 0.0 && metrics.compliance_score <= 100.0);
    
    println!("✅ Comprehensive security metrics verified");
    println!("   Successful Logins: {}", metrics.successful_logins);
    println!("   Failed Logins: {}", metrics.failed_login_attempts);
    println!("   Breach Count: {}", metrics.breach_count);
    println!("   Weak Passwords: {}", metrics.weak_password_count);
    println!("   Security Violations: {}", metrics.security_violations);
    println!("   Compliance Score: {:.1}%", metrics.compliance_score);
    
    Ok(())
}

#[tokio::test]
async fn test_audit_log_integrity_verification() -> Result<()> {
    let audit_logger = AuditLogger::new()?;
    
    // Test integrity verification on clean audit log
    let is_valid = audit_logger.verify_integrity().await?;
    assert!(is_valid, "Clean audit log should pass integrity check");
    
    println!("✅ Audit log integrity verification completed");
    Ok(())
}

#[tokio::test]
async fn test_security_event_search_and_filtering() -> Result<()> {
    let mut audit_logger = AuditLogger::new()?;
    
    // Add various events with different types and severities
    let events = vec![
        SecurityEvent::new(SecurityEventType::LoginSuccess, SecuritySeverity::Info, "Login 1".to_string()),
        SecurityEvent::new(SecurityEventType::LoginFailure, SecuritySeverity::Medium, "Failed login 1".to_string()),
        SecurityEvent::new(SecurityEventType::LoginFailure, SecuritySeverity::Medium, "Failed login 2".to_string()),
        SecurityEvent::new(SecurityEventType::SecurityViolation, SecuritySeverity::Critical, "Violation".to_string()),
    ];
    
    for event in &events {
        audit_logger.log_event(event).await?;
    }
    
    // Search for login failures
    let login_failures = audit_logger.search_events(
        None, // start_time
        None, // end_time
        Some(vec![SecurityEventType::LoginFailure]),
        None, // severity
        None, // user_id
        None, // limit
    ).await?;
    
    assert_eq!(login_failures.len(), 2, "Should find 2 login failures");
    
    // Search for critical events
    let critical_events = audit_logger.search_events(
        None,
        None,
        None,
        Some(SecuritySeverity::Critical),
        None,
        None,
    ).await?;
    
    assert_eq!(critical_events.len(), 1, "Should find 1 critical event");
    
    println!("✅ Security event search and filtering verified");
    Ok(())
}