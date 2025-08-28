//! Zero Trust security architecture implementation
//!
//! Implements comprehensive zero-trust security model with continuous verification,
//! least-privilege access, and adaptive security policies.

use crate::security::{SecurityEvent, SecurityEventType, SecuritySeverity, SecurityContext, AuthenticationLevel};
use crate::{Result, TwoPasswordError};
use chrono::{DateTime, Duration, Utc, Timelike, Datelike};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

/// Trust levels for zero trust assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum TrustLevel {
    Untrusted,    // 0-20%: Unknown or suspicious
    Low,          // 21-40%: Basic verification
    Medium,       // 41-60%: Standard verification
    High,         // 61-80%: Strong verification
    Verified,     // 81-100%: Full verification
}

impl TrustLevel {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s <= 0.20 => TrustLevel::Untrusted,
            s if s <= 0.40 => TrustLevel::Low,
            s if s <= 0.60 => TrustLevel::Medium,
            s if s <= 0.80 => TrustLevel::High,
            _ => TrustLevel::Verified,
        }
    }
    
    pub fn to_score(&self) -> f64 {
        match self {
            TrustLevel::Untrusted => 0.10,
            TrustLevel::Low => 0.30,
            TrustLevel::Medium => 0.50,
            TrustLevel::High => 0.70,
            TrustLevel::Verified => 0.90,
        }
    }
}

/// Risk assessment factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactors {
    pub device_risk: f64,        // 0.0-1.0
    pub network_risk: f64,       // 0.0-1.0
    pub behavioral_risk: f64,    // 0.0-1.0
    pub temporal_risk: f64,      // 0.0-1.0
    pub geographical_risk: f64,  // 0.0-1.0
}

impl RiskFactors {
    pub fn calculate_overall_risk(&self) -> f64 {
        // Weighted risk calculation
        let weights = [0.25, 0.20, 0.25, 0.15, 0.15]; // Must sum to 1.0
        let factors = [
            self.device_risk,
            self.network_risk,
            self.behavioral_risk,
            self.temporal_risk,
            self.geographical_risk,
        ];
        
        weights.iter().zip(factors.iter()).map(|(w, f)| w * f).sum()
    }
}

/// Device trust information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceTrust {
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
    pub os_type: String,
    pub os_version: String,
    pub trust_level: TrustLevel,
    pub last_seen: DateTime<Utc>,
    pub first_seen: DateTime<Utc>,
    pub access_count: u64,
    pub security_violations: u32,
    pub is_managed: bool,
    pub encryption_status: bool,
    pub compliance_status: bool,
    pub risk_score: f64,
}

/// Network context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkContext {
    pub ip_address: String,
    pub network_type: NetworkType,
    pub location: Option<String>,
    pub country: Option<String>,
    pub isp: Option<String>,
    pub is_vpn: bool,
    pub is_tor: bool,
    pub is_known_malicious: bool,
    pub trust_level: TrustLevel,
    pub risk_score: f64,
}

/// Network type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkType {
    Corporate,    // Trusted corporate network
    Home,         // Home network
    Public,       // Public WiFi/network
    Mobile,       // Mobile carrier network
    Unknown,      // Unclassified network
}

/// Behavioral pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralProfile {
    pub user_id: String,
    pub typical_access_hours: Vec<u8>,  // Hours 0-23
    pub typical_days: Vec<u8>,          // Days 0-6 (Sun-Sat)
    pub common_locations: HashSet<String>,
    pub common_devices: HashSet<String>,
    pub access_patterns: HashMap<String, u32>, // Operation -> Count
    pub last_updated: DateTime<Utc>,
    pub confidence_level: f64,          // 0.0-1.0
}

/// Access policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub resource_types: Vec<String>,
    pub required_trust_level: TrustLevel,
    pub required_auth_level: AuthenticationLevel,
    pub max_session_duration: Duration,
    pub allowed_networks: Option<Vec<NetworkType>>,
    pub allowed_devices: Option<Vec<String>>,
    pub time_restrictions: Option<TimeRestrictions>,
    pub enabled: bool,
}

