//! Compliance framework and regulatory standards support
//!
//! Implements compliance monitoring for various standards including
//! SOC2, HIPAA, GDPR, ISO 27001, and other security frameworks.

use crate::security::{SecurityEvent, SecurityEventType, SecuritySeverity};
use crate::{Result, TwoPasswordError};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported compliance standards
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ComplianceStandard {
    SOC2,       // System and Organization Controls 2
    HIPAA,      // Health Insurance Portability and Accountability Act
    GDPR,       // General Data Protection Regulation
    PCI_DSS,    // Payment Card Industry Data Security Standard
    ISO27001,   // ISO/IEC 27001 Information Security Management
    NIST,       // NIST Cybersecurity Framework
    FISMA,      // Federal Information Security Management Act
    FedRAMP,    // Federal Risk and Authorization Management Program
}

impl ComplianceStandard {
    pub fn name(&self) -> &'static str {
        match self {
            ComplianceStandard::SOC2 => "SOC 2",
            ComplianceStandard::HIPAA => "HIPAA",
            ComplianceStandard::GDPR => "GDPR",
            ComplianceStandard::PCI_DSS => "PCI DSS",
            ComplianceStandard::ISO27001 => "ISO 27001",
            ComplianceStandard::NIST => "NIST Framework",
            ComplianceStandard::FISMA => "FISMA",
            ComplianceStandard::FedRAMP => "FedRAMP",
        }
    }
}

/// Compliance requirement definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    pub id: String,
    pub standard: ComplianceStandard,
    pub category: String,
    pub title: String,
    pub description: String,
    pub control_id: String,
    pub severity: ComplianceSeverity,
    pub required_evidence: Vec<String>,
    pub automated_check: Option<String>,
    pub remediation_guidance: String,
}

/// Compliance severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComplianceSeverity {
    Critical,   // Must be addressed immediately
    High,       // Important compliance gap
    Medium,     // Should be addressed
    Low,        // Best practice recommendation
}

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckResult {
    pub requirement_id: String,
    pub status: ComplianceStatus,
    pub score: f64,           // 0.0-1.0
    pub evidence: Vec<String>,
    pub gaps: Vec<String>,
    pub recommendations: Vec<String>,
    pub last_checked: DateTime<Utc>,
    pub next_check_due: DateTime<Utc>,
}

/// Compliance status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ComplianceStatus {
    Compliant,      // Fully meets requirements
    PartiallyCompliant, // Partially meets requirements
    NonCompliant,   // Does not meet requirements
    NotApplicable,  // Requirement does not apply
    PendingReview,  // Awaiting manual review
}

/// Compliance assessment report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAssessment {
    pub id: String,
    pub standard: ComplianceStandard,
    pub assessment_date: DateTime<Utc>,
    pub overall_score: f64,
    pub status: ComplianceStatus,
    pub total_requirements: u32,
    pub compliant_requirements: u32,
    pub non_compliant_requirements: u32,
    pub critical_gaps: Vec<String>,
    pub recommendations: Vec<String>,
    pub evidence_collected: Vec<String>,
    pub next_assessment_due: DateTime<Utc>,
}

/// Data classification for compliance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataClassification {
    Public,         // No restrictions
    Internal,       // Internal use only
    Confidential,   // Restricted access
    Restricted,     // Highly restricted
    TopSecret,      // Maximum security
}

/// Data retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRetentionPolicy {
    pub data_type: String,
    pub classification: DataClassification,
    pub retention_period_days: u32,
    pub deletion_method: String,
    pub backup_retention_days: u32,
    pub legal_hold_exempt: bool,
}

/// Compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    pub enabled_standards: Vec<ComplianceStandard>,
    pub assessment_frequency_days: u32,
    pub evidence_retention_days: u32,
    pub automatic_remediation_enabled: bool,
    pub notification_enabled: bool,
    pub external_audit_mode: bool,
    pub data_classification_enabled: bool,
    pub retention_policies: Vec<DataRetentionPolicy>,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            enabled_standards: vec![ComplianceStandard::SOC2, ComplianceStandard::GDPR],
            assessment_frequency_days: 90,
            evidence_retention_days: 2555, // ~7 years
            automatic_remediation_enabled: false,
            notification_enabled: true,
            external_audit_mode: false,
            data_classification_enabled: true,
            retention_policies: Vec::new(),
        }
    }
}

