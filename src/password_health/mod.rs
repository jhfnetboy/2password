//! Password Health Analysis and Dashboard
//!
//! This module provides comprehensive password health monitoring including:
//! - Advanced password strength analysis using zxcvbn
//! - Reused password detection across vault entries
//! - Breach checking via HaveIBeenPwned API
//! - Security scoring and dashboard metrics

use crate::storage::PasswordEntry;
use crate::{Result, TwoPasswordError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use lru::LruCache;
use std::num::NonZeroUsize;

pub mod analyzer;
pub mod breach_checker;
pub mod dashboard;
pub mod scorer;

/// Password analysis result with detailed feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordAnalysis {
    /// Strength score from 0-100
    pub score: u8,
    /// Estimated time to crack (in seconds)
    pub crack_times: CrackTimes,
    /// Feedback messages for improvement
    pub feedback: Vec<String>,
    /// Warnings about the password
    pub warnings: Vec<String>,
    /// Pattern matches found in the password
    pub patterns: Vec<PatternMatch>,
}

/// Time estimates for cracking a password under different scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrackTimes {
    /// Online attack throttled (100/hour)
    pub online_throttled: f64,
    /// Online attack unthrottled (10/second)
    pub online_unthrottled: f64,
    /// Offline attack slow hashing (10^4/second)
    pub offline_slow: f64,
    /// Offline attack fast hashing (10^10/second)
    pub offline_fast: f64,
}

/// Pattern found in password analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    /// Type of pattern (dictionary, sequence, repeat, etc.)
    pub pattern_type: String,
    /// The matched text
    pub matched_word: String,
    /// Position in password
    pub start: usize,
    pub end: usize,
    /// Entropy contribution
    pub entropy: f64,
}

/// Result of reused password detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReusedPasswordGroup {
    /// The password hash (for identification)
    pub password_hash: String,
    /// Entries that share this password
    pub entries: Vec<uuid::Uuid>, // Entry IDs
    /// Risk level based on site types
    pub risk_level: RiskLevel,
}

/// Risk level for reused passwords
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Result from breach checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreachCheckResult {
    /// Whether the password was found in breaches
    pub is_breached: bool,
    /// Number of times found in breaches (if any)
    pub breach_count: Option<u64>,
    /// Last check timestamp
    pub last_checked: chrono::DateTime<chrono::Utc>,
}

/// Overall security score for the vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScore {
    /// Overall score from 0-100
    pub total_score: u8,
    /// Individual component scores
    pub strength_score: u8,
    pub uniqueness_score: u8,
    pub age_score: u8,
    pub breach_score: u8,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
    /// Count of issues by severity
    pub weak_passwords: usize,
    pub reused_passwords: usize,
    pub breached_passwords: usize,
    pub old_passwords: usize,
}

/// Password Health Dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    /// Overall security metrics
    pub security_score: SecurityScore,
    /// Detailed password analyses
    pub password_analyses: HashMap<uuid::Uuid, PasswordAnalysis>, // Entry ID -> Analysis
    /// Reused password groups
    pub reused_groups: Vec<ReusedPasswordGroup>,
    /// Breach check results
    pub breach_results: HashMap<uuid::Uuid, BreachCheckResult>, // Entry ID -> Breach result
    /// Password age distribution
    pub age_distribution: HashMap<String, usize>, // Age range -> Count
    /// Generated timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Main Password Health service
pub struct PasswordHealthService {
    /// Cache for password analyses to avoid recomputation
    analysis_cache: Mutex<LruCache<String, PasswordAnalysis>>,
    /// Cache for breach check results
    breach_cache: Mutex<LruCache<String, BreachCheckResult>>,
}