/// Time-based access restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestrictions {
    pub allowed_hours: Vec<u8>,    // Hours 0-23
    pub allowed_days: Vec<u8>,     // Days 0-6
    pub timezone: String,
}

/// Access decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessDecision {
    pub granted: bool,
    pub trust_score: f64,
    pub risk_score: f64,
    pub required_actions: Vec<String>,
    pub session_duration: Option<Duration>,
    pub monitoring_level: String,
    pub reason: String,
}

/// Zero trust configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroTrustConfig {
    pub enable_continuous_verification: bool,
    pub min_trust_level: TrustLevel,
    pub max_risk_threshold: f64,
    pub session_timeout_minutes: u32,
    pub require_device_compliance: bool,
    pub enable_behavioral_analysis: bool,
    pub enable_location_restrictions: bool,
    pub adaptive_policies_enabled: bool,
}

impl Default for ZeroTrustConfig {
    fn default() -> Self {
        Self {
            enable_continuous_verification: true,
            min_trust_level: TrustLevel::Medium,
            max_risk_threshold: 0.7,
            session_timeout_minutes: 60,
            require_device_compliance: true,
            enable_behavioral_analysis: true,
            enable_location_restrictions: false,
            adaptive_policies_enabled: true,
        }
    }
}

/// Zero Trust manager
pub struct ZeroTrustManager {
    config: ZeroTrustConfig,
    device_registry: HashMap<String, DeviceTrust>,
    behavioral_profiles: HashMap<String, BehavioralProfile>,
    access_policies: HashMap<String, AccessPolicy>,
    network_intelligence: HashMap<String, NetworkContext>,
    active_sessions: HashMap<String, ZeroTrustSession>,
}

/// Active zero trust session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroTrustSession {
    pub session_id: String,
    pub user_id: String,
    pub device_id: String,
    pub network_context: NetworkContext,
    pub initial_trust_score: f64,
    pub current_trust_score: f64,
    pub risk_score: f64,
    pub created_at: DateTime<Utc>,
    pub last_verification: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub access_count: u32,
    pub verification_failures: u32,
}

impl ZeroTrustManager {
    /// Create new zero trust manager
    pub fn new() -> Self {
        Self {
            config: ZeroTrustConfig::default(),
            device_registry: HashMap::new(),
            behavioral_profiles: HashMap::new(),
            access_policies: HashMap::new(),
            network_intelligence: HashMap::new(),
            active_sessions: HashMap::new(),
        }
    }
    
    /// Create manager with custom configuration
    pub fn with_config(config: ZeroTrustConfig) -> Self {
        let mut manager = Self::new();
        manager.config = config;
        manager.load_default_policies();
        manager
    }
    
    /// Load default access policies
    fn load_default_policies(&mut self) {
        // High-security resource policy
        let high_security_policy = AccessPolicy {
            id: "high_security".to_string(),
            name: "High Security Resources".to_string(),
            description: "Access to sensitive data and administrative functions".to_string(),
            resource_types: vec!["admin".to_string(), "financial".to_string(), "pii".to_string()],
            required_trust_level: TrustLevel::High,
            required_auth_level: AuthenticationLevel::MultiFactor,
            max_session_duration: Duration::minutes(30),
            allowed_networks: Some(vec![NetworkType::Corporate]),
            allowed_devices: None,
            time_restrictions: None,
            enabled: true,
        };
        
        // Standard resource policy
        let standard_policy = AccessPolicy {
            id: "standard_access".to_string(),
            name: "Standard Access".to_string(),
            description: "Normal application access and data operations".to_string(),
            resource_types: vec!["passwords".to_string(), "notes".to_string()],
            required_trust_level: TrustLevel::Medium,
            required_auth_level: AuthenticationLevel::Password,
            max_session_duration: Duration::hours(2),
            allowed_networks: None,
            allowed_devices: None,
            time_restrictions: None,
            enabled: true,
        };
        
        self.access_policies.insert("high_security".to_string(), high_security_policy);
        self.access_policies.insert("standard_access".to_string(), standard_policy);
    }
    
