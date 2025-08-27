//! Advanced Security Score Calculation Engine
//!
//! Provides comprehensive security scoring with detailed analysis,
//! improvement recommendations, and risk prioritization.

use crate::password_health::{PasswordAnalysis, ReusedPasswordGroup, BreachCheckResult, SecurityScore, RiskLevel};
use crate::storage::PasswordEntry;
use crate::{Result, TwoPasswordError};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, BTreeMap};
use uuid::Uuid;

/// Individual password security assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordSecurityAssessment {
    pub entry_id: Uuid,
    pub title: String,
    pub overall_score: u8,        // 0-100
    pub strength_score: u8,       // 0-100
    pub uniqueness_score: u8,     // 0-100
    pub age_score: u8,            // 0-100
    pub breach_score: u8,         // 0-100
    pub risk_level: RiskLevel,
    pub issues: Vec<SecurityIssue>,
    pub recommendations: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

/// Security issue types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityIssue {
    WeakPassword { strength_score: u8 },
    ReusedPassword { reuse_count: usize },
    OldPassword { age_days: i64 },
    BreachedPassword { breach_count: Option<u64> },
    CommonPattern { pattern_type: String },
    ShortLength { length: usize, minimum: usize },
    NoSpecialChars,
    NoNumbers,
    NoUppercase,
    NoLowercase,
    Dictionary { matched_words: Vec<String> },
    Keyboard { pattern_length: usize },
    Sequential { pattern_type: String },
}

/// Vault-wide security metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSecurityMetrics {
    pub total_passwords: usize,
    pub security_distribution: SecurityDistribution,
    pub issue_summary: IssueSummary,
    pub improvement_potential: f64,  // 0.0-1.0
    pub estimated_fix_time: EstimatedTime,
    pub priority_actions: Vec<PriorityAction>,
}

/// Security score distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityDistribution {
    pub excellent: usize,  // 90-100
    pub good: usize,       // 75-89
    pub fair: usize,       // 60-74
    pub poor: usize,       // 40-59
    pub critical: usize,   // 0-39
}

/// Summary of security issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSummary {
    pub weak_passwords: usize,
    pub reused_passwords: usize,
    pub old_passwords: usize,
    pub breached_passwords: usize,
    pub total_issues: usize,
    pub critical_issues: usize,
}

/// Time estimates for improvements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimatedTime {
    pub quick_wins: Duration,    // < 5 minutes
    pub moderate_fixes: Duration, // 5-30 minutes
    pub major_overhauls: Duration, // > 30 minutes
    pub total_time: Duration,
}

/// Priority action item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityAction {
    pub action_type: ActionType,
    pub title: String,
    pub description: String,
    pub impact_score: u8,      // 0-100 - how much this improves security
    pub effort_estimate: Duration,
    pub affected_passwords: Vec<Uuid>,
    pub priority: ActionPriority,
}

/// Types of security actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    UpdateWeakPassword,
    ReplaceReusedPassword,
    RefreshOldPassword,
    ChangeBreachedPassword,
    EnableTwoFactor,
    AuditHighValueAccounts,
    ConsolidateAccounts,
    ReviewPermissions,
}

/// Action priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionPriority {
    Critical,   // Security breach risk
    High,       // Significant vulnerability
    Medium,     // Good security practice
    Low,        // Nice to have
}

/// Advanced security scorer with detailed analytics
pub struct SecurityScorer {
    current_time: DateTime<Utc>,
    score_cache: HashMap<Uuid, PasswordSecurityAssessment>,
    vault_metrics: Option<VaultSecurityMetrics>,
}

impl SecurityScorer {
    /// Create new security scorer
    pub fn new() -> Self {
        Self {
            current_time: Utc::now(),
            score_cache: HashMap::new(),
            vault_metrics: None,
        }
    }

    /// Create scorer with specific time (for testing)
    pub fn with_time(time: DateTime<Utc>) -> Self {
        Self {
            current_time: time,
            score_cache: HashMap::new(),
            vault_metrics: None,
        }
    }

