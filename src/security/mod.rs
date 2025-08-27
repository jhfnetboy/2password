//! Advanced Security Infrastructure
//!
//! Provides enterprise-grade security features including audit logging,
//! security event monitoring, hardware security key support, and zero-trust architecture.

use crate::{Result, TwoPasswordError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod audit_log;
pub mod event_monitor;
pub mod hardware_security;
pub mod zero_trust;
pub mod compliance;

/// Security event types for audit logging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SecurityEventType {
    // Authentication events
    LoginAttempt,
    LoginSuccess,
    LoginFailure,
    LogoutRequested,
    BiometricAuth,
    PasswordAuth,
    
    // Vault operations
    VaultUnlocked,
    VaultLocked,
    VaultCreated,
    VaultDeleted,
    VaultBackup,
    VaultRestore,
    
    // Password operations
    PasswordCreated,
    PasswordUpdated,
    PasswordDeleted,
    PasswordViewed,
    PasswordCopied,
    PasswordGenerated,
    
    // Security operations
    PasswordBreachDetected,
    WeakPasswordDetected,
    DuplicatePasswordDetected,
    SecurityScanCompleted,
    
    // System events
    ApplicationStarted,
    ApplicationShutdown,
    SettingsChanged,
    ExtensionConnected,
    ExtensionDisconnected,
    
    // Compliance events
    DataExported,
    DataImported,
    AuditLogAccessed,
    ComplianceReportGenerated,
    
    // Security incidents
    UnauthorizedAccess,
    SuspiciousActivity,
    SecurityViolation,
    IntrusionDetected,
}

impl SecurityEventType {
    pub fn as_u32(&self) -> u32 {
        match self {
            SecurityEventType::LoginAttempt => 1,
            SecurityEventType::LoginSuccess => 2,
            SecurityEventType::LoginFailure => 3,
            SecurityEventType::LogoutRequested => 4,
            SecurityEventType::BiometricAuth => 5,
            SecurityEventType::PasswordAuth => 6,
            SecurityEventType::VaultUnlocked => 10,
            SecurityEventType::VaultLocked => 11,
            SecurityEventType::VaultCreated => 12,
            SecurityEventType::VaultDeleted => 13,
            SecurityEventType::VaultBackup => 14,
            SecurityEventType::VaultRestore => 15,
            SecurityEventType::PasswordCreated => 20,
            SecurityEventType::PasswordUpdated => 21,
            SecurityEventType::PasswordDeleted => 22,
            SecurityEventType::PasswordViewed => 23,
            SecurityEventType::PasswordCopied => 24,
            SecurityEventType::PasswordGenerated => 25,
            SecurityEventType::PasswordBreachDetected => 30,
            SecurityEventType::WeakPasswordDetected => 31,
            SecurityEventType::DuplicatePasswordDetected => 32,
            SecurityEventType::SecurityScanCompleted => 33,
            SecurityEventType::ApplicationStarted => 40,
            SecurityEventType::ApplicationShutdown => 41,
            SecurityEventType::SettingsChanged => 42,
            SecurityEventType::ExtensionConnected => 43,
            SecurityEventType::ExtensionDisconnected => 44,
            SecurityEventType::DataExported => 50,
            SecurityEventType::DataImported => 51,
            SecurityEventType::AuditLogAccessed => 52,
            SecurityEventType::ComplianceReportGenerated => 53,
            SecurityEventType::UnauthorizedAccess => 60,
            SecurityEventType::SuspiciousActivity => 61,
            SecurityEventType::SecurityViolation => 62,
            SecurityEventType::IntrusionDetected => 63,
        }
    }
}

/// Security event severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SecuritySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl SecuritySeverity {
    pub fn as_u32(&self) -> u32 {
        match self {
            SecuritySeverity::Info => 0,
            SecuritySeverity::Low => 1,
            SecuritySeverity::Medium => 2,
            SecuritySeverity::High => 3,
            SecuritySeverity::Critical => 4,
        }
    }
}

/// Security event structure for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: Uuid,
    pub event_type: SecurityEventType,
    pub severity: SecuritySeverity,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub device_id: String,
    pub session_id: Option<String>,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub description: String,
    pub metadata: HashMap<String, String>,
    pub success: bool,
    pub error_message: Option<String>,
}

impl SecurityEvent {
    /// Create a new security event
    pub fn new(
        event_type: SecurityEventType,
        severity: SecuritySeverity,
        description: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            severity,
            timestamp: Utc::now(),
            user_id: None,
            device_id: get_device_id(),
            session_id: None,
            source_ip: None,
            user_agent: None,
            resource_id: None,
            resource_type: None,
            description,
            metadata: HashMap::new(),
            success: true,
            error_message: None,
        }
    }
    
    /// Set user information
    pub fn with_user(mut self, user_id: String, session_id: Option<String>) -> Self {
        self.user_id = Some(user_id);
        self.session_id = session_id;
        self
    }
    
    /// Set network information
    pub fn with_network(mut self, source_ip: Option<String>, user_agent: Option<String>) -> Self {
        self.source_ip = source_ip;
        self.user_agent = user_agent;
        self
    }
    
    /// Set resource information
    pub fn with_resource(mut self, resource_id: String, resource_type: String) -> Self {
        self.resource_id = Some(resource_id);
        self.resource_type = Some(resource_type);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Mark event as failed
    pub fn with_error(mut self, error_message: String) -> Self {
        self.success = false;
        self.error_message = Some(error_message);
        self
    }
}