    /// Evaluate access request using zero trust principles
    pub async fn evaluate_access_request(
        &mut self,
        context: &SecurityContext,
        resource_type: &str,
    ) -> Result<AccessDecision> {
        // 1. Calculate trust score
        let trust_score = self.calculate_trust_score(context).await?;
        
        // 2. Assess risk factors
        let risk_factors = self.assess_risk_factors(context).await?;
        let risk_score = risk_factors.calculate_overall_risk();
        
        // 3. Find applicable policy
        let policy = self.find_applicable_policy(resource_type)?;
        
        // 4. Make access decision
        let mut decision = AccessDecision {
            granted: false,
            trust_score,
            risk_score,
            required_actions: Vec::new(),
            session_duration: None,
            monitoring_level: "standard".to_string(),
            reason: String::new(),
        };
        
        // Check minimum trust level
        if TrustLevel::from_score(trust_score) < policy.required_trust_level {
            decision.reason = format!(
                "Trust level {} below required {}",
                format!("{:?}", TrustLevel::from_score(trust_score)),
                format!("{:?}", policy.required_trust_level)
            );
            decision.required_actions.push("Increase authentication strength".to_string());
            return Ok(decision);
        }
        
        // Check risk threshold
        if risk_score > self.config.max_risk_threshold {
            decision.reason = format!("Risk score {:.2} exceeds threshold {:.2}", risk_score, self.config.max_risk_threshold);
            decision.required_actions.push("Additional verification required".to_string());
            return Ok(decision);
        }
        
        // Check authentication level
        if context.authentication_level < policy.required_auth_level {
            decision.reason = format!(
                "Authentication level {:?} below required {:?}",
                context.authentication_level,
                policy.required_auth_level
            );
            decision.required_actions.push("Upgrade authentication method".to_string());
            return Ok(decision);
        }
        
        // Check network restrictions
        if let Some(allowed_networks) = &policy.allowed_networks {
            let network_context = self.get_network_context(&context.source_ip).await?;
            if !allowed_networks.contains(&network_context.network_type) {
                decision.reason = "Access from unauthorized network".to_string();
                decision.required_actions.push("Connect from approved network".to_string());
                return Ok(decision);
            }
        }
        
        // Check time restrictions
        if let Some(time_restrictions) = &policy.time_restrictions {
            if !self.check_time_restrictions(time_restrictions) {
                decision.reason = "Access outside allowed time window".to_string();
                return Ok(decision);
            }
        }
        
        // Grant access
        decision.granted = true;
        decision.session_duration = Some(policy.max_session_duration);
        decision.monitoring_level = if risk_score > 0.5 { "enhanced".to_string() } else { "standard".to_string() };
        decision.reason = "Access granted based on zero trust evaluation".to_string();
        
        // Add conditional requirements
        if trust_score < 0.8 {
            decision.required_actions.push("Continuous verification enabled".to_string());
        }
        
        if risk_score > 0.4 {
            decision.required_actions.push("Enhanced monitoring activated".to_string());
        }
        
        Ok(decision)
    }
    
    /// Calculate comprehensive trust score
    async fn calculate_trust_score(&mut self, context: &SecurityContext) -> Result<f64> {
        let mut trust_components = Vec::new();
        
        // Device trust (40%)
        let device_trust = self.get_device_trust(&context.device_id).await?;
        trust_components.push((device_trust.trust_level.to_score(), 0.40));
        
        // Authentication strength (30%)
        let auth_score = match context.authentication_level {
            AuthenticationLevel::None => 0.0,
            AuthenticationLevel::Password => 0.3,
            AuthenticationLevel::Biometric => 0.6,
            AuthenticationLevel::MultiFactor => 0.8,
            AuthenticationLevel::HardwareKey => 1.0,
        };
        trust_components.push((auth_score, 0.30));
        
        // Behavioral consistency (20%)
        let behavioral_score = if let Some(user_id) = &context.user_id {
            self.calculate_behavioral_score(user_id, context).await?
        } else {
            0.5 // Default for unauthenticated
        };
        trust_components.push((behavioral_score, 0.20));
        
        // Network trust (10%)
        let network_score = if let Some(ip) = &context.source_ip {
            let network_context = self.get_network_context(&Some(ip.clone())).await?;
            network_context.trust_level.to_score()
        } else {
            0.5
        };
        trust_components.push((network_score, 0.10));
        
        // Calculate weighted average
        let total_score = trust_components.iter()
            .map(|(score, weight)| score * weight)
            .sum();
        
        Ok(total_score)
    }
    
