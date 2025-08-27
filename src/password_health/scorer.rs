//! Security scoring and vault health assessment
//!
//! Calculates comprehensive security scores based on password strength,
//! uniqueness, age, and breach status to provide actionable recommendations.

use crate::password_health::{PasswordAnalysis, ReusedPasswordGroup, BreachCheckResult, SecurityScore, RiskLevel};
use crate::storage::PasswordEntry;
use crate::{Result, TwoPasswordError};
use std::collections::HashMap;

/// Weight factors for security score calculation
const STRENGTH_WEIGHT: f64 = 0.40; // 40% - Password strength
const UNIQUENESS_WEIGHT: f64 = 0.25; // 25% - Password uniqueness
const AGE_WEIGHT: f64 = 0.20; // 20% - Password age
const BREACH_WEIGHT: f64 = 0.15; // 15% - Breach status

/// Age thresholds in days
const FRESH_THRESHOLD: i64 = 90;   // 3 months
const OLD_THRESHOLD: i64 = 365;    // 1 year
const ANCIENT_THRESHOLD: i64 = 730; // 2 years

/// Calculate overall security score for the vault
pub fn calculate_vault_security_score(
    entries: &[PasswordEntry],
    analyses: &HashMap<uuid::Uuid, PasswordAnalysis>,
    reused_groups: &[ReusedPasswordGroup],
    breach_results: &HashMap<uuid::Uuid, BreachCheckResult>,
) -> Result<SecurityScore> {
    if entries.is_empty() {
        return Ok(SecurityScore {
            total_score: 100, // Empty vault is technically secure
            strength_score: 100,
            uniqueness_score: 100,
            age_score: 100,
            breach_score: 100,
            recommendations: vec!["Add passwords to start building your vault".to_string()],
            weak_passwords: 0,
            reused_passwords: 0,
            breached_passwords: 0,
            old_passwords: 0,
        });
    }

    // Calculate individual component scores
    let strength_score = calculate_strength_score(entries, analyses)?;
    let uniqueness_score = calculate_uniqueness_score(entries, reused_groups)?;
    let age_score = calculate_age_score(entries)?;
    let breach_score = calculate_breach_score(entries, breach_results)?;

    // Calculate weighted total score
    let total_score = (
        strength_score as f64 * STRENGTH_WEIGHT +
        uniqueness_score as f64 * UNIQUENESS_WEIGHT +
        age_score as f64 * AGE_WEIGHT +
        breach_score as f64 * BREACH_WEIGHT
    ).round() as u8;

    // Count various issues
    let weak_passwords = count_weak_passwords(analyses);
    let reused_passwords = count_reused_passwords(reused_groups);
    let breached_passwords = count_breached_passwords(breach_results);
    let old_passwords = count_old_passwords(entries);

    // Generate recommendations
    let recommendations = generate_recommendations(
        total_score,
        strength_score,
        uniqueness_score,
        age_score,
        breach_score,
        weak_passwords,
        reused_passwords,
        breached_passwords,
        old_passwords,
        entries.len(),
    );

    Ok(SecurityScore {
        total_score,
        strength_score,
        uniqueness_score,
        age_score,
        breach_score,
        recommendations,
        weak_passwords,
        reused_passwords,
        breached_passwords,
        old_passwords,
    })
}

/// Calculate password strength component score
fn calculate_strength_score(
    entries: &[PasswordEntry],
    analyses: &HashMap<uuid::Uuid, PasswordAnalysis>,
) -> Result<u8> {
    if entries.is_empty() {
        return Ok(100);
    }

    let mut total_strength = 0u32;
    let mut analyzed_count = 0;

    for entry in entries {
        if let Some(analysis) = analyses.get(&entry.id) {
            total_strength += analysis.score as u32;
            analyzed_count += 1;
        }
    }

    if analyzed_count == 0 {
        return Ok(50); // Neutral score if no analyses available
    }

    let average_strength = total_strength / analyzed_count as u32;
    Ok(average_strength as u8)
}

/// Calculate password uniqueness component score
fn calculate_uniqueness_score(
    entries: &[PasswordEntry],
    reused_groups: &[ReusedPasswordGroup],
) -> Result<u8> {
    if entries.is_empty() {
        return Ok(100);
    }

    let total_entries = entries.len() as f64;
    let mut reused_entries = 0;
    let mut high_risk_reused = 0;

    for group in reused_groups {
        reused_entries += group.entries.len();
        
        // Apply additional penalty for high-risk reuse
        match group.risk_level {
            RiskLevel::Critical => high_risk_reused += group.entries.len() * 3,
            RiskLevel::High => high_risk_reused += group.entries.len() * 2,
            RiskLevel::Medium => high_risk_reused += group.entries.len(),
            RiskLevel::Low => {} // No additional penalty
        }
    }

    // Calculate uniqueness percentage
    let unique_entries = total_entries - reused_entries as f64;
    let base_uniqueness = (unique_entries / total_entries * 100.0) as u8;

    // Apply risk penalty
    let risk_penalty = ((high_risk_reused as f64 / total_entries) * 50.0) as u8;
    let final_score = base_uniqueness.saturating_sub(risk_penalty);

    Ok(final_score.max(0))
}

