//! Security event monitoring and threat detection
//!
//! Real-time monitoring of security events with pattern detection,
//! anomaly detection, and automated response capabilities.

use crate::security::{SecurityEvent, SecurityEventType, SecuritySeverity};
use crate::{Result, TwoPasswordError};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Security monitoring statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMonitoringStats {
    pub security_violations: u64,
    pub anomalies_detected: u64,
    pub threats_blocked: u64,
    pub active_sessions: u32,
    pub breach_count: u32,
    pub weak_password_count: u32,
    pub last_scan_time: Option<DateTime<Utc>>,
    pub monitoring_level: MonitoringLevel,
}

/// Monitoring sensitivity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MonitoringLevel {
    Low,      // Basic monitoring
    Normal,   // Standard monitoring
    High,     // Enhanced monitoring
    Critical, // Maximum sensitivity
}

/// Threat pattern definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatPattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub event_types: Vec<SecurityEventType>,
    pub time_window_minutes: u32,
    pub threshold_count: u32,
    pub severity: SecuritySeverity,
    pub enabled: bool,
}

/// Detected security anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnomaly {
    pub id: String,
    pub pattern_id: String,
    pub detected_at: DateTime<Utc>,
    pub events: Vec<SecurityEvent>,
    pub risk_score: f64,
    pub recommended_action: String,
    pub auto_response_taken: Option<String>,
}

/// Event monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub monitoring_level: MonitoringLevel,
    pub event_retention_hours: u32,
    pub max_events_per_pattern: usize,
    pub anomaly_detection_enabled: bool,
    pub auto_response_enabled: bool,
    pub notification_enabled: bool,
    pub threat_patterns: Vec<ThreatPattern>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            monitoring_level: MonitoringLevel::Normal,
            event_retention_hours: 24,
            max_events_per_pattern: 1000,
            anomaly_detection_enabled: true,
            auto_response_enabled: false,
            notification_enabled: true,
            threat_patterns: Self::default_threat_patterns(),
        }
    }
}

impl MonitoringConfig {
    fn default_threat_patterns() -> Vec<ThreatPattern> {
        vec![
            ThreatPattern {
                id: "brute_force_login".to_string(),
                name: "Brute Force Login Attempts".to_string(),
                description: "Multiple failed login attempts from the same source".to_string(),
                event_types: vec![SecurityEventType::LoginFailure],
                time_window_minutes: 5,
                threshold_count: 5,
                severity: SecuritySeverity::High,
                enabled: true,
            },
            ThreatPattern {
                id: "rapid_password_changes".to_string(),
                name: "Rapid Password Changes".to_string(),
                description: "Suspicious rapid password modification pattern".to_string(),
                event_types: vec![SecurityEventType::PasswordUpdated],
                time_window_minutes: 10,
                threshold_count: 10,
                severity: SecuritySeverity::Medium,
                enabled: true,
            },
            ThreatPattern {
                id: "mass_data_access".to_string(),
                name: "Mass Data Access".to_string(),
                description: "Rapid access to multiple password entries".to_string(),
                event_types: vec![SecurityEventType::PasswordViewed, SecurityEventType::PasswordCopied],
                time_window_minutes: 5,
                threshold_count: 20,
                severity: SecuritySeverity::High,
                enabled: true,
            },
            ThreatPattern {
                id: "unauthorized_access_attempts".to_string(),
                name: "Unauthorized Access Attempts".to_string(),
                description: "Multiple unauthorized access attempts".to_string(),
                event_types: vec![SecurityEventType::UnauthorizedAccess],
                time_window_minutes: 15,
                threshold_count: 3,
                severity: SecuritySeverity::Critical,
                enabled: true,
            },
            ThreatPattern {
                id: "suspicious_vault_operations".to_string(),
                name: "Suspicious Vault Operations".to_string(),
                description: "Unusual vault creation/deletion patterns".to_string(),
                event_types: vec![SecurityEventType::VaultCreated, SecurityEventType::VaultDeleted],
                time_window_minutes: 30,
                threshold_count: 5,
                severity: SecuritySeverity::Medium,
                enabled: true,
            },
        ]
    }
}

/// Security event monitor with real-time threat detection
pub struct SecurityEventMonitor {
    config: MonitoringConfig,
    event_buffer: Arc<RwLock<VecDeque<SecurityEvent>>>,
    pattern_events: Arc<RwLock<HashMap<String, VecDeque<SecurityEvent>>>>,
    detected_anomalies: Arc<RwLock<Vec<SecurityAnomaly>>>,
    stats: Arc<RwLock<SecurityMonitoringStats>>,
}