    /// Calculate comprehensive security assessment for entire vault
    pub fn calculate_vault_assessment(
        &mut self,
        entries: &[PasswordEntry],
        analyses: &HashMap<Uuid, PasswordAnalysis>,
        reused_groups: &[ReusedPasswordGroup],
        breach_results: &HashMap<Uuid, BreachCheckResult>,
    ) -> Result<(SecurityScore, VaultSecurityMetrics)> {
        // Clear previous calculations
        self.score_cache.clear();

        if entries.is_empty() {
            let empty_score = SecurityScore {
                total_score: 100,
                strength_score: 100,
                uniqueness_score: 100,
                age_score: 100,
                breach_score: 100,
                recommendations: vec!["Add passwords to start building your vault".to_string()],
                weak_passwords: 0,
                reused_passwords: 0,
                breached_passwords: 0,
                old_passwords: 0,
            };

            let empty_metrics = VaultSecurityMetrics {
                total_passwords: 0,
                security_distribution: SecurityDistribution {
                    excellent: 0, good: 0, fair: 0, poor: 0, critical: 0,
                },
                issue_summary: IssueSummary {
                    weak_passwords: 0, reused_passwords: 0, old_passwords: 0,
                    breached_passwords: 0, total_issues: 0, critical_issues: 0,
                },
                improvement_potential: 0.0,
                estimated_fix_time: EstimatedTime {
                    quick_wins: Duration::zero(),
                    moderate_fixes: Duration::zero(),
                    major_overhauls: Duration::zero(),
                    total_time: Duration::zero(),
                },
                priority_actions: vec![],
            };

            return Ok((empty_score, empty_metrics));
        }

        // Calculate individual password assessments
        for entry in entries {
            let assessment = self.assess_individual_password(
                entry, 
                analyses.get(&entry.id),
                reused_groups,
                breach_results.get(&entry.id)
            )?;
            self.score_cache.insert(entry.id, assessment);
        }

        // Calculate vault-wide metrics
        let vault_metrics = self.calculate_vault_metrics(entries, reused_groups)?;
        self.vault_metrics = Some(vault_metrics.clone());

        // Generate overall security score
        let security_score = self.calculate_overall_security_score(entries, reused_groups, breach_results)?;

        Ok((security_score, vault_metrics))
    }