/// Calculate password age component score
fn calculate_age_score(entries: &[PasswordEntry]) -> Result<u8> {
    if entries.is_empty() {
        return Ok(100);
    }

    let now = chrono::Utc::now();
    let mut age_scores = Vec::new();

    for entry in entries {
        let age_days = (now - entry.created_at).num_days();
        
        let age_score = match age_days {
            ..FRESH_THRESHOLD => 100,
            FRESH_THRESHOLD..OLD_THRESHOLD => {
                // Linear decay from 100 to 70 over 9 months
                let decay_days = age_days - FRESH_THRESHOLD;
                let decay_range = OLD_THRESHOLD - FRESH_THRESHOLD;
                let penalty = ((decay_days * 30) / decay_range) as u8;
                100u8.saturating_sub(penalty)
            },
            OLD_THRESHOLD..ANCIENT_THRESHOLD => {
                // Linear decay from 70 to 40 over 1 year  
                let decay_days = age_days - OLD_THRESHOLD;
                let decay_range = ANCIENT_THRESHOLD - OLD_THRESHOLD;
                let penalty = ((decay_days * 30) / decay_range) as u8;
                70u8.saturating_sub(penalty)
            },
            _ => 40, // Very old passwords get minimum score
        };

        age_scores.push(age_score);
    }

    let total_score = age_scores.iter().map(|&score| score as u32).sum::<u32>();
    let average_age_score = (total_score / age_scores.len() as u32) as u8;
    Ok(average_age_score)
}

/// Calculate breach status component score
fn calculate_breach_score(
    entries: &[PasswordEntry],
    breach_results: &HashMap<uuid::Uuid, BreachCheckResult>,
) -> Result<u8> {
    if entries.is_empty() {
        return Ok(100);
    }

    let total_entries = entries.len() as f64;
    let mut breach_penalty = 0.0;

    for entry in entries {
        if let Some(breach_result) = breach_results.get(&entry.id) {
            if breach_result.is_breached {
                if let Some(count) = breach_result.breach_count {
                    // Higher breach count means higher penalty
                    let penalty = match count {
                        1..=10 => 20.0,
                        11..=100 => 30.0,
                        101..=1000 => 40.0,
                        _ => 50.0, // Massively breached passwords
                    };
                    breach_penalty += penalty;
                }
            }
        }
    }

    let average_penalty = breach_penalty / total_entries;
    let final_score = (100.0 - average_penalty).max(0.0) as u8;
    Ok(final_score)
}

/// Count weak passwords (score < 50)
fn count_weak_passwords(analyses: &HashMap<uuid::Uuid, PasswordAnalysis>) -> usize {
    analyses.values().filter(|analysis| analysis.score < 50).count()
}

/// Count total reused passwords
fn count_reused_passwords(reused_groups: &[ReusedPasswordGroup]) -> usize {
    reused_groups.iter().map(|group| group.entries.len()).sum()
}

/// Count breached passwords
fn count_breached_passwords(breach_results: &HashMap<uuid::Uuid, BreachCheckResult>) -> usize {
    breach_results.values().filter(|result| result.is_breached).count()
}

/// Count old passwords (> 1 year)
fn count_old_passwords(entries: &[PasswordEntry]) -> usize {
    let now = chrono::Utc::now();
    entries.iter()
        .filter(|entry| (now - entry.created_at).num_days() > OLD_THRESHOLD)
        .count()
}