/// Security policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    // Authentication policies
    pub max_login_attempts: u32,
    pub login_lockout_duration_minutes: u32,
    pub session_timeout_minutes: u32,
    pub require_biometric_auth: bool,
    pub require_mfa: bool,
    
    // Password policies
    pub min_password_length: u32,
    pub require_password_complexity: bool,
    pub password_history_count: u32,
    pub max_password_age_days: u32,
    
    // Security monitoring
    pub enable_audit_logging: bool,
    pub audit_log_retention_days: u32,
    pub enable_security_monitoring: bool,
    pub enable_intrusion_detection: bool,
    
    // Compliance settings
    pub enable_compliance_mode: bool,
    pub compliance_standard: Option<String>, // "SOC2", "HIPAA", "GDPR", etc.
    pub require_audit_approval: bool,
    pub enable_data_classification: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            // Authentication defaults
            max_login_attempts: 5,
            login_lockout_duration_minutes: 15,
            session_timeout_minutes: 60,
            require_biometric_auth: false,
            require_mfa: false,
            
            // Password defaults
            min_password_length: 12,
            require_password_complexity: true,
            password_history_count: 5,
            max_password_age_days: 90,
            
            // Security monitoring defaults
            enable_audit_logging: true,
            audit_log_retention_days: 90,
            enable_security_monitoring: true,
            enable_intrusion_detection: false,
            
            // Compliance defaults
            enable_compliance_mode: false,
            compliance_standard: None,
            require_audit_approval: false,
            enable_data_classification: false,
        }
    }
}

/// Security context for operations
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub device_id: String,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
    pub permissions: Vec<String>,
    pub authentication_level: AuthenticationLevel,
}

/// Authentication strength levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationLevel {
    None,
    Password,
    Biometric,
    MultiFactor,
    HardwareKey,
}

/// Security metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    pub total_events: u64,
    pub failed_login_attempts: u64,
    pub successful_logins: u64,
    pub security_violations: u64,
    pub audit_log_size_bytes: u64,
    pub last_security_scan: Option<DateTime<Utc>>,
    pub active_sessions: u32,
    pub breach_count: u32,
    pub weak_password_count: u32,
    pub compliance_score: f64,
}