    /// Assess comprehensive risk factors
    async fn assess_risk_factors(&mut self, context: &SecurityContext) -> Result<RiskFactors> {
        let device_risk = self.calculate_device_risk(&context.device_id).await?;
        let network_risk = self.calculate_network_risk(&context.source_ip).await?;
        let behavioral_risk = if let Some(user_id) = &context.user_id {
            self.calculate_behavioral_risk(user_id, context).await?
        } else {
            0.7 // Higher risk for unauthenticated
        };
        let temporal_risk = self.calculate_temporal_risk().await?;
        let geographical_risk = self.calculate_geographical_risk(&context.source_ip).await?;
        
        Ok(RiskFactors {
            device_risk,
            network_risk,
            behavioral_risk,
            temporal_risk,
            geographical_risk,
        })
    }
    
    /// Get or create device trust information
    async fn get_device_trust(&mut self, device_id: &str) -> Result<DeviceTrust> {
        if let Some(device_trust) = self.device_registry.get(device_id) {
            Ok(device_trust.clone())
        } else {
            // Create new device entry with low initial trust
            let device_trust = DeviceTrust {
                device_id: device_id.to_string(),
                device_name: "Unknown Device".to_string(),
                device_type: "Unknown".to_string(),
                os_type: "Unknown".to_string(),
                os_version: "Unknown".to_string(),
                trust_level: TrustLevel::Low,
                last_seen: Utc::now(),
                first_seen: Utc::now(),
                access_count: 0,
                security_violations: 0,
                is_managed: false,
                encryption_status: false,
                compliance_status: false,
                risk_score: 0.7,
            };
            
            self.device_registry.insert(device_id.to_string(), device_trust.clone());
            Ok(device_trust)
        }
    }
    
    /// Calculate device-specific risk
    async fn calculate_device_risk(&mut self, device_id: &str) -> Result<f64> {
        let device_trust = self.get_device_trust(device_id).await?;
        
        let mut risk_score = 0.0;
        
        // Base risk from violations
        risk_score += (device_trust.security_violations as f64) * 0.1;
        
        // Risk from non-compliance
        if !device_trust.compliance_status {
            risk_score += 0.3;
        }
        
        // Risk from unmanaged device
        if !device_trust.is_managed {
            risk_score += 0.2;
        }
        
        // Risk from no encryption
        if !device_trust.encryption_status {
            risk_score += 0.2;
        }
        
        // Risk from being new device
        let days_known = (Utc::now() - device_trust.first_seen).num_days();
        if days_known < 7 {
            risk_score += 0.3;
        }
        
        Ok(risk_score.min(1.0))
    }
    
    /// Calculate network-specific risk
    async fn calculate_network_risk(&mut self, source_ip: &Option<String>) -> Result<f64> {
        if let Some(ip) = source_ip {
            let network_context = self.get_network_context(&Some(ip.clone())).await?;
            Ok(network_context.risk_score)
        } else {
            Ok(0.8) // High risk for unknown IP
        }
    }
    
    /// Get network context information
    async fn get_network_context(&mut self, source_ip: &Option<String>) -> Result<NetworkContext> {
        let ip_str = source_ip.as_deref().unwrap_or("0.0.0.0");
        
        if let Some(context) = self.network_intelligence.get(ip_str) {
            Ok(context.clone())
        } else {
            // Create basic network context
            let network_context = NetworkContext {
                ip_address: ip_str.to_string(),
                network_type: NetworkType::Unknown,
                location: None,
                country: None,
                isp: None,
                is_vpn: false,
                is_tor: false,
                is_known_malicious: false,
                trust_level: TrustLevel::Low,
                risk_score: 0.6, // Medium risk for unknown networks
            };
            
            self.network_intelligence.insert(ip_str.to_string(), network_context.clone());
            Ok(network_context)
        }
    }
    