impl SecurityEventMonitor {
    /// Create new security event monitor
    pub fn new() -> Self {
        Self {
            config: MonitoringConfig::default(),
            event_buffer: Arc::new(RwLock::new(VecDeque::new())),
            pattern_events: Arc::new(RwLock::new(HashMap::new())),
            detected_anomalies: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(SecurityMonitoringStats {
                security_violations: 0,
                anomalies_detected: 0,
                threats_blocked: 0,
                active_sessions: 0,
                breach_count: 0,
                weak_password_count: 0,
                last_scan_time: None,
                monitoring_level: MonitoringLevel::Normal,
            })),
        }
    }
    
    /// Create monitor with custom configuration
    pub fn with_config(config: MonitoringConfig) -> Self {
        let mut monitor = Self::new();
        monitor.config = config.clone();
        monitor
    }
    
    /// Process incoming security event
    pub async fn process_event(&self, event: &SecurityEvent) -> Result<()> {
        // Add to event buffer
        {
            let mut buffer = self.event_buffer.write().await;
            buffer.push_back(event.clone());
            
            // Maintain buffer size based on retention policy
            let retention_cutoff = Utc::now() - Duration::hours(self.config.event_retention_hours as i64);
            while let Some(front_event) = buffer.front() {
                if front_event.timestamp < retention_cutoff {
                    buffer.pop_front();
                } else {
                    break;
                }
            }
        }
        
        // Update statistics
        self.update_statistics(event).await;
        
        // Check for threat patterns if enabled
        if self.config.anomaly_detection_enabled {
            self.check_threat_patterns(event).await?;
        }
        
        // Perform real-time anomaly detection
        if matches!(self.config.monitoring_level, MonitoringLevel::High | MonitoringLevel::Critical) {
            self.detect_behavioral_anomalies(event).await?;
        }
        
        Ok(())
    }
    
    /// Update monitoring statistics
    async fn update_statistics(&self, event: &SecurityEvent) {
        let mut stats = self.stats.write().await;
        
        match event.event_type {
            SecurityEventType::SecurityViolation |
            SecurityEventType::UnauthorizedAccess |
            SecurityEventType::IntrusionDetected => {
                stats.security_violations += 1;
            }
            SecurityEventType::PasswordBreachDetected => {
                stats.breach_count += 1;
            }
            SecurityEventType::WeakPasswordDetected => {
                stats.weak_password_count += 1;
            }
            _ => {}
        }
        
        stats.last_scan_time = Some(Utc::now());
    }
    
    /// Check for known threat patterns
    async fn check_threat_patterns(&self, event: &SecurityEvent) -> Result<()> {
        let mut pattern_events = self.pattern_events.write().await;
        
        for pattern in &self.config.threat_patterns {
            if !pattern.enabled || !pattern.event_types.contains(&event.event_type) {
                continue;
            }
            
            // Get or create event list for this pattern
            let events = pattern_events
                .entry(pattern.id.clone())
                .or_insert_with(VecDeque::new);
            
            // Add current event
            events.push_back(event.clone());
            
            // Clean up old events outside the time window
            let time_window = Duration::minutes(pattern.time_window_minutes as i64);
            let cutoff_time = Utc::now() - time_window;
            
            while let Some(front_event) = events.front() {
                if front_event.timestamp < cutoff_time {
                    events.pop_front();
                } else {
                    break;
                }
            }
            
            // Limit buffer size
            while events.len() > self.config.max_events_per_pattern {
                events.pop_front();
            }
            
            // Check if threshold is exceeded
            if events.len() >= pattern.threshold_count as usize {
                self.handle_pattern_detection(pattern, events.clone().into()).await?;
            }
        }
        
        Ok(())
    }
    
    /// Handle detected threat pattern
    async fn handle_pattern_detection(
        &self,
        pattern: &ThreatPattern,
        events: Vec<SecurityEvent>,
    ) -> Result<()> {
        let anomaly_id = format!("{}_{}", pattern.id, Utc::now().timestamp());
        
        let risk_score = self.calculate_risk_score(pattern, &events);
        let recommended_action = self.get_recommended_action(pattern, risk_score);
        
        let anomaly = SecurityAnomaly {
            id: anomaly_id.clone(),
            pattern_id: pattern.id.clone(),
            detected_at: Utc::now(),
            events: events.clone(),
            risk_score,
            recommended_action: recommended_action.clone(),
            auto_response_taken: None,
        };
        
        // Log the anomaly
        tracing::warn!(
            "Security anomaly detected: {} (Risk Score: {:.2})",
            pattern.name,
            risk_score
        );
        
        // Store anomaly
        {
            let mut anomalies = self.detected_anomalies.write().await;
            anomalies.push(anomaly);
            
            // Update stats
            let mut stats = self.stats.write().await;
            stats.anomalies_detected += 1;
        }
        
        // Take automatic response if enabled and risk is high
        if self.config.auto_response_enabled && risk_score > 0.7 {
            self.execute_auto_response(pattern, &anomaly_id, &recommended_action).await?;
        }
        
        Ok(())
    }
    
    /// Calculate risk score for detected pattern
    fn calculate_risk_score(&self, pattern: &ThreatPattern, events: &[SecurityEvent]) -> f64 {
        let mut base_score = match pattern.severity {
            SecuritySeverity::Critical => 0.9,
            SecuritySeverity::High => 0.7,
            SecuritySeverity::Medium => 0.5,
            SecuritySeverity::Low => 0.3,
            SecuritySeverity::Info => 0.1,
        };
        
        // Adjust based on event frequency
        let frequency_multiplier = (events.len() as f64 / pattern.threshold_count as f64).min(2.0);
        base_score *= frequency_multiplier;
        
        // Adjust based on time distribution (rapid events = higher risk)
        if events.len() >= 2 {
            let time_span = events.last().unwrap().timestamp - events.first().unwrap().timestamp;
            let time_concentration = pattern.time_window_minutes as f64 / time_span.num_minutes().max(1) as f64;
            base_score *= (1.0 + time_concentration.min(1.0));
        }
        
        // Adjust based on monitoring level
        match self.config.monitoring_level {
            MonitoringLevel::Critical => base_score *= 1.2,
            MonitoringLevel::High => base_score *= 1.1,
            MonitoringLevel::Normal => {},
            MonitoringLevel::Low => base_score *= 0.8,
        }
        
        base_score.min(1.0)
    }
    
    /// Get recommended action for threat pattern
    fn get_recommended_action(&self, pattern: &ThreatPattern, risk_score: f64) -> String {
        match pattern.id.as_str() {
            "brute_force_login" => {
                if risk_score > 0.8 {
                    "Temporarily block source IP and require additional authentication".to_string()
                } else {
                    "Increase login attempt monitoring and consider CAPTCHA".to_string()
                }
            }
            "rapid_password_changes" => {
                if risk_score > 0.7 {
                    "Review account activity and require security questions".to_string()
                } else {
                    "Monitor user behavior patterns".to_string()
                }
            }
            "mass_data_access" => {
                if risk_score > 0.8 {
                    "Immediately lock account and require administrator review".to_string()
                } else {
                    "Require additional authentication for bulk operations".to_string()
                }
            }
            "unauthorized_access_attempts" => {
                "Immediately lock affected resources and notify security team".to_string()
            }
            "suspicious_vault_operations" => {
                if risk_score > 0.6 {
                    "Require administrator approval for vault operations".to_string()
                } else {
                    "Increase monitoring of vault activities".to_string()
                }
            }
            _ => "Review security logs and consider additional monitoring".to_string(),
        }
    }
    
    /// Execute automatic security response
    async fn execute_auto_response(
        &self,
        pattern: &ThreatPattern,
        anomaly_id: &str,
        recommended_action: &str,
    ) -> Result<()> {
        tracing::info!("Executing auto-response for anomaly {}: {}", anomaly_id, recommended_action);
        
        let mut response_taken = None;
        
        // Implement basic auto-responses
        match pattern.id.as_str() {
            "brute_force_login" => {
                // In a real implementation, this would temporarily block the source
                response_taken = Some("Increased login monitoring activated".to_string());
            }
            "mass_data_access" => {
                // In a real implementation, this would trigger session review
                response_taken = Some("Enhanced session monitoring activated".to_string());
            }
            "unauthorized_access_attempts" => {
                // Critical response - would normally lock affected resources
                response_taken = Some("Security lockdown procedures initiated".to_string());
            }
            _ => {
                response_taken = Some("Enhanced monitoring activated".to_string());
            }
        }
        
        // Update anomaly with response taken
        if let Some(response) = response_taken {
            let mut anomalies = self.detected_anomalies.write().await;
            if let Some(anomaly) = anomalies.iter_mut().find(|a| a.id == anomaly_id) {
                anomaly.auto_response_taken = Some(response.clone());
            }
            
            // Update stats
            let mut stats = self.stats.write().await;
            stats.threats_blocked += 1;
            
            tracing::info!("Auto-response completed: {}", response);
        }
        
        Ok(())
    }
    
    /// Detect behavioral anomalies using simple statistical analysis
    async fn detect_behavioral_anomalies(&self, event: &SecurityEvent) -> Result<()> {
        // Simple anomaly detection based on event frequency
        let buffer = self.event_buffer.read().await;
        
        if buffer.len() < 10 {
            return Ok(()); // Need more data for analysis
        }
        
        // Check for unusual event frequency
        let recent_events: Vec<_> = buffer
            .iter()
            .rev()
            .take(10)
            .filter(|e| e.event_type == event.event_type)
            .collect();
        
        if recent_events.len() >= 5 {
            // Check time distribution
            let mut intervals = Vec::new();
            for i in 1..recent_events.len() {
                let interval = recent_events[i-1].timestamp - recent_events[i].timestamp;
                intervals.push(interval.num_seconds().abs());
            }
            
            if !intervals.is_empty() {
                let avg_interval = intervals.iter().sum::<i64>() / intervals.len() as i64;
                
                // If events are happening much faster than average, flag as anomaly
                if let Some(last_interval) = intervals.last() {
                    if *last_interval < avg_interval / 3 && avg_interval > 10 {
                        tracing::warn!(
                            "Behavioral anomaly detected: Rapid {} events ({}s vs {}s avg)",
                            format!("{:?}", event.event_type),
                            last_interval,
                            avg_interval
                        );
                        
                        let mut stats = self.stats.write().await;
                        stats.anomalies_detected += 1;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get monitoring statistics
    pub async fn get_statistics(&self) -> Result<SecurityMonitoringStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }
    
    /// Get recent anomalies
    pub async fn get_recent_anomalies(&self, limit: Option<usize>) -> Result<Vec<SecurityAnomaly>> {
        let anomalies = self.detected_anomalies.read().await;
        let mut recent: Vec<_> = anomalies.clone();
        
        // Sort by detection time (most recent first)
        recent.sort_by(|a, b| b.detected_at.cmp(&a.detected_at));
        
        if let Some(limit) = limit {
            recent.truncate(limit);
        }
        
        Ok(recent)
    }
    
    /// Increase monitoring level temporarily
    pub async fn increase_monitoring_level(&mut self) -> Result<()> {
        self.config.monitoring_level = match self.config.monitoring_level {
            MonitoringLevel::Low => MonitoringLevel::Normal,
            MonitoringLevel::Normal => MonitoringLevel::High,
            MonitoringLevel::High => MonitoringLevel::Critical,
            MonitoringLevel::Critical => MonitoringLevel::Critical, // Already at max
        };
        
        tracing::info!("Monitoring level increased to {:?}", self.config.monitoring_level);
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.monitoring_level = self.config.monitoring_level.clone();
        
        Ok(())
    }
    
    /// Reset monitoring level to normal
    pub async fn reset_monitoring_level(&mut self) -> Result<()> {
        self.config.monitoring_level = MonitoringLevel::Normal;
        
        let mut stats = self.stats.write().await;
        stats.monitoring_level = MonitoringLevel::Normal;
        
        tracing::info!("Monitoring level reset to Normal");
        Ok(())
    }
    
    /// Clear old anomalies beyond retention period
    pub async fn cleanup_old_anomalies(&self, retention_days: u32) -> Result<()> {
        let cutoff_time = Utc::now() - Duration::days(retention_days as i64);
        
        let mut anomalies = self.detected_anomalies.write().await;
        anomalies.retain(|anomaly| anomaly.detected_at > cutoff_time);
        
        Ok(())
    }
    
    /// Export security monitoring report
    pub async fn generate_security_report(&self) -> Result<SecurityReport> {
        let stats = self.get_statistics().await?;
        let anomalies = self.get_recent_anomalies(Some(50)).await?;
        let buffer = self.event_buffer.read().await;
        
        // Analyze event trends
        let mut event_counts = HashMap::new();
        for event in buffer.iter() {
            let event_type = format!("{:?}", event.event_type);
            *event_counts.entry(event_type).or_insert(0u32) += 1;
        }
        
        let monitoring_level = stats.monitoring_level;
        let security_violations = stats.security_violations;
        let anomalies_detected = stats.anomalies_detected;
        let threats_blocked = stats.threats_blocked;
        let recommendations = self.generate_security_recommendations(&stats).await;
        
        Ok(SecurityReport {
            generated_at: Utc::now(),
            monitoring_level,
            total_events_monitored: buffer.len() as u64,
            security_violations,
            anomalies_detected,
            threats_blocked,
            event_type_distribution: event_counts,
            recent_anomalies: anomalies,
            recommendations,
        })
    }
    
    /// Generate security recommendations based on current state
    async fn generate_security_recommendations(&self, stats: &SecurityMonitoringStats) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if stats.security_violations > 10 {
            recommendations.push("High number of security violations detected. Consider reviewing access controls.".to_string());
        }
        
        if stats.breach_count > 5 {
            recommendations.push("Multiple breached passwords detected. Implement mandatory password updates.".to_string());
        }
        
        if stats.weak_password_count > stats.breach_count * 2 {
            recommendations.push("High ratio of weak passwords. Consider enforcing stronger password policies.".to_string());
        }
        
        if stats.anomalies_detected > 20 {
            recommendations.push("Frequent anomaly detection. Review monitoring thresholds and patterns.".to_string());
        }
        
        if matches!(stats.monitoring_level, MonitoringLevel::Critical) {
            recommendations.push("System is in critical monitoring mode. Review recent security events and consider additional security measures.".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Security monitoring is operating normally. Continue regular review cycles.".to_string());
        }
        
        recommendations
    }
}

/// Security monitoring report
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityReport {
    pub generated_at: DateTime<Utc>,
    pub monitoring_level: MonitoringLevel,
    pub total_events_monitored: u64,
    pub security_violations: u64,
    pub anomalies_detected: u64,
    pub threats_blocked: u64,
    pub event_type_distribution: HashMap<String, u32>,
    pub recent_anomalies: Vec<SecurityAnomaly>,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_monitor_creation() {
        let monitor = SecurityEventMonitor::new();
        let stats = monitor.get_statistics().await.unwrap();
        
        assert_eq!(stats.security_violations, 0);
        assert_eq!(stats.anomalies_detected, 0);
        assert_eq!(stats.monitoring_level, MonitoringLevel::Normal);
    }
    
    #[tokio::test]
    async fn test_process_event() {
        let monitor = SecurityEventMonitor::new();
        
        let event = crate::security::SecurityEvent::new(
            SecurityEventType::LoginSuccess,
            SecuritySeverity::Info,
            "User logged in".to_string(),
        );
        
        monitor.process_event(&event).await.unwrap();
        
        let buffer = monitor.event_buffer.read().await;
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer[0].event_type, SecurityEventType::LoginSuccess);
    }
    
    #[tokio::test]
    async fn test_brute_force_detection() {
        let monitor = SecurityEventMonitor::new();
        
        // Simulate brute force attack (6 failed logins in quick succession)
        for _ in 0..6 {
            let event = crate::security::SecurityEvent::new(
                SecurityEventType::LoginFailure,
                SecuritySeverity::Medium,
                "Login failed".to_string(),
            );
            monitor.process_event(&event).await.unwrap();
        }
        
        // Should detect brute force pattern
        let anomalies = monitor.get_recent_anomalies(None).await.unwrap();
        assert!(!anomalies.is_empty());
        assert_eq!(anomalies[0].pattern_id, "brute_force_login");
    }
    
    #[tokio::test]
    async fn test_monitoring_level_increase() {
        let mut monitor = SecurityEventMonitor::new();
        
        assert_eq!(monitor.config.monitoring_level, MonitoringLevel::Normal);
        
        monitor.increase_monitoring_level().await.unwrap();
        assert_eq!(monitor.config.monitoring_level, MonitoringLevel::High);
        
        monitor.increase_monitoring_level().await.unwrap();
        assert_eq!(monitor.config.monitoring_level, MonitoringLevel::Critical);
    }
    
    #[tokio::test]
    async fn test_risk_score_calculation() {
        let monitor = SecurityEventMonitor::new();
        
        let pattern = &monitor.config.threat_patterns[0]; // brute_force_login
        let events = vec![
            crate::security::SecurityEvent::new(
                SecurityEventType::LoginFailure,
                SecuritySeverity::Medium,
                "Login failed 1".to_string(),
            ),
            crate::security::SecurityEvent::new(
                SecurityEventType::LoginFailure,
                SecuritySeverity::Medium,
                "Login failed 2".to_string(),
            ),
        ];
        
        let risk_score = monitor.calculate_risk_score(pattern, &events);
        assert!(risk_score > 0.0 && risk_score <= 1.0);
    }
    
    #[tokio::test]
    async fn test_security_report_generation() {
        let monitor = SecurityEventMonitor::new();
        
        // Add some test events
        let event = crate::security::SecurityEvent::new(
            SecurityEventType::SecurityViolation,
            SecuritySeverity::High,
            "Test violation".to_string(),
        );
        monitor.process_event(&event).await.unwrap();
        
        let report = monitor.generate_security_report().await.unwrap();
        
        assert_eq!(report.total_events_monitored, 1);
        assert!(report.security_violations >= 1);
        assert!(!report.recommendations.is_empty());
    }
}