/// Get device identifier
fn get_device_id() -> String {
    // This would typically use a platform-specific method to get device ID
    // For now, we'll use a combination of system info
    use std::env;
    
    let hostname = env::var("HOSTNAME")
        .or_else(|_| env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    
    let user = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    
    format!("{}@{}", user, hostname)
}

/// Security infrastructure manager
pub struct SecurityInfrastructure {
    pub audit_log: audit_log::AuditLogger,
    pub event_monitor: event_monitor::SecurityEventMonitor,
    pub hardware_security: hardware_security::HardwareSecurityManager,
    pub zero_trust: zero_trust::ZeroTrustManager,
    pub policy: SecurityPolicy,
}

impl SecurityInfrastructure {
    /// Create new security infrastructure
    pub fn new() -> Result<Self> {
        Ok(Self {
            audit_log: audit_log::AuditLogger::new()?,
            event_monitor: event_monitor::SecurityEventMonitor::new(),
            hardware_security: hardware_security::HardwareSecurityManager::new(),
            zero_trust: zero_trust::ZeroTrustManager::new(),
            policy: SecurityPolicy::default(),
        })
    }
    
    /// Log security event
    pub async fn log_security_event(&mut self, event: SecurityEvent) -> Result<()> {
        // Log to audit system
        self.audit_log.log_event(&event).await?;
        
        // Process through event monitor
        self.event_monitor.process_event(&event).await?;
        
        // Check for security violations
        if matches!(event.severity, SecuritySeverity::High | SecuritySeverity::Critical) {
            self.handle_high_severity_event(&event).await?;
        }
        
        Ok(())
    }
    
    /// Handle high severity security events
    async fn handle_high_severity_event(&mut self, event: &SecurityEvent) -> Result<()> {
        tracing::warn!("High severity security event: {:?}", event);
        
        match event.event_type {
            SecurityEventType::UnauthorizedAccess | 
            SecurityEventType::IntrusionDetected => {
                // Immediate security response
                self.trigger_security_lockdown().await?;
            }
            SecurityEventType::SuspiciousActivity => {
                // Increase monitoring
                self.event_monitor.increase_monitoring_level().await?;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Trigger security lockdown
    async fn trigger_security_lockdown(&mut self) -> Result<()> {
        tracing::error!("Security lockdown triggered");
        
        // This would typically:
        // 1. Lock all active sessions
        // 2. Require re-authentication
        // 3. Notify administrators
        // 4. Increase audit logging
        
        // For now, just log the event
        let event = SecurityEvent::new(
            SecurityEventType::SecurityViolation,
            SecuritySeverity::Critical,
            "Security lockdown triggered due to potential threat".to_string(),
        );
        
        self.audit_log.log_event(&event).await?;
        
        Ok(())
    }
    
    /// Get security metrics
    pub async fn get_security_metrics(&self) -> Result<SecurityMetrics> {
        let audit_stats = self.audit_log.get_statistics().await?;
        let monitor_stats = self.event_monitor.get_statistics().await?;
        
        Ok(SecurityMetrics {
            total_events: audit_stats.total_events,
            failed_login_attempts: audit_stats.failed_login_attempts,
            successful_logins: audit_stats.successful_logins,
            security_violations: monitor_stats.security_violations,
            audit_log_size_bytes: audit_stats.log_size_bytes,
            last_security_scan: monitor_stats.last_scan_time,
            active_sessions: monitor_stats.active_sessions,
            breach_count: monitor_stats.breach_count,
            weak_password_count: monitor_stats.weak_password_count,
            compliance_score: self.calculate_compliance_score().await,
        })
    }
    
    /// Calculate compliance score
    async fn calculate_compliance_score(&self) -> f64 {
        // Simplified compliance scoring
        let mut score: f64 = 100.0;
        
        if !self.policy.enable_audit_logging {
            score -= 20.0;
        }
        
        if !self.policy.require_password_complexity {
            score -= 15.0;
        }
        
        if !self.policy.enable_security_monitoring {
            score -= 10.0;
        }
        
        if self.policy.session_timeout_minutes > 120 {
            score -= 5.0;
        }
        
        score.max(0.0)
    }
    
    /// Validate security context
    pub fn validate_security_context(&self, context: &SecurityContext, required_level: AuthenticationLevel) -> bool {
        match required_level {
            AuthenticationLevel::None => true,
            AuthenticationLevel::Password => {
                matches!(
                    context.authentication_level,
                    AuthenticationLevel::Password |
                    AuthenticationLevel::Biometric |
                    AuthenticationLevel::MultiFactor |
                    AuthenticationLevel::HardwareKey
                )
            }
            AuthenticationLevel::Biometric => {
                matches!(
                    context.authentication_level,
                    AuthenticationLevel::Biometric |
                    AuthenticationLevel::MultiFactor |
                    AuthenticationLevel::HardwareKey
                )
            }
            AuthenticationLevel::MultiFactor => {
                matches!(
                    context.authentication_level,
                    AuthenticationLevel::MultiFactor |
                    AuthenticationLevel::HardwareKey
                )
            }
            AuthenticationLevel::HardwareKey => {
                matches!(context.authentication_level, AuthenticationLevel::HardwareKey)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_event_creation() {
        let event = SecurityEvent::new(
            SecurityEventType::LoginSuccess,
            SecuritySeverity::Info,
            "User logged in successfully".to_string(),
        );

        assert_eq!(event.event_type, SecurityEventType::LoginSuccess);
        assert_eq!(event.severity, SecuritySeverity::Info);
        assert!(event.success);
        assert!(event.error_message.is_none());
    }

    #[test]
    fn test_security_event_with_metadata() {
        let event = SecurityEvent::new(
            SecurityEventType::PasswordCreated,
            SecuritySeverity::Low,
            "New password created".to_string(),
        )
        .with_user("user123".to_string(), Some("session456".to_string()))
        .with_resource("password789".to_string(), "password".to_string())
        .with_metadata("complexity_score".to_string(), "85".to_string());

        assert_eq!(event.user_id, Some("user123".to_string()));
        assert_eq!(event.session_id, Some("session456".to_string()));
        assert_eq!(event.resource_id, Some("password789".to_string()));
        assert_eq!(event.metadata.get("complexity_score"), Some(&"85".to_string()));
    }

    #[test]
    fn test_security_policy_defaults() {
        let policy = SecurityPolicy::default();
        
        assert_eq!(policy.max_login_attempts, 5);
        assert_eq!(policy.min_password_length, 12);
        assert!(policy.enable_audit_logging);
        assert!(!policy.enable_compliance_mode);
    }

    #[test]
    fn test_authentication_level_validation() {
        let infrastructure = SecurityInfrastructure {
            audit_log: audit_log::AuditLogger::new().unwrap(),
            event_monitor: event_monitor::SecurityEventMonitor::new(),
            hardware_security: hardware_security::HardwareSecurityManager::new(),
            zero_trust: zero_trust::ZeroTrustManager::new(),
            policy: SecurityPolicy::default(),
        };

        let context = SecurityContext {
            user_id: Some("user123".to_string()),
            session_id: Some("session456".to_string()),
            device_id: "device789".to_string(),
            source_ip: None,
            user_agent: None,
            permissions: vec!["read".to_string()],
            authentication_level: AuthenticationLevel::Biometric,
        };

        assert!(infrastructure.validate_security_context(&context, AuthenticationLevel::Password));
        assert!(infrastructure.validate_security_context(&context, AuthenticationLevel::Biometric));
        assert!(!infrastructure.validate_security_context(&context, AuthenticationLevel::HardwareKey));
    }
}