    /// Calculate behavioral consistency score
    async fn calculate_behavioral_score(&mut self, user_id: &str, context: &SecurityContext) -> Result<f64> {
        // Clone the profile to avoid borrowing issues
        let profile_clone = self.behavioral_profiles.get(user_id).cloned();
        
        if let Some(profile) = profile_clone {
            let mut score = 1.0;
            
            // Check time patterns
            let current_hour = Utc::now().hour() as u8;
            if !profile.typical_access_hours.contains(&current_hour) {
                score *= 0.7;
            }
            
            // Check device patterns
            if !profile.common_devices.contains(&context.device_id) {
                score *= 0.8;
            }
            
            // Check location patterns (if available)
            if let Some(ip) = &context.source_ip {
                if let Ok(network_context) = self.get_network_context(&Some(ip.clone())).await {
                    if let Some(location) = &network_context.location {
                        if !profile.common_locations.contains(location) {
                            score *= 0.6;
                        }
                    }
                }
            }
            
            Ok(score)
        } else {
            Ok(0.5) // Neutral score for new users
        }
    }
    
    /// Calculate behavioral risk
    async fn calculate_behavioral_risk(&mut self, user_id: &str, context: &SecurityContext) -> Result<f64> {
        let behavioral_score = self.calculate_behavioral_score(user_id, context).await?;
        Ok(1.0 - behavioral_score) // Higher behavioral inconsistency = higher risk
    }
    
    /// Calculate temporal risk
    async fn calculate_temporal_risk(&self) -> Result<f64> {
        let now = Utc::now();
        let hour = now.hour();
        
        // Higher risk during unusual hours
        match hour {
            22..=23 | 0..=5 => Ok(0.7), // Late night/early morning
            6..=8 | 18..=21 => Ok(0.3), // Early morning/evening
            _ => Ok(0.1), // Business hours
        }
    }
    
    /// Calculate geographical risk
    async fn calculate_geographical_risk(&mut self, source_ip: &Option<String>) -> Result<f64> {
        if let Some(ip) = source_ip {
            let network_context = self.get_network_context(&Some(ip.clone())).await?;
            
            let mut risk = 0.0;
            
            if network_context.is_vpn {
                risk += 0.3;
            }
            
            if network_context.is_tor {
                risk += 0.8;
            }
            
            if network_context.is_known_malicious {
                risk = 1.0;
            }
            
            Ok(risk)
        } else {
            Ok(0.5)
        }
    }
    
    /// Find applicable access policy
    fn find_applicable_policy(&self, resource_type: &str) -> Result<AccessPolicy> {
        for policy in self.access_policies.values() {
            if policy.enabled && policy.resource_types.contains(&resource_type.to_string()) {
                return Ok(policy.clone());
            }
        }
        
        // Return default policy if no specific policy found
        Ok(AccessPolicy {
            id: "default".to_string(),
            name: "Default Policy".to_string(),
            description: "Default access policy".to_string(),
            resource_types: vec!["*".to_string()],
            required_trust_level: self.config.min_trust_level.clone(),
            required_auth_level: AuthenticationLevel::Password,
            max_session_duration: Duration::minutes(self.config.session_timeout_minutes as i64),
            allowed_networks: None,
            allowed_devices: None,
            time_restrictions: None,
            enabled: true,
        })
    }
    
    /// Check time-based access restrictions
    fn check_time_restrictions(&self, restrictions: &TimeRestrictions) -> bool {
        let now = Utc::now();
        let current_hour = now.hour() as u8;
        let current_day = now.weekday().num_days_from_sunday() as u8;
        
        restrictions.allowed_hours.contains(&current_hour) &&
        restrictions.allowed_days.contains(&current_day)
    }
    