    /// Assess individual password security
    fn assess_individual_password(
        &self,
        entry: &PasswordEntry,
        analysis: Option<&PasswordAnalysis>,
        reused_groups: &[ReusedPasswordGroup],
        breach_result: Option<&BreachCheckResult>,
    ) -> Result<PasswordSecurityAssessment> {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Calculate strength score
        let strength_score = if let Some(analysis) = analysis {
            self.calculate_strength_component(&entry.password, &mut issues, &mut recommendations, analysis)
        } else {
            50 // Default score if no analysis available
        };

        // Calculate uniqueness score
        let uniqueness_score = self.calculate_uniqueness_component(
            &mut issues, &mut recommendations, entry, reused_groups
        );

        // Calculate age score
        let age_score = self.calculate_age_component(
            &mut issues, &mut recommendations, entry
        );

        // Calculate breach score
        let breach_score = self.calculate_breach_component(
            &mut issues, &mut recommendations, entry, breach_result
        );

        // Calculate weighted overall score
        let overall_score = (
            (strength_score as f64 * 0.40) +
            (uniqueness_score as f64 * 0.25) +
            (age_score as f64 * 0.20) +
            (breach_score as f64 * 0.15)
        ).round() as u8;

        // Determine risk level
        let risk_level = match overall_score {
            90..=100 => RiskLevel::Low,
            75..=89 => RiskLevel::Medium,
            60..=74 => RiskLevel::Medium,
            40..=59 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        Ok(PasswordSecurityAssessment {
            entry_id: entry.id,
            title: entry.title.clone(),
            overall_score,
            strength_score,
            uniqueness_score,
            age_score,
            breach_score,
            risk_level,
            issues,
            recommendations,
            last_updated: self.current_time,
        })
    }

    /// Calculate strength component score
    fn calculate_strength_component(
        &self,
        password: &str,
        issues: &mut Vec<SecurityIssue>,
        recommendations: &mut Vec<String>,
        analysis: &PasswordAnalysis,
    ) -> u8 {
        let mut score = 100u8;

        // Length assessment
        if password.len() < 8 {
            issues.push(SecurityIssue::ShortLength { 
                length: password.len(), 
                minimum: 12 
            });
            recommendations.push("Use at least 12 characters for better security".to_string());
            score = score.saturating_sub(30);
        } else if password.len() < 12 {
            issues.push(SecurityIssue::ShortLength { 
                length: password.len(), 
                minimum: 12 
            });
            recommendations.push("Consider using at least 12 characters".to_string());
            score = score.saturating_sub(10);
        }

        // Character variety assessment based on feedback
        let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
        let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_symbol = password.chars().any(|c| !c.is_ascii_alphanumeric());

        if !has_upper {
            issues.push(SecurityIssue::NoUppercase);
            recommendations.push("Include uppercase letters (A-Z)".to_string());
            score = score.saturating_sub(10);
        }

        if !has_lower {
            issues.push(SecurityIssue::NoLowercase);
            recommendations.push("Include lowercase letters (a-z)".to_string());
            score = score.saturating_sub(10);
        }

        if !has_digit {
            issues.push(SecurityIssue::NoNumbers);
            recommendations.push("Include numbers (0-9)".to_string());
            score = score.saturating_sub(10);
        }

        if !has_symbol {
            issues.push(SecurityIssue::NoSpecialChars);
            recommendations.push("Include special characters (!@#$%^&*)".to_string());
            score = score.saturating_sub(10);
        }

        // Pattern-based deductions
        for pattern in &analysis.patterns {
            match pattern.pattern_type.as_str() {
                "dictionary" => {
                    issues.push(SecurityIssue::Dictionary { 
                        matched_words: vec![pattern.matched_word.clone()] 
                    });
                    recommendations.push("Avoid dictionary words - use random combinations".to_string());
                    score = score.saturating_sub(15);
                }
                "keyboard" => {
                    issues.push(SecurityIssue::Keyboard { 
                        pattern_length: pattern.matched_word.len() 
                    });
                    recommendations.push("Avoid keyboard patterns like 'qwerty' or '123456'".to_string());
                    score = score.saturating_sub(20);
                }
                "repeat" => {
                    recommendations.push("Avoid repeating characters or patterns".to_string());
                    score = score.saturating_sub(15);
                }
                "sequence" => {
                    issues.push(SecurityIssue::Sequential { 
                        pattern_type: "character sequence".to_string() 
                    });
                    recommendations.push("Avoid sequential patterns like 'abc' or '123'".to_string());
                    score = score.saturating_sub(15);
                }
                _ => {
                    score = score.saturating_sub(5);
                }
            }
        }

        // Overall weakness assessment
        if analysis.score < 3 {
            issues.push(SecurityIssue::WeakPassword { 
                strength_score: (analysis.score * 25) as u8 
            });
            recommendations.push("This password is very weak - consider using a password generator".to_string());
        }

        score
    }

    /// Calculate uniqueness component score
    fn calculate_uniqueness_component(
        &self,
        issues: &mut Vec<SecurityIssue>,
        recommendations: &mut Vec<String>,
        entry: &PasswordEntry,
        reused_groups: &[ReusedPasswordGroup],
    ) -> u8 {
        for group in reused_groups {
            if group.entries.iter().any(|e| *e == entry.id) {
                let reuse_count = group.entries.len();
                issues.push(SecurityIssue::ReusedPassword { reuse_count });
                
                let recommendation = match reuse_count {
                    2 => "This password is used in 1 other account - make it unique".to_string(),
                    3..=5 => format!("This password is reused in {} other accounts - create unique passwords", reuse_count - 1),
                    _ => format!("This password is heavily reused ({} accounts) - urgent update needed", reuse_count),
                };
                recommendations.push(recommendation);

                // Score penalty based on reuse severity
                return match reuse_count {
                    2 => 75,      // -25 for single reuse
                    3..=5 => 50,  // -50 for moderate reuse
                    _ => 25,      // -75 for heavy reuse
                };
            }
        }
        100 // Unique password
    }

    /// Calculate age component score
    fn calculate_age_component(
        &self,
        issues: &mut Vec<SecurityIssue>,
        recommendations: &mut Vec<String>,
        entry: &PasswordEntry,
    ) -> u8 {
        let age = self.current_time - entry.updated_at;
        let age_days = age.num_days();

        match age_days {
            0..=90 => 100,        // Fresh (0-3 months)
            91..=365 => 85,       // Good (3-12 months)
            366..=730 => {        // Old (1-2 years)
                issues.push(SecurityIssue::OldPassword { age_days });
                recommendations.push("Consider updating this password - it's over a year old".to_string());
                60
            }
            _ => {                // Ancient (2+ years)
                issues.push(SecurityIssue::OldPassword { age_days });
                recommendations.push("This password is very old - update immediately".to_string());
                25
            }
        }
    }

    /// Calculate breach component score
    fn calculate_breach_component(
        &self,
        issues: &mut Vec<SecurityIssue>,
        recommendations: &mut Vec<String>,
        _entry: &PasswordEntry,
        breach_result: Option<&BreachCheckResult>,
    ) -> u8 {
        if let Some(result) = breach_result {
            if result.is_breached {
                issues.push(SecurityIssue::BreachedPassword { 
                    breach_count: result.breach_count 
                });
                
                let recommendation = match result.breach_count {
                    Some(count) if count > 1000 => "This password appears in major data breaches - change immediately!".to_string(),
                    Some(count) if count > 100 => format!("This password was found in {} breaches - change soon", count),
                    Some(count) => format!("This password was found in {} breach(es) - consider changing", count),
                    None => "This password appears in known data breaches - change it".to_string(),
                };
                recommendations.push(recommendation);
                
                // Score penalty based on breach severity
                return match result.breach_count {
                    Some(count) if count > 1000 => 0,   // Critical
                    Some(count) if count > 100 => 25,  // Very bad
                    Some(count) if count > 10 => 50,   // Bad
                    _ => 75,                            // Concerning but not critical
                };
            }
        }
        100 // Not breached or unknown
    }

    /// Calculate vault-wide security metrics
    fn calculate_vault_metrics(
        &self,
        entries: &[PasswordEntry],
        _reused_groups: &[ReusedPasswordGroup],
    ) -> Result<VaultSecurityMetrics> {
        let mut security_distribution = SecurityDistribution {
            excellent: 0, good: 0, fair: 0, poor: 0, critical: 0,
        };

        let mut issue_summary = IssueSummary {
            weak_passwords: 0, reused_passwords: 0, old_passwords: 0,
            breached_passwords: 0, total_issues: 0, critical_issues: 0,
        };

        let mut total_score = 0u64;
        let mut total_improvement_potential = 0.0;
        let mut priority_actions = Vec::new();

        // Analyze each password assessment
        for assessment in self.score_cache.values() {
            total_score += assessment.overall_score as u64;

            // Update distribution
            match assessment.overall_score {
                90..=100 => security_distribution.excellent += 1,
                75..=89 => security_distribution.good += 1,
                60..=74 => security_distribution.fair += 1,
                40..=59 => security_distribution.poor += 1,
                _ => security_distribution.critical += 1,
            }

            // Count issues
            for issue in &assessment.issues {
                match issue {
                    SecurityIssue::WeakPassword { .. } => issue_summary.weak_passwords += 1,
                    SecurityIssue::ReusedPassword { .. } => issue_summary.reused_passwords += 1,
                    SecurityIssue::OldPassword { .. } => issue_summary.old_passwords += 1,
                    SecurityIssue::BreachedPassword { .. } => issue_summary.breached_passwords += 1,
                    _ => {}
                }
                issue_summary.total_issues += 1;

                if matches!(assessment.risk_level, RiskLevel::Critical) {
                    issue_summary.critical_issues += 1;
                }
            }

            // Calculate improvement potential (100 - current score)
            total_improvement_potential += (100 - assessment.overall_score) as f64;

            // Generate priority actions
            if assessment.overall_score < 75 {
                priority_actions.extend(self.generate_priority_actions_for_password(assessment));
            }
        }

        // Calculate average improvement potential
        let improvement_potential = if !entries.is_empty() {
            total_improvement_potential / (entries.len() as f64 * 100.0)
        } else {
            0.0
        };

        // Sort priority actions by priority and impact
        priority_actions.sort_by(|a, b| {
            a.priority.cmp(&b.priority)
                .then_with(|| b.impact_score.cmp(&a.impact_score))
        });

        // Calculate estimated fix times
        let estimated_fix_time = self.calculate_estimated_fix_time(&priority_actions);

        Ok(VaultSecurityMetrics {
            total_passwords: entries.len(),
            security_distribution,
            issue_summary,
            improvement_potential,
            estimated_fix_time,
            priority_actions,
        })
    }

    /// Generate priority actions for a specific password
    fn generate_priority_actions_for_password(
        &self,
        assessment: &PasswordSecurityAssessment,
    ) -> Vec<PriorityAction> {
        let mut actions = Vec::new();

        for issue in &assessment.issues {
            let action = match issue {
                SecurityIssue::WeakPassword { strength_score } => {
                    PriorityAction {
                        action_type: ActionType::UpdateWeakPassword,
                        title: format!("Strengthen weak password for {}", assessment.title),
                        description: "This password is too weak and vulnerable to attacks".to_string(),
                        impact_score: 100 - strength_score,
                        effort_estimate: Duration::minutes(2),
                        affected_passwords: vec![assessment.entry_id],
                        priority: if *strength_score < 25 { ActionPriority::Critical } else { ActionPriority::High },
                    }
                }
                SecurityIssue::ReusedPassword { reuse_count } => {
                    PriorityAction {
                        action_type: ActionType::ReplaceReusedPassword,
                        title: format!("Make unique password for {}", assessment.title),
                        description: format!("This password is reused across {} accounts", reuse_count),
                        impact_score: (reuse_count * 10).min(100) as u8,
                        effort_estimate: Duration::minutes(3),
                        affected_passwords: vec![assessment.entry_id],
                        priority: if *reuse_count > 5 { ActionPriority::Critical } else { ActionPriority::High },
                    }
                }
                SecurityIssue::OldPassword { age_days } => {
                    PriorityAction {
                        action_type: ActionType::RefreshOldPassword,
                        title: format!("Update old password for {}", assessment.title),
                        description: format!("This password is {} days old", age_days),
                        impact_score: (age_days / 10).min(100) as u8,
                        effort_estimate: Duration::minutes(4),
                        affected_passwords: vec![assessment.entry_id],
                        priority: if *age_days > 730 { ActionPriority::High } else { ActionPriority::Medium },
                    }
                }
                SecurityIssue::BreachedPassword { breach_count } => {
                    let impact = match breach_count {
                        Some(count) if *count > 1000 => 100,
                        Some(count) if *count > 100 => 85,
                        Some(count) if *count > 10 => 70,
                        _ => 60,
                    };
                    
                    PriorityAction {
                        action_type: ActionType::ChangeBreachedPassword,
                        title: format!("Change breached password for {}", assessment.title),
                        description: "This password appears in known data breaches".to_string(),
                        impact_score: impact,
                        effort_estimate: Duration::minutes(3),
                        affected_passwords: vec![assessment.entry_id],
                        priority: ActionPriority::Critical,
                    }
                }
                _ => continue,
            };
            actions.push(action);
        }

        actions
    }

    /// Calculate estimated time to fix all issues
    fn calculate_estimated_fix_time(&self, actions: &[PriorityAction]) -> EstimatedTime {
        let mut quick_wins = Duration::zero();
        let mut moderate_fixes = Duration::zero();
        let mut major_overhauls = Duration::zero();

        for action in actions {
            if action.effort_estimate <= Duration::minutes(5) {
                quick_wins = quick_wins + action.effort_estimate;
            } else if action.effort_estimate <= Duration::minutes(30) {
                moderate_fixes = moderate_fixes + action.effort_estimate;
            } else {
                major_overhauls = major_overhauls + action.effort_estimate;
            }
        }

        let total_time = quick_wins + moderate_fixes + major_overhauls;

        EstimatedTime {
            quick_wins,
            moderate_fixes,
            major_overhauls,
            total_time,
        }
    }

    /// Calculate overall security score using legacy format for compatibility
    fn calculate_overall_security_score(
        &self,
        entries: &[PasswordEntry],
        reused_groups: &[ReusedPasswordGroup],
        breach_results: &HashMap<Uuid, BreachCheckResult>,
    ) -> Result<SecurityScore> {
        let total_assessments: u32 = self.score_cache.values()
            .map(|a| a.overall_score as u32)
            .sum();
        
        let average_score = if !entries.is_empty() {
            (total_assessments / entries.len() as u32) as u8
        } else {
            100
        };

        // Component scores (weighted averages)
        let strength_scores: u32 = self.score_cache.values()
            .map(|a| a.strength_score as u32)
            .sum();
        let strength_score = if !entries.is_empty() {
            (strength_scores / entries.len() as u32) as u8
        } else {
            100
        };

        let uniqueness_scores: u32 = self.score_cache.values()
            .map(|a| a.uniqueness_score as u32)
            .sum();
        let uniqueness_score = if !entries.is_empty() {
            (uniqueness_scores / entries.len() as u32) as u8
        } else {
            100
        };

        let age_scores: u32 = self.score_cache.values()
            .map(|a| a.age_score as u32)
            .sum();
        let age_score = if !entries.is_empty() {
            (age_scores / entries.len() as u32) as u8
        } else {
            100
        };

        let breach_scores: u32 = self.score_cache.values()
            .map(|a| a.breach_score as u32)
            .sum();
        let breach_score = if !entries.is_empty() {
            (breach_scores / entries.len() as u32) as u8
        } else {
            100
        };

        // Count issues for legacy format
        let weak_passwords = self.score_cache.values()
            .filter(|a| a.strength_score < 75)
            .count() as u32;

        let reused_passwords = reused_groups.iter()
            .map(|g| g.entries.len())
            .sum::<usize>() as u32;

        let breached_passwords = breach_results.values()
            .filter(|r| r.is_breached)
            .count() as u32;

        let old_passwords = self.score_cache.values()
            .filter(|a| a.age_score < 75)
            .count() as u32;

        // Generate recommendations
        let recommendations = self.generate_overall_recommendations();

        Ok(SecurityScore {
            total_score: average_score,
            strength_score,
            uniqueness_score,
            age_score,
            breach_score,
            recommendations,
            weak_passwords: weak_passwords as usize,
            reused_passwords: reused_passwords as usize,
            breached_passwords: breached_passwords as usize,
            old_passwords: old_passwords as usize,
        })
    }

    /// Generate overall recommendations for the vault
    fn generate_overall_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if let Some(metrics) = &self.vault_metrics {
            // Priority-based recommendations
            if metrics.issue_summary.critical_issues > 0 {
                recommendations.push(format!(
                    "🚨 {} critical security issues need immediate attention",
                    metrics.issue_summary.critical_issues
                ));
            }

            if metrics.security_distribution.critical > 0 || metrics.security_distribution.poor > 0 {
                recommendations.push("Focus on your weakest passwords first for maximum security improvement".to_string());
            }

            if metrics.issue_summary.breached_passwords > 0 {
                recommendations.push(format!(
                    "Change {} breached password(s) immediately - they're known to attackers",
                    metrics.issue_summary.breached_passwords
                ));
            }

            if metrics.issue_summary.reused_passwords > 5 {
                recommendations.push("Use unique passwords for each account to prevent credential stuffing attacks".to_string());
            }

            if metrics.issue_summary.weak_passwords > 3 {
                recommendations.push("Consider using a password generator for stronger, random passwords".to_string());
            }

            if metrics.issue_summary.old_passwords > 2 {
                recommendations.push("Set up regular password rotation for your most important accounts".to_string());
            }

            // Positive reinforcement
            if metrics.security_distribution.excellent > metrics.total_passwords / 2 {
                recommendations.push("Great job! Most of your passwords are secure. Keep it up!".to_string());
            }

            // Time estimate
            if metrics.estimated_fix_time.total_time > Duration::zero() {
                let total_minutes = metrics.estimated_fix_time.total_time.num_minutes();
                if total_minutes < 30 {
                    recommendations.push(format!(
                        "All improvements can be completed in about {} minutes",
                        total_minutes
                    ));
                } else {
                    recommendations.push(format!(
                        "Consider tackling improvements over time - estimated {} hours total",
                        (total_minutes + 30) / 60
                    ));
                }
            }
        }

        if recommendations.is_empty() {
            recommendations.push("Your password security looks good! Keep monitoring for new threats.".to_string());
        }

        recommendations
    }