/// Compliance manager
pub struct ComplianceManager {
    config: ComplianceConfig,
    requirements: HashMap<String, ComplianceRequirement>,
    assessment_results: HashMap<String, ComplianceCheckResult>,
    assessments: Vec<ComplianceAssessment>,
    evidence_store: HashMap<String, Vec<ComplianceEvidence>>,
}

/// Compliance evidence record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidence {
    pub id: String,
    pub requirement_id: String,
    pub evidence_type: String,
    pub description: String,
    pub collected_at: DateTime<Utc>,
    pub collected_by: String,
    pub file_path: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl ComplianceManager {
    /// Create new compliance manager
    pub fn new() -> Self {
        let mut manager = Self {
            config: ComplianceConfig::default(),
            requirements: HashMap::new(),
            assessment_results: HashMap::new(),
            assessments: Vec::new(),
            evidence_store: HashMap::new(),
        };
        
        manager.load_compliance_requirements();
        manager
    }
    
    /// Create manager with custom configuration
    pub fn with_config(config: ComplianceConfig) -> Self {
        let mut manager = Self::new();
        manager.config = config;
        manager.load_compliance_requirements();
        manager
    }
    
    /// Load compliance requirements for enabled standards
    fn load_compliance_requirements(&mut self) {
        let standards = self.config.enabled_standards.clone();
        for standard in &standards {
            match standard {
                ComplianceStandard::SOC2 => self.load_soc2_requirements(),
                ComplianceStandard::GDPR => self.load_gdpr_requirements(),
                ComplianceStandard::HIPAA => self.load_hipaa_requirements(),
                ComplianceStandard::ISO27001 => self.load_iso27001_requirements(),
                ComplianceStandard::PCI_DSS => self.load_pci_dss_requirements(),
                ComplianceStandard::NIST => self.load_nist_requirements(),
                _ => {} // Other standards can be added as needed
            }
        }
    }
    
    /// Load SOC 2 compliance requirements
    fn load_soc2_requirements(&mut self) {
        let requirements = vec![
            ComplianceRequirement {
                id: "soc2_cc6_1".to_string(),
                standard: ComplianceStandard::SOC2,
                category: "Logical and Physical Access Controls".to_string(),
                title: "Access Control Management".to_string(),
                description: "The entity implements logical access security software, infrastructure, and architectures over protected information assets to protect them from security events to meet the entity's objectives.".to_string(),
                control_id: "CC6.1".to_string(),
                severity: ComplianceSeverity::Critical,
                required_evidence: vec![
                    "Access control policies".to_string(),
                    "User access reviews".to_string(),
                    "Authentication logs".to_string(),
                ],
                automated_check: Some("check_access_controls".to_string()),
                remediation_guidance: "Implement comprehensive access control policies and regular access reviews.".to_string(),
            },
            ComplianceRequirement {
                id: "soc2_cc6_7".to_string(),
                standard: ComplianceStandard::SOC2,
                category: "Logical and Physical Access Controls".to_string(),
                title: "Data Transmission and Disposal".to_string(),
                description: "The entity restricts the transmission, movement, and removal of information to authorized internal and external users and processes, and protects it during transmission, movement, or removal to meet the entity's objectives.".to_string(),
                control_id: "CC6.7".to_string(),
                severity: ComplianceSeverity::High,
                required_evidence: vec![
                    "Data encryption policies".to_string(),
                    "Transmission logs".to_string(),
                    "Data disposal procedures".to_string(),
                ],
                automated_check: Some("check_data_protection".to_string()),
                remediation_guidance: "Implement encryption for data in transit and at rest, establish secure data disposal procedures.".to_string(),
            },
            ComplianceRequirement {
                id: "soc2_cc7_2".to_string(),
                standard: ComplianceStandard::SOC2,
                category: "System Operations".to_string(),
                title: "System Monitoring".to_string(),
                description: "The entity monitors system components and the operation of controls to detect anomalies that are indicative of malicious acts, natural disasters, and errors affecting the entity's ability to meet its objectives.".to_string(),
                control_id: "CC7.2".to_string(),
                severity: ComplianceSeverity::High,
                required_evidence: vec![
                    "Security monitoring logs".to_string(),
                    "Incident response procedures".to_string(),
                    "Anomaly detection reports".to_string(),
                ],
                automated_check: Some("check_monitoring_systems".to_string()),
                remediation_guidance: "Implement comprehensive system monitoring and anomaly detection capabilities.".to_string(),
            },
        ];
        
        for req in requirements {
            self.requirements.insert(req.id.clone(), req);
        }
    }
    
    /// Load GDPR compliance requirements
    fn load_gdpr_requirements(&mut self) {
        let requirements = vec![
            ComplianceRequirement {
                id: "gdpr_art32".to_string(),
                standard: ComplianceStandard::GDPR,
                category: "Security of Processing".to_string(),
                title: "Security of Processing".to_string(),
                description: "The controller and the processor shall implement appropriate technical and organisational measures to ensure a level of security appropriate to the risk.".to_string(),
                control_id: "Article 32".to_string(),
                severity: ComplianceSeverity::Critical,
                required_evidence: vec![
                    "Data encryption implementation".to_string(),
                    "Access control measures".to_string(),
                    "Security incident procedures".to_string(),
                ],
                automated_check: Some("check_data_security".to_string()),
                remediation_guidance: "Implement pseudonymisation, encryption, confidentiality measures, and regular security testing.".to_string(),
            },
            ComplianceRequirement {
                id: "gdpr_art33".to_string(),
                standard: ComplianceStandard::GDPR,
                category: "Notification".to_string(),
                title: "Notification of Data Breach".to_string(),
                description: "In the case of a personal data breach, the controller shall without undue delay and, where feasible, not later than 72 hours after having become aware of it, notify the personal data breach to the supervisory authority.".to_string(),
                control_id: "Article 33".to_string(),
                severity: ComplianceSeverity::Critical,
                required_evidence: vec![
                    "Breach notification procedures".to_string(),
                    "Incident response plan".to_string(),
                    "Breach notification logs".to_string(),
                ],
                automated_check: Some("check_breach_procedures".to_string()),
                remediation_guidance: "Establish documented breach notification procedures with timeline requirements.".to_string(),
            },
            ComplianceRequirement {
                id: "gdpr_art17".to_string(),
                standard: ComplianceStandard::GDPR,
                category: "Data Subject Rights".to_string(),
                title: "Right to Erasure (Right to be Forgotten)".to_string(),
                description: "The data subject shall have the right to obtain from the controller the erasure of personal data concerning him or her without undue delay.".to_string(),
                control_id: "Article 17".to_string(),
                severity: ComplianceSeverity::High,
                required_evidence: vec![
                    "Data deletion procedures".to_string(),
                    "User request handling logs".to_string(),
                    "Data retention policies".to_string(),
                ],
                automated_check: Some("check_data_deletion".to_string()),
                remediation_guidance: "Implement automated data deletion capabilities and request handling procedures.".to_string(),
            },
        ];
        
        for req in requirements {
            self.requirements.insert(req.id.clone(), req);
        }
    }
    
    /// Load HIPAA compliance requirements
    fn load_hipaa_requirements(&mut self) {
        let requirements = vec![
            ComplianceRequirement {
                id: "hipaa_164_312_a_1".to_string(),
                standard: ComplianceStandard::HIPAA,
                category: "Administrative Safeguards".to_string(),
                title: "Access Control".to_string(),
                description: "A covered entity or business associate must assign a unique name and/or number for identifying and tracking user identity.".to_string(),
                control_id: "§ 164.312(a)(1)".to_string(),
                severity: ComplianceSeverity::Critical,
                required_evidence: vec![
                    "User identification procedures".to_string(),
                    "Access control documentation".to_string(),
                    "User activity logs".to_string(),
                ],
                automated_check: Some("check_user_identification".to_string()),
                remediation_guidance: "Implement unique user identification and comprehensive access controls for PHI.".to_string(),
            },
            ComplianceRequirement {
                id: "hipaa_164_312_e_1".to_string(),
                standard: ComplianceStandard::HIPAA,
                category: "Technical Safeguards".to_string(),
                title: "Transmission Security".to_string(),
                description: "Implement technical security measures to guard against unauthorized access to electronic protected health information that is being transmitted over an electronic communications network.".to_string(),
                control_id: "§ 164.312(e)(1)".to_string(),
                severity: ComplianceSeverity::Critical,
                required_evidence: vec![
                    "Transmission encryption".to_string(),
                    "Network security measures".to_string(),
                    "Communication logs".to_string(),
                ],
                automated_check: Some("check_transmission_security".to_string()),
                remediation_guidance: "Implement end-to-end encryption for all PHI transmissions.".to_string(),
            },
        ];
        
        for req in requirements {
            self.requirements.insert(req.id.clone(), req);
        }
    }
    
    /// Load ISO 27001 compliance requirements
    fn load_iso27001_requirements(&mut self) {
        let requirements = vec![
            ComplianceRequirement {
                id: "iso27001_a9_1_1".to_string(),
                standard: ComplianceStandard::ISO27001,
                category: "Access Control Policy".to_string(),
                title: "Access Control Policy".to_string(),
                description: "An access control policy shall be established, documented and reviewed based on business and information security requirements.".to_string(),
                control_id: "A.9.1.1".to_string(),
                severity: ComplianceSeverity::High,
                required_evidence: vec![
                    "Access control policy document".to_string(),
                    "Policy review records".to_string(),
                    "Implementation evidence".to_string(),
                ],
                automated_check: Some("check_access_policy".to_string()),
                remediation_guidance: "Document comprehensive access control policy with regular review cycles.".to_string(),
            },
        ];
        
        for req in requirements {
            self.requirements.insert(req.id.clone(), req);
        }
    }
    
    /// Load PCI DSS compliance requirements
    fn load_pci_dss_requirements(&mut self) {
        let requirements = vec![
            ComplianceRequirement {
                id: "pci_req_3_4".to_string(),
                standard: ComplianceStandard::PCI_DSS,
                category: "Protect Stored Cardholder Data".to_string(),
                title: "Render PAN Unreadable".to_string(),
                description: "Render PAN unreadable anywhere it is stored by using strong cryptography and security protocols.".to_string(),
                control_id: "Requirement 3.4".to_string(),
                severity: ComplianceSeverity::Critical,
                required_evidence: vec![
                    "Encryption implementation".to_string(),
                    "Key management procedures".to_string(),
                    "Cryptographic standards documentation".to_string(),
                ],
                automated_check: Some("check_data_encryption".to_string()),
                remediation_guidance: "Implement strong encryption for all stored payment card data.".to_string(),
            },
        ];
        
        for req in requirements {
            self.requirements.insert(req.id.clone(), req);
        }
    }
    
    /// Load NIST Framework requirements
    fn load_nist_requirements(&mut self) {
        let requirements = vec![
            ComplianceRequirement {
                id: "nist_pr_ac_1".to_string(),
                standard: ComplianceStandard::NIST,
                category: "Protect - Access Control".to_string(),
                title: "Identity and Access Management".to_string(),
                description: "Identities and credentials are issued, managed, verified, revoked, and audited for authorized devices, users and processes.".to_string(),
                control_id: "PR.AC-1".to_string(),
                severity: ComplianceSeverity::High,
                required_evidence: vec![
                    "Identity management procedures".to_string(),
                    "Credential management logs".to_string(),
                    "Access audit reports".to_string(),
                ],
                automated_check: Some("check_identity_management".to_string()),
                remediation_guidance: "Implement comprehensive identity and access management system.".to_string(),
            },
        ];
        
        for req in requirements {
            self.requirements.insert(req.id.clone(), req);
        }
    }
    
    /// Run compliance assessment
    pub async fn run_assessment(&mut self, standard: ComplianceStandard) -> Result<ComplianceAssessment> {
        let assessment_id = format!("{}_{}", standard.name().replace(" ", "_").to_lowercase(), Utc::now().timestamp());
        let mut total_score = 0.0;
        let mut total_requirements = 0u32;
        let mut compliant_requirements = 0u32;
        let mut non_compliant_requirements = 0u32;
        let mut critical_gaps = Vec::new();
        let mut recommendations = Vec::new();
        let mut evidence_collected = Vec::new();
        
        // Collect requirements for this standard first to avoid borrowing issues
        let relevant_requirements: Vec<(String, ComplianceRequirement)> = self.requirements.iter()
            .filter(|(_, requirement)| requirement.standard == standard)
            .map(|(id, req)| (id.clone(), req.clone()))
            .collect();
            
        // Check each requirement for this standard
        for (req_id, requirement) in relevant_requirements {
            total_requirements += 1;
            
            // Run compliance check
            let check_result = self.check_compliance_requirement(&req_id).await?;
            self.assessment_results.insert(req_id.clone(), check_result.clone());
            
            total_score += check_result.score;
            
            match check_result.status {
                ComplianceStatus::Compliant => compliant_requirements += 1,
                ComplianceStatus::NonCompliant => {
                    non_compliant_requirements += 1;
                    if requirement.severity == ComplianceSeverity::Critical {
                        critical_gaps.push(format!("{}: {}", requirement.control_id, requirement.title));
                    }
                }
                ComplianceStatus::PartiallyCompliant => {
                    // Count as non-compliant for metrics but with lower severity
                    non_compliant_requirements += 1;
                }
                _ => {}
            }
            
            recommendations.extend(check_result.recommendations);
            evidence_collected.extend(check_result.evidence);
        }
        
        let overall_score = if total_requirements > 0 {
            total_score / total_requirements as f64
        } else {
            0.0
        };
        
        let overall_status = if overall_score >= 0.95 {
            ComplianceStatus::Compliant
        } else if overall_score >= 0.70 {
            ComplianceStatus::PartiallyCompliant
        } else {
            ComplianceStatus::NonCompliant
        };
        
        let assessment = ComplianceAssessment {
            id: assessment_id,
            standard,
            assessment_date: Utc::now(),
            overall_score,
            status: overall_status,
            total_requirements,
            compliant_requirements,
            non_compliant_requirements,
            critical_gaps,
            recommendations,
            evidence_collected,
            next_assessment_due: Utc::now() + Duration::days(self.config.assessment_frequency_days as i64),
        };
        
        self.assessments.push(assessment.clone());
        
        Ok(assessment)
    }
    
    /// Check individual compliance requirement
    async fn check_compliance_requirement(&mut self, requirement_id: &str) -> Result<ComplianceCheckResult> {
        let requirement = self.requirements.get(requirement_id)
            .ok_or_else(|| TwoPasswordError::SecurityError("Requirement not found".to_string()))?;
        
        let mut score = 0.0;
        let mut status = ComplianceStatus::NonCompliant;
        let mut evidence = Vec::new();
        let mut gaps = Vec::new();
        let mut recommendations = Vec::new();
        
        // Run automated checks if available
        if let Some(check_name) = &requirement.automated_check {
            match self.run_automated_check(check_name).await {
                Ok(check_score) => {
                    score = check_score;
                    if score >= 0.95 {
                        status = ComplianceStatus::Compliant;
                        evidence.push(format!("Automated check '{}' passed with score {:.2}", check_name, score));
                    } else if score >= 0.70 {
                        status = ComplianceStatus::PartiallyCompliant;
                        evidence.push(format!("Automated check '{}' partially passed with score {:.2}", check_name, score));
                        gaps.push("Some automated checks failed".to_string());
                        recommendations.push("Review and address failed automated checks".to_string());
                    } else {
                        gaps.push(format!("Automated check '{}' failed with score {:.2}", check_name, score));
                        recommendations.push(requirement.remediation_guidance.clone());
                    }
                }
                Err(_) => {
                    gaps.push("Automated check could not be performed".to_string());
                    recommendations.push("Manual review required".to_string());
                    status = ComplianceStatus::PendingReview;
                }
            }
        } else {
            // Manual assessment required
            status = ComplianceStatus::PendingReview;
            recommendations.push("Manual compliance assessment required".to_string());
        }
        
        Ok(ComplianceCheckResult {
            requirement_id: requirement_id.to_string(),
            status,
            score,
            evidence,
            gaps,
            recommendations,
            last_checked: Utc::now(),
            next_check_due: Utc::now() + Duration::days(self.config.assessment_frequency_days as i64),
        })
    }
    
    /// Run automated compliance check
    async fn run_automated_check(&self, check_name: &str) -> Result<f64> {
        // In a real implementation, these would be actual system checks
        match check_name {
            "check_access_controls" => {
                // Check if access controls are properly configured
                // For simulation, return a score based on basic criteria
                Ok(0.85) // Good but not perfect
            }
            "check_data_protection" => {
                // Check encryption and data protection measures
                Ok(0.90) // Very good
            }
            "check_monitoring_systems" => {
                // Check security monitoring capabilities
                Ok(0.75) // Partially compliant
            }
            "check_data_security" => {
                // GDPR data security checks
                Ok(0.80) // Good
            }
            "check_breach_procedures" => {
                // Check breach notification procedures
                Ok(0.70) // Minimum compliance
            }
            "check_data_deletion" => {
                // Check data deletion capabilities
                Ok(0.65) // Needs improvement
            }
            "check_user_identification" => {
                // HIPAA user identification checks
                Ok(0.95) // Excellent
            }
            "check_transmission_security" => {
                // HIPAA transmission security
                Ok(0.88) // Very good
            }
            "check_access_policy" => {
                // ISO 27001 access policy check
                Ok(0.82) // Good
            }
            "check_data_encryption" => {
                // PCI DSS encryption check
                Ok(0.92) // Excellent
            }
            "check_identity_management" => {
                // NIST identity management check
                Ok(0.78) // Good
            }
            _ => Err(TwoPasswordError::SecurityError(format!("Unknown check: {}", check_name))),
        }
    }
    
    /// Add compliance evidence
    pub async fn add_evidence(
        &mut self,
        requirement_id: String,
        evidence_type: String,
        description: String,
        collected_by: String,
        file_path: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Result<String> {
        let evidence_id = format!("evidence_{}_{}", requirement_id, Utc::now().timestamp());
        
        let evidence = ComplianceEvidence {
            id: evidence_id.clone(),
            requirement_id: requirement_id.clone(),
            evidence_type,
            description,
            collected_at: Utc::now(),
            collected_by,
            file_path,
            metadata,
        };
        
        self.evidence_store
            .entry(requirement_id)
            .or_insert_with(Vec::new)
            .push(evidence);
        
        Ok(evidence_id)
    }
    
    /// Generate compliance report
    pub async fn generate_compliance_report(&self) -> Result<ComplianceReport> {
        let mut total_requirements = 0u32;
        let mut compliant_requirements = 0u32;
        let mut critical_gaps = 0u32;
        let mut standards_status = HashMap::new();
        
        // Analyze latest assessments
        for standard in &self.config.enabled_standards {
            if let Some(latest_assessment) = self.assessments.iter()
                .filter(|a| a.standard == *standard)
                .max_by_key(|a| a.assessment_date) {
                
                total_requirements += latest_assessment.total_requirements;
                compliant_requirements += latest_assessment.compliant_requirements;
                
                if !latest_assessment.critical_gaps.is_empty() {
                    critical_gaps += latest_assessment.critical_gaps.len() as u32;
                }
                
                standards_status.insert(standard.clone(), latest_assessment.status.clone());
            }
        }
        
        let overall_compliance_percentage = if total_requirements > 0 {
            (compliant_requirements as f64 / total_requirements as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(ComplianceReport {
            generated_at: Utc::now(),
            enabled_standards: self.config.enabled_standards.clone(),
            total_requirements,
            compliant_requirements,
            overall_compliance_percentage,
            critical_gaps,
            standards_status,
            recent_assessments: self.assessments.iter()
                .rev()
                .take(5)
                .cloned()
                .collect(),
            evidence_count: self.evidence_store.values()
                .map(|v| v.len())
                .sum::<usize>() as u32,
            next_assessment_due: self.assessments.iter()
                .map(|a| a.next_assessment_due)
                .min(),
        })
    }
    
    /// Get compliance status for specific standard
    pub fn get_standard_status(&self, standard: &ComplianceStandard) -> Option<ComplianceStatus> {
        self.assessments.iter()
            .filter(|a| a.standard == *standard)
            .max_by_key(|a| a.assessment_date)
            .map(|a| a.status.clone())
    }
    
    /// Get requirements needing attention
    pub fn get_priority_requirements(&self) -> Vec<&ComplianceRequirement> {
        let mut requirements = Vec::new();
        
        for (req_id, requirement) in &self.requirements {
            if let Some(result) = self.assessment_results.get(req_id) {
                if result.status == ComplianceStatus::NonCompliant && 
                   requirement.severity == ComplianceSeverity::Critical {
                    requirements.push(requirement);
                }
            }
        }
        
        requirements
    }
}

/// Compliance report
#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub generated_at: DateTime<Utc>,
    pub enabled_standards: Vec<ComplianceStandard>,
    pub total_requirements: u32,
    pub compliant_requirements: u32,
    pub overall_compliance_percentage: f64,
    pub critical_gaps: u32,
    pub standards_status: HashMap<ComplianceStandard, ComplianceStatus>,
    pub recent_assessments: Vec<ComplianceAssessment>,
    pub evidence_count: u32,
    pub next_assessment_due: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compliance_manager_creation() {
        let manager = ComplianceManager::new();
        assert!(!manager.requirements.is_empty());
        assert!(manager.config.enabled_standards.contains(&ComplianceStandard::SOC2));
        assert!(manager.config.enabled_standards.contains(&ComplianceStandard::GDPR));
    }
    
    #[tokio::test]
    async fn test_compliance_standard_names() {
        assert_eq!(ComplianceStandard::SOC2.name(), "SOC 2");
        assert_eq!(ComplianceStandard::GDPR.name(), "GDPR");
        assert_eq!(ComplianceStandard::HIPAA.name(), "HIPAA");
        assert_eq!(ComplianceStandard::ISO27001.name(), "ISO 27001");
    }
    
    #[tokio::test]
    async fn test_automated_check() {
        let manager = ComplianceManager::new();
        
        let score = manager.run_automated_check("check_access_controls").await.unwrap();
        assert!(score >= 0.0 && score <= 1.0);
        
        let invalid_result = manager.run_automated_check("invalid_check").await;
        assert!(invalid_result.is_err());
    }
    
    #[tokio::test]
    async fn test_soc2_assessment() {
        let mut manager = ComplianceManager::new();
        
        let assessment = manager.run_assessment(ComplianceStandard::SOC2).await.unwrap();
        
        assert_eq!(assessment.standard, ComplianceStandard::SOC2);
        assert!(assessment.total_requirements > 0);
        assert!(assessment.overall_score >= 0.0 && assessment.overall_score <= 1.0);
    }
    
    #[tokio::test]
    async fn test_evidence_collection() {
        let mut manager = ComplianceManager::new();
        
        let evidence_id = manager.add_evidence(
            "soc2_cc6_1".to_string(),
            "document".to_string(),
            "Access control policy document".to_string(),
            "auditor@example.com".to_string(),
            Some("/path/to/document.pdf".to_string()),
            HashMap::new(),
        ).await.unwrap();
        
        assert!(evidence_id.starts_with("evidence_soc2_cc6_1"));
        assert!(manager.evidence_store.contains_key("soc2_cc6_1"));
        assert_eq!(manager.evidence_store.get("soc2_cc6_1").unwrap().len(), 1);
    }
    
    #[tokio::test]
    async fn test_compliance_report_generation() {
        let mut manager = ComplianceManager::new();
        
        // Run some assessments first
        manager.run_assessment(ComplianceStandard::SOC2).await.unwrap();
        manager.run_assessment(ComplianceStandard::GDPR).await.unwrap();
        
        let report = manager.generate_compliance_report().await.unwrap();
        
        assert_eq!(report.enabled_standards.len(), 2);
        assert!(report.total_requirements > 0);
        assert!(report.overall_compliance_percentage >= 0.0 && report.overall_compliance_percentage <= 100.0);
        assert_eq!(report.recent_assessments.len(), 2);
    }
    
    #[tokio::test]
    async fn test_priority_requirements() {
        let mut manager = ComplianceManager::new();
        
        // Run assessment to populate results
        manager.run_assessment(ComplianceStandard::SOC2).await.unwrap();
        
        let priority_requirements = manager.get_priority_requirements();
        
        // All priority requirements should be critical
        for req in priority_requirements {
            assert_eq!(req.severity, ComplianceSeverity::Critical);
        }
    }
}