/// Generate actionable recommendations based on security analysis
fn generate_recommendations(
    total_score: u8,
    strength_score: u8,
    uniqueness_score: u8,
    age_score: u8,
    breach_score: u8,
    weak_passwords: usize,
    reused_passwords: usize,
    breached_passwords: usize,
    old_passwords: usize,
    total_passwords: usize,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Overall security status
    match total_score {
        90..=100 => recommendations.push("🟢 Excellent security! Keep up the good work.".to_string()),
        70..=89 => recommendations.push("🟡 Good security with room for improvement.".to_string()),
        50..=69 => recommendations.push("🟠 Moderate security - several issues need attention.".to_string()),
        _ => recommendations.push("🔴 Poor security - immediate action required!".to_string()),
    }

    // Strength-specific recommendations
    if strength_score < 70 {
        if weak_passwords > 0 {
            recommendations.push(format!(
                "💪 Strengthen {} weak password{} (use 12+ characters with mixed case, numbers, and symbols)",
                weak_passwords,
                if weak_passwords == 1 { "" } else { "s" }
            ));
        }
        
        if strength_score < 50 {
            recommendations.push("🎯 Priority: Replace passwords scoring below 50/100".to_string());
        }
    }

    // Uniqueness-specific recommendations  
    if uniqueness_score < 80 {
        if reused_passwords > 0 {
            recommendations.push(format!(
                "🔄 Create unique passwords for {} reused password{}", 
                reused_passwords,
                if reused_passwords == 1 { "" } else { "s" }
            ));
        }
        
        if uniqueness_score < 50 {
            recommendations.push("⚠️ Critical: Stop reusing passwords, especially for important accounts".to_string());
        }
    }

    // Age-specific recommendations
    if age_score < 70 {
        if old_passwords > 0 {
            recommendations.push(format!(
                "📅 Update {} password{} older than 1 year",
                old_passwords,
                if old_passwords == 1 { "" } else { "s" }
            ));
        }
        
        if age_score < 50 {
            recommendations.push("⏰ Establish a password rotation schedule (6-12 months)".to_string());
        }
    }

    // Breach-specific recommendations
    if breach_score < 90 {
        if breached_passwords > 0 {
            recommendations.push(format!(
                "🚨 URGENT: Change {} breached password{} immediately!",
                breached_passwords,
                if breached_passwords == 1 { "" } else { "s" }
            ));
        }
    }

    // General improvement suggestions
    if total_score < 80 {
        recommendations.push("💡 Consider using a password generator for new passwords".to_string());
        recommendations.push("📚 Learn about creating strong, memorable passphrases".to_string());
    }

    // Positive reinforcement
    if total_score >= 80 && recommendations.len() == 1 {
        recommendations.push("✨ Consider enabling breach monitoring alerts".to_string());
        recommendations.push("🎯 Set up regular security reviews (quarterly)".to_string());
    }

    // Small vault recommendations
    if total_passwords < 5 {
        recommendations.push("📈 Great start! Add more accounts to build a comprehensive security profile".to_string());
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    fn create_test_entry(id: &str, created_days_ago: i64) -> PasswordEntry {
        PasswordEntry {
            id: uuid::Uuid::new_v4(),
            title: format!("Test {}", id),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            url: Some("https://example.com".to_string()),
            notes: Some("".to_string()),
            tags: vec![],
            created_at: Utc::now() - chrono::Duration::days(created_days_ago),
            updated_at: Utc::now() - chrono::Duration::days(created_days_ago),
        }
    }

    #[test]
    fn test_empty_vault_score() {
        let entries = vec![];
        let analyses = HashMap::new();
        let reused_groups = vec![];
        let breach_results = HashMap::new();

        let score = calculate_vault_security_score(&entries, &analyses, &reused_groups, &breach_results).unwrap();
        assert_eq!(score.total_score, 100);
        assert!(!score.recommendations.is_empty());
    }

    #[test]
    fn test_age_score_calculation() {
        let entries = vec![
            create_test_entry("1", 30),   // Fresh
            create_test_entry("2", 180),  // Medium
            create_test_entry("3", 400),  // Old
        ];

        let age_score = calculate_age_score(&entries).unwrap();
        assert!(age_score > 60);
        assert!(age_score < 100);
    }

    #[test] 
    fn test_strength_score_calculation() {
        let entry1 = create_test_entry("1", 30);
        let entry2 = create_test_entry("2", 30);
        let entries = vec![entry1.clone(), entry2.clone()];

        let mut analyses = HashMap::new();
        analyses.insert(entry1.id, PasswordAnalysis {
            score: 80,
            crack_times: crate::password_health::CrackTimes {
                online_throttled: 0.0,
                online_unthrottled: 0.0,
                offline_slow: 0.0,
                offline_fast: 0.0,
            },
            feedback: vec![],
            warnings: vec![],
            patterns: vec![],
        });
        analyses.insert(entry2.id, PasswordAnalysis {
            score: 60,
            crack_times: crate::password_health::CrackTimes {
                online_throttled: 0.0,
                online_unthrottled: 0.0,
                offline_slow: 0.0,
                offline_fast: 0.0,
            },
            feedback: vec![],
            warnings: vec![],
            patterns: vec![],
        });

        let strength_score = calculate_strength_score(&entries, &analyses).unwrap();
        assert_eq!(strength_score, 70); // Average of 80 and 60
    }

    #[test]
    fn test_uniqueness_score_with_reuse() {
        let entry1 = create_test_entry("1", 30);
        let entry2 = create_test_entry("2", 30);
        let entry3 = create_test_entry("3", 30);
        let entries = vec![entry1.clone(), entry2.clone(), entry3.clone()];

        let reused_groups = vec![
            ReusedPasswordGroup {
                password_hash: "hash1".to_string(),
                entries: vec![entry1.id, entry2.id],
                risk_level: RiskLevel::Medium,
            }
        ];

        let uniqueness_score = calculate_uniqueness_score(&entries, &reused_groups).unwrap();
        assert!(uniqueness_score < 100);
    }

    #[test]
    fn test_breach_score_calculation() {
        let entry1 = create_test_entry("1", 30);
        let entry2 = create_test_entry("2", 30);
        let entries = vec![entry1.clone(), entry2.clone()];

        let mut breach_results = HashMap::new();
        breach_results.insert(entry1.id, BreachCheckResult {
            is_breached: true,
            breach_count: Some(50),
            last_checked: Utc::now(),
        });
        breach_results.insert(entry2.id, BreachCheckResult {
            is_breached: false,
            breach_count: None,
            last_checked: Utc::now(),
        });

        let breach_score = calculate_breach_score(&entries, &breach_results).unwrap();
        assert!(breach_score < 100);
        assert!(breach_score > 50);
    }
}