    /// Create zero trust session
    pub async fn create_session(
        &mut self,
        session_id: String,
        context: &SecurityContext,
        trust_score: f64,
        risk_score: f64,
    ) -> Result<()> {
        let network_context = self.get_network_context(&context.source_ip).await?;
        
        let session = ZeroTrustSession {
            session_id: session_id.clone(),
            user_id: context.user_id.clone().unwrap_or_default(),
            device_id: context.device_id.clone(),
            network_context,
            initial_trust_score: trust_score,
            current_trust_score: trust_score,
            risk_score,
            created_at: Utc::now(),
            last_verification: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(self.config.session_timeout_minutes as i64),
            access_count: 0,
            verification_failures: 0,
        };
        
        self.active_sessions.insert(session_id, session);
        Ok(())
    }
    
    /// Continuously verify active session
    pub async fn verify_session(&mut self, session_id: &str, context: &SecurityContext) -> Result<bool> {
        // First check if session exists and get necessary values
        let (initial_trust_score, expires_at) = if let Some(session) = self.active_sessions.get(session_id) {
            (session.initial_trust_score, session.expires_at)
        } else {
            return Ok(false);
        };
        
        // Check if session expired
        if Utc::now() > expires_at {
            self.active_sessions.remove(session_id);
            return Ok(false);
        }
        
        // Recalculate trust score (this can borrow self mutably)
        let current_trust = self.calculate_trust_score(context).await?;
        let trust_degradation = initial_trust_score - current_trust;
        
        // Now update session with mutable access
        if let Some(session) = self.active_sessions.get_mut(session_id) {
            // Check for significant trust degradation
            if trust_degradation > 0.3 {
                session.verification_failures += 1;
                
                if session.verification_failures >= 3 {
                    self.active_sessions.remove(session_id);
                    return Ok(false);
                }
            }
            
            // Update session
            session.current_trust_score = current_trust;
            session.last_verification = Utc::now();
            session.access_count += 1;
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Update device trust based on behavior
    pub async fn update_device_trust(&mut self, device_id: &str, positive_behavior: bool) -> Result<()> {
        if let Some(device_trust) = self.device_registry.get_mut(device_id) {
            device_trust.last_seen = Utc::now();
            device_trust.access_count += 1;
            
            if positive_behavior {
                // Gradually increase trust - use larger increment to cross thresholds
                let current_score = device_trust.trust_level.to_score();
                let new_score = (current_score + 0.15).min(1.0);
                device_trust.trust_level = TrustLevel::from_score(new_score);
                device_trust.risk_score = (device_trust.risk_score - 0.02).max(0.0);
            } else {
                device_trust.security_violations += 1;
                device_trust.trust_level = TrustLevel::Untrusted;
                device_trust.risk_score = (device_trust.risk_score + 0.1).min(1.0);
            }
        }
        Ok(())
    }
    
    /// Generate zero trust security report
    pub async fn generate_zero_trust_report(&self) -> Result<ZeroTrustReport> {
        let total_devices = self.device_registry.len() as u32;
        let trusted_devices = self.device_registry.values()
            .filter(|d| d.trust_level >= TrustLevel::High)
            .count() as u32;
        
        let active_sessions = self.active_sessions.len() as u32;
        let high_risk_sessions = self.active_sessions.values()
            .filter(|s| s.risk_score > 0.7)
            .count() as u32;
        
        let avg_trust_score = if !self.active_sessions.is_empty() {
            self.active_sessions.values()
                .map(|s| s.current_trust_score)
                .sum::<f64>() / self.active_sessions.len() as f64
        } else {
            0.0
        };
        
        Ok(ZeroTrustReport {
            generated_at: Utc::now(),
            total_devices,
            trusted_devices,
            compliance_percentage: if total_devices > 0 {
                (trusted_devices as f64 / total_devices as f64) * 100.0
            } else {
                0.0
            },
            active_sessions,
            high_risk_sessions,
            average_trust_score: avg_trust_score,
            policy_count: self.access_policies.len() as u32,
            configuration: self.config.clone(),
        })
    }
}

/// Zero trust security report
#[derive(Debug, Serialize, Deserialize)]
pub struct ZeroTrustReport {
    pub generated_at: DateTime<Utc>,
    pub total_devices: u32,
    pub trusted_devices: u32,
    pub compliance_percentage: f64,
    pub active_sessions: u32,
    pub high_risk_sessions: u32,
    pub average_trust_score: f64,
    pub policy_count: u32,
    pub configuration: ZeroTrustConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_zero_trust_manager_creation() {
        let manager = ZeroTrustManager::new();
        assert_eq!(manager.device_registry.len(), 0);
        assert_eq!(manager.active_sessions.len(), 0);
    }
    
    #[tokio::test]
    async fn test_trust_level_conversion() {
        assert_eq!(TrustLevel::from_score(0.1), TrustLevel::Untrusted);
        assert_eq!(TrustLevel::from_score(0.3), TrustLevel::Low);
        assert_eq!(TrustLevel::from_score(0.5), TrustLevel::Medium);
        assert_eq!(TrustLevel::from_score(0.7), TrustLevel::High);
        assert_eq!(TrustLevel::from_score(0.9), TrustLevel::Verified);
    }
    
    #[tokio::test]
    async fn test_risk_factors_calculation() {
        let risk_factors = RiskFactors {
            device_risk: 0.2,
            network_risk: 0.3,
            behavioral_risk: 0.1,
            temporal_risk: 0.4,
            geographical_risk: 0.5,
        };
        
        let overall_risk = risk_factors.calculate_overall_risk();
        assert!(overall_risk > 0.0 && overall_risk < 1.0);
    }
    
    #[tokio::test]
    async fn test_access_evaluation() {
        let mut manager = ZeroTrustManager::new();
        
        let context = SecurityContext {
            user_id: Some("test_user".to_string()),
            session_id: Some("test_session".to_string()),
            device_id: "test_device".to_string(),
            source_ip: Some("192.168.1.100".to_string()),
            user_agent: Some("TestAgent/1.0".to_string()),
            permissions: vec!["read".to_string()],
            authentication_level: AuthenticationLevel::Password,
        };
        
        let decision = manager.evaluate_access_request(&context, "passwords").await.unwrap();
        
        assert!(decision.trust_score >= 0.0 && decision.trust_score <= 1.0);
        assert!(decision.risk_score >= 0.0 && decision.risk_score <= 1.0);
        assert!(!decision.reason.is_empty());
    }
    
    #[tokio::test]
    async fn test_device_trust_update() {
        let mut manager = ZeroTrustManager::new();
        let device_id = "test_device";
        
        // Get initial device trust (should create new entry)
        let initial_trust = manager.get_device_trust(device_id).await.unwrap();
        assert_eq!(initial_trust.trust_level, TrustLevel::Low);
        
        // Update with positive behavior - one update should cross threshold  
        // Low = 0.3 + 0.15 = 0.45, should become Medium
        manager.update_device_trust(device_id, true).await.unwrap();
        
        let final_trust = manager.get_device_trust(device_id).await.unwrap();
        
        // After 1 update: 0.3 + 0.15 = 0.45, should be Medium
        assert_eq!(final_trust.trust_level, TrustLevel::Medium);
        assert!(final_trust.trust_level.to_score() > initial_trust.trust_level.to_score());
    }
    
    #[tokio::test]
    async fn test_session_management() {
        let mut manager = ZeroTrustManager::new();
        
        let context = SecurityContext {
            user_id: Some("test_user".to_string()),
            session_id: Some("test_session".to_string()),
            device_id: "test_device".to_string(),
            source_ip: Some("192.168.1.100".to_string()),
            user_agent: None,
            permissions: vec![],
            authentication_level: AuthenticationLevel::Password,
        };
        
        // Create session
        manager.create_session("session123".to_string(), &context, 0.8, 0.2).await.unwrap();
        assert_eq!(manager.active_sessions.len(), 1);
        
        // Verify session
        let is_valid = manager.verify_session("session123", &context).await.unwrap();
        assert!(is_valid);
    }
}