impl PasswordHealthService {
    /// Create new password health service
    pub fn new() -> Self {
        Self {
            analysis_cache: Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())),
            breach_cache: Mutex::new(LruCache::new(NonZeroUsize::new(10000).unwrap())),
        }
    }

    /// Generate comprehensive dashboard data for all vault entries
    pub async fn generate_dashboard(&self, entries: &[PasswordEntry]) -> Result<DashboardData> {
        let mut password_analyses = HashMap::new();
        let mut breach_results = HashMap::new();

        // Analyze each password
        for entry in entries {
            if !entry.password.is_empty() {
                let analysis = self.analyze_password(&entry.password)?;
                password_analyses.insert(entry.id, analysis);

                // Check for breaches (async)
                let breach_result = self.check_breach(&entry.password).await?;
                breach_results.insert(entry.id, breach_result);
            }
        }

        // Detect reused passwords
        let reused_groups = self.detect_reused_passwords(entries)?;

        // Calculate security score
        let security_score = self.calculate_security_score(entries, &password_analyses, &reused_groups, &breach_results)?;

        // Calculate age distribution
        let age_distribution = self.calculate_age_distribution(entries);

        Ok(DashboardData {
            security_score,
            password_analyses,
            reused_groups,
            breach_results,
            age_distribution,
            generated_at: chrono::Utc::now(),
        })
    }

    /// Analyze a single password strength
    pub fn analyze_password(&self, password: &str) -> Result<PasswordAnalysis> {
        // Check cache first
        let cache_key = format!("{:x}", md5::compute(password.as_bytes()));
        
        if let Ok(mut cache) = self.analysis_cache.lock() {
            if let Some(cached_result) = cache.get(&cache_key) {
                return Ok(cached_result.clone());
            }
        }

        // Perform analysis
        let result = analyzer::analyze_password_strength(password)?;
        
        // Cache result
        if let Ok(mut cache) = self.analysis_cache.lock() {
            cache.put(cache_key, result.clone());
        }

        Ok(result)
    }

    /// Check if password has been breached
    pub async fn check_breach(&self, password: &str) -> Result<BreachCheckResult> {
        // Check cache first
        let cache_key = format!("{:x}", sha1::Sha1::digest(password.as_bytes()));
        
        if let Ok(mut cache) = self.breach_cache.lock() {
            if let Some(cached_result) = cache.get(&cache_key) {
                // Check if cache is still valid (24 hours)
                let age = chrono::Utc::now() - cached_result.last_checked;
                if age.num_hours() < 24 {
                    return Ok(cached_result.clone());
                }
            }
        }

        // Perform breach check
        let result = breach_checker::check_password_breach(password).await?;
        
        // Cache result
        if let Ok(mut cache) = self.breach_cache.lock() {
            cache.put(cache_key, result.clone());
        }

        Ok(result)
    }

    /// Detect reused passwords across vault entries
    pub fn detect_reused_passwords(&self, entries: &[PasswordEntry]) -> Result<Vec<ReusedPasswordGroup>> {
        let mut password_groups: HashMap<String, Vec<uuid::Uuid>> = HashMap::new();
        
        // Group entries by password hash
        for entry in entries {
            if !entry.password.is_empty() {
                let password_hash = format!("{:x}", sha2::Sha256::digest(entry.password.as_bytes()));
                password_groups.entry(password_hash)
                    .or_insert_with(Vec::new)
                    .push(entry.id);
            }
        }

        // Find groups with more than one entry (reused passwords)
        let mut reused_groups = Vec::new();
        for (password_hash, entry_ids) in password_groups {
            if entry_ids.len() > 1 {
                let risk_level = self.assess_reuse_risk(&entry_ids, entries);
                reused_groups.push(ReusedPasswordGroup {
                    password_hash,
                    entries: entry_ids,
                    risk_level,
                });
            }
        }

        Ok(reused_groups)
    }

    /// Calculate overall security score
    pub fn calculate_security_score(
        &self,
        entries: &[PasswordEntry],
        analyses: &HashMap<uuid::Uuid, PasswordAnalysis>,
        reused_groups: &[ReusedPasswordGroup],
        breach_results: &HashMap<uuid::Uuid, BreachCheckResult>,
    ) -> Result<SecurityScore> {
        scorer::calculate_vault_security_score(entries, analyses, reused_groups, breach_results)
    }

    /// Assess risk level for reused password group
    fn assess_reuse_risk(&self, entry_ids: &[uuid::Uuid], entries: &[PasswordEntry]) -> RiskLevel {
        let mut high_value_sites = 0;
        let mut financial_sites = 0;
        let total_sites = entry_ids.len();

        for entry_id in entry_ids {
            if let Some(entry) = entries.iter().find(|e| &e.id == entry_id) {
                let url_lower = entry.url.as_ref().unwrap_or(&String::new()).to_lowercase();
                
                // Check for high-value sites
                if url_lower.contains("bank") || url_lower.contains("paypal") 
                   || url_lower.contains("amazon") || url_lower.contains("apple") {
                    high_value_sites += 1;
                }
                
                // Check for financial sites
                if url_lower.contains("bank") || url_lower.contains("credit") 
                   || url_lower.contains("invest") || url_lower.contains("finance") {
                    financial_sites += 1;
                }
            }
        }

        // Determine risk level
        match (high_value_sites, financial_sites, total_sites) {
            (h, f, _) if h > 0 || f > 1 => RiskLevel::Critical,
            (_, f, _) if f > 0 => RiskLevel::High,
            (_, _, t) if t > 5 => RiskLevel::High,
            (_, _, t) if t > 3 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }

    /// Calculate password age distribution
    fn calculate_age_distribution(&self, entries: &[PasswordEntry]) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        let now = chrono::Utc::now();

        for entry in entries {
            let age_days = (now - entry.created_at).num_days();
            let age_range = match age_days {
                0..=30 => "0-30 days",
                31..=90 => "31-90 days", 
                91..=365 => "3-12 months",
                366..=730 => "1-2 years",
                _ => "2+ years",
            };
            
            *distribution.entry(age_range.to_string()).or_insert(0) += 1;
        }

        distribution
    }
}

impl Default for PasswordHealthService {
    fn default() -> Self {
        Self::new()
    }
}

// Add required hash imports
use md5;
use sha1;
use sha2::{Sha256, Digest};