    /// Get individual password assessment
    pub fn get_password_assessment(&self, password_id: &Uuid) -> Option<&PasswordSecurityAssessment> {
        self.score_cache.get(password_id)
    }

    /// Get all password assessments sorted by score (worst first)
    pub fn get_all_assessments_by_priority(&self) -> Vec<&PasswordSecurityAssessment> {
        let mut assessments: Vec<&PasswordSecurityAssessment> = self.score_cache.values().collect();
        assessments.sort_by(|a, b| a.overall_score.cmp(&b.overall_score));
        assessments
    }

    /// Get vault security metrics
    pub fn get_vault_metrics(&self) -> Option<&VaultSecurityMetrics> {
        self.vault_metrics.as_ref()
    }
}

impl Default for SecurityScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_security_scorer_creation() {
        let scorer = SecurityScorer::new();
        assert!(scorer.score_cache.is_empty());
        assert!(scorer.vault_metrics.is_none());
    }

    #[test]
    fn test_empty_vault_assessment() {
        let mut scorer = SecurityScorer::new();
        let entries = vec![];
        let analyses = HashMap::new();
        let reused_groups = vec![];
        let breach_results = HashMap::new();

        let (score, metrics) = scorer.calculate_vault_assessment(
            &entries, &analyses, &reused_groups, &breach_results
        ).unwrap();

        assert_eq!(score.total_score, 100);
        assert_eq!(metrics.total_passwords, 0);
        assert_eq!(metrics.issue_summary.total_issues, 0);
    }

    #[test]
    fn test_security_issue_classification() {
        let weak_issue = SecurityIssue::WeakPassword { strength_score: 30 };
        let reused_issue = SecurityIssue::ReusedPassword { reuse_count: 3 };
        let old_issue = SecurityIssue::OldPassword { age_days: 400 };
        let breach_issue = SecurityIssue::BreachedPassword { breach_count: Some(1000) };

        // Test that different issue types are correctly classified
        assert!(matches!(weak_issue, SecurityIssue::WeakPassword { .. }));
        assert!(matches!(reused_issue, SecurityIssue::ReusedPassword { .. }));
        assert!(matches!(old_issue, SecurityIssue::OldPassword { .. }));
        assert!(matches!(breach_issue, SecurityIssue::BreachedPassword { .. }));
    }

    #[test]
    fn test_action_priority_ordering() {
        let critical = ActionPriority::Critical;
        let high = ActionPriority::High;
        let medium = ActionPriority::Medium;
        let low = ActionPriority::Low;

        assert!(critical < high);
        assert!(high < medium);
        assert!(medium < low);
    }

    #[test]
    fn test_security_distribution() {
        let distribution = SecurityDistribution {
            excellent: 2,
            good: 3,
            fair: 4,
            poor: 1,
            critical: 0,
        };

        let total = distribution.excellent + distribution.good + 
                   distribution.fair + distribution.poor + distribution.critical;
        assert_eq!(total, 10);
    }
}