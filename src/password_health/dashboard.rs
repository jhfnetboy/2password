//! Password Health Dashboard UI and reporting
//!
//! Provides text-based dashboard visualization and reporting capabilities
//! for password health metrics and security analysis.

use crate::password_health::{DashboardData, SecurityScore, ReusedPasswordGroup, RiskLevel};
use crate::{Result, TwoPasswordError};
use std::collections::HashMap;

/// Generate a comprehensive dashboard report as formatted text
pub fn generate_dashboard_report(data: &DashboardData) -> Result<String> {
    let mut report = String::new();
    
    // Header
    report.push_str("╭─────────────────────────────────────────╮\n");
    report.push_str("│     🔐 PASSWORD HEALTH DASHBOARD       │\n");
    report.push_str(&format!("│     Generated: {}   │\n", data.generated_at.format("%Y-%m-%d %H:%M UTC")));
    report.push_str("╰─────────────────────────────────────────╯\n\n");

    // Security Score Overview
    add_security_overview(&mut report, &data.security_score);
    
    // Detailed Metrics
    add_detailed_metrics(&mut report, data);
    
    // Reused Passwords Section
    add_reused_passwords_section(&mut report, &data.reused_groups);
    
    // Breach Results Section
    add_breach_results_section(&mut report, data);
    
    // Age Distribution Section
    add_age_distribution_section(&mut report, &data.age_distribution);
    
    // Recommendations Section
    add_recommendations_section(&mut report, &data.security_score.recommendations);

    Ok(report)
}

/// Add security score overview section
fn add_security_overview(report: &mut String, score: &SecurityScore) {
    report.push_str("📊 SECURITY SCORE OVERVIEW\n");
    report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let status_icon = match score.total_score {
        90..=100 => "🟢",
        70..=89 => "🟡", 
        50..=69 => "🟠",
        _ => "🔴",
    };
    
    report.push_str(&format!("Overall Security: {} {}/100\n\n", status_icon, score.total_score));
    
    // Component scores with progress bars
    add_score_bar(report, "Password Strength", score.strength_score);
    add_score_bar(report, "Uniqueness     ", score.uniqueness_score);
    add_score_bar(report, "Password Age   ", score.age_score);
    add_score_bar(report, "Breach Status  ", score.breach_score);
    
    report.push('\n');
}

/// Add a score progress bar
fn add_score_bar(report: &mut String, label: &str, score: u8) {
    let bar_length = 20;
    let filled = (score as usize * bar_length) / 100;
    let empty = bar_length - filled;
    
    let color = match score {
        90..=100 => "🟩",
        70..=89 => "🟨",
        50..=69 => "🟧", 
        _ => "🟥",
    };
    
    let bar = format!("{}{}",
        color.repeat(filled),
        "⬜".repeat(empty)
    );
    
    report.push_str(&format!("{}: {} {}%\n", label, bar, score));
}

/// Add detailed metrics section
fn add_detailed_metrics(report: &mut String, data: &DashboardData) {
    report.push_str("📈 DETAILED METRICS\n");
    report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let total_passwords = data.password_analyses.len();
    
    report.push_str(&format!("Total Passwords:     {}\n", total_passwords));
    report.push_str(&format!("Weak Passwords:      {} 💪\n", data.security_score.weak_passwords));
    report.push_str(&format!("Reused Passwords:    {} 🔄\n", data.security_score.reused_passwords));
    report.push_str(&format!("Breached Passwords:  {} 🚨\n", data.security_score.breached_passwords));
    report.push_str(&format!("Old Passwords:       {} 📅\n", data.security_score.old_passwords));
    
    // Calculate some additional metrics
    let strong_passwords = data.password_analyses.values()
        .filter(|analysis| analysis.score >= 75)
        .count();
    let very_weak_passwords = data.password_analyses.values()
        .filter(|analysis| analysis.score < 25)
        .count();
        
    report.push_str(&format!("Strong Passwords:    {} ✨\n", strong_passwords));
    report.push_str(&format!("Very Weak Passwords: {} ⚠️\n", very_weak_passwords));
    
    report.push('\n');
}

/// Add reused passwords section
fn add_reused_passwords_section(report: &mut String, reused_groups: &[ReusedPasswordGroup]) {
    report.push_str("🔄 REUSED PASSWORDS\n");
    report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    if reused_groups.is_empty() {
        report.push_str("✅ No reused passwords found - excellent!\n");
    } else {
        report.push_str(&format!("Found {} password group{} with reuse:\n\n",
                reused_groups.len(),
                if reused_groups.len() == 1 { "" } else { "s" }));
        
        for (i, group) in reused_groups.iter().enumerate() {
            let risk_icon = match group.risk_level {
                RiskLevel::Critical => "🚨",
                RiskLevel::High => "⚠️",
                RiskLevel::Medium => "🟡", 
                RiskLevel::Low => "🟢",
            };
            
            report.push_str(&format!("{}. {} {:?} Risk - {} sites affected\n",
                    i + 1, risk_icon, group.risk_level, group.entries.len()));
            
            // Show first few entry IDs (don't expose sensitive info)
            let preview_count = 3.min(group.entries.len());
            for j in 0..preview_count {
                report.push_str(&format!("   • Entry {}\n", group.entries[j]));
            }
            if group.entries.len() > preview_count {
                report.push_str(&format!("   • ... and {} more\n", group.entries.len() - preview_count));
            }
            report.push('\n');
        }
    }
    
    report.push('\n');
}

/// Add breach results section
fn add_breach_results_section(report: &mut String, data: &DashboardData) {
    report.push_str("🚨 BREACH STATUS\n");
    report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let breached_count = data.breach_results.values()
        .filter(|result| result.is_breached)
        .count();
        
    if breached_count == 0 {
        report.push_str("✅ No breached passwords detected - great job!\n");
    } else {
        report.push_str(&format!("⚠️ {} password{} found in known breaches:\n\n",
                breached_count,
                if breached_count == 1 { "" } else { "s" }));
        
        // Group breaches by severity
        let mut critical_breaches = Vec::new();
        let mut major_breaches = Vec::new();
        let mut minor_breaches = Vec::new();
        
        for (entry_id, breach_result) in &data.breach_results {
            if breach_result.is_breached {
                if let Some(count) = breach_result.breach_count {
                    match count {
                        1000.. => critical_breaches.push((entry_id.to_string(), count)),
                        100..=999 => major_breaches.push((entry_id.to_string(), count)),
                        _ => minor_breaches.push((entry_id.to_string(), count)),
                    }
                }
            }
        }
        
        if !critical_breaches.is_empty() {
            report.push_str("🚨 CRITICAL (1000+ breaches):\n");
            for (entry_id, count) in critical_breaches {
                report.push_str(&format!("   • Entry {} - {} breaches\n", entry_id, count));
            }
            report.push('\n');
        }
        
        if !major_breaches.is_empty() {
            report.push_str("⚠️ MAJOR (100-999 breaches):\n");
            for (entry_id, count) in major_breaches {
                report.push_str(&format!("   • Entry {} - {} breaches\n", entry_id, count));
            }
            report.push('\n');
        }
        
        if !minor_breaches.is_empty() {
            report.push_str("🟡 MINOR (1-99 breaches):\n");
            for (entry_id, count) in minor_breaches {
                report.push_str(&format!("   • Entry {} - {} breaches\n", entry_id, count));
            }
            report.push('\n');
        }
    }
    
    report.push('\n');
}

/// Add age distribution section
fn add_age_distribution_section(report: &mut String, age_distribution: &HashMap<String, usize>) {
    report.push_str("📅 PASSWORD AGE DISTRIBUTION\n");
    report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Sort age ranges in logical order
    let age_order = ["0-30 days", "31-90 days", "3-12 months", "1-2 years", "2+ years"];
    let total_passwords: usize = age_distribution.values().sum();
    
    if total_passwords == 0 {
        report.push_str("No password age data available\n");
    } else {
        for age_range in age_order.iter() {
            if let Some(&count) = age_distribution.get(*age_range) {
                let percentage = (count as f64 / total_passwords as f64) * 100.0;
                let bar_length = ((percentage / 100.0) * 20.0) as usize;
                let bar = "█".repeat(bar_length) + &"░".repeat(20 - bar_length);
                
                report.push_str(&format!("{:<12}: {} {:.1}% ({})\n",
                        age_range, bar, percentage, count));
            }
        }
    }
    
    report.push('\n');
}

/// Add recommendations section
fn add_recommendations_section(report: &mut String, recommendations: &[String]) {
    report.push_str("💡 RECOMMENDATIONS\n");
    report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    if recommendations.is_empty() {
        report.push_str("✨ No specific recommendations - your security looks great!\n");
    } else {
        for (i, recommendation) in recommendations.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, recommendation));
        }
    }
    
    report.push('\n');
    
    // Add footer
    report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    report.push_str("💡 TwoPassword - Keep your digital life secure\n");
}

/// Generate a summary one-liner for quick status
pub fn generate_summary_line(data: &DashboardData) -> String {
    let score = &data.security_score;
    let status = match score.total_score {
        90..=100 => "🟢 EXCELLENT",
        70..=89 => "🟡 GOOD",
        50..=69 => "🟠 NEEDS WORK",
        _ => "🔴 POOR",
    };
    
    format!("Security: {} ({}/100) | Weak: {} | Reused: {} | Breached: {} | Old: {}",
           status, score.total_score, score.weak_passwords, 
           score.reused_passwords, score.breached_passwords, score.old_passwords)
}

/// Export dashboard data as JSON for external tools
pub fn export_dashboard_json(data: &DashboardData) -> Result<String> {
    serde_json::to_string_pretty(data)
        .map_err(|e| TwoPasswordError::SerializationError(e))
}

/// Export security metrics as CSV for analysis
pub fn export_metrics_csv(data: &DashboardData) -> Result<String> {
    let mut csv = String::new();
    
    // Header
    csv.push_str("entry_id,strength_score,is_reused,is_breached,breach_count,age_days\n");
    
    // Data rows
    for (entry_id, analysis) in &data.password_analyses {
        let is_reused = data.reused_groups.iter()
            .any(|group| group.entries.contains(entry_id));
            
        let (is_breached, breach_count) = if let Some(breach_result) = data.breach_results.get(entry_id) {
            (breach_result.is_breached, breach_result.breach_count.unwrap_or(0))
        } else {
            (false, 0)
        };
        
        // We can't easily get entry age without the original entry, so we'll use 0 as placeholder
        let age_days = 0;
        
        csv.push_str(&format!("{},{},{},{},{},{}\n",
                entry_id, analysis.score, is_reused, is_breached, breach_count, age_days));
    }
    
    Ok(csv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::password_health::{PasswordAnalysis, CrackTimes, BreachCheckResult, SecurityScore};

    fn create_test_dashboard_data() -> DashboardData {
        let mut password_analyses = HashMap::new();
        password_analyses.insert(uuid::Uuid::new_v4(), PasswordAnalysis {
            score: 85,
            crack_times: CrackTimes {
                online_throttled: 1000.0,
                online_unthrottled: 100.0,
                offline_slow: 10.0,
                offline_fast: 1.0,
            },
            feedback: vec!["Good password!".to_string()],
            warnings: vec![],
            patterns: vec![],
        });

        let mut breach_results = HashMap::new();
        breach_results.insert(uuid::Uuid::new_v4(), BreachCheckResult {
            is_breached: false,
            breach_count: None,
            last_checked: chrono::Utc::now(),
        });

        let mut age_distribution = HashMap::new();
        age_distribution.insert("0-30 days".to_string(), 1);

        DashboardData {
            security_score: SecurityScore {
                total_score: 85,
                strength_score: 85,
                uniqueness_score: 100,
                age_score: 95,
                breach_score: 100,
                recommendations: vec!["Keep up the good work!".to_string()],
                weak_passwords: 0,
                reused_passwords: 0,
                breached_passwords: 0,
                old_passwords: 0,
            },
            password_analyses,
            reused_groups: vec![],
            breach_results,
            age_distribution,
            generated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_generate_dashboard_report() {
        let data = create_test_dashboard_data();
        let report = generate_dashboard_report(&data).unwrap();
        
        assert!(report.contains("PASSWORD HEALTH DASHBOARD"));
        assert!(report.contains("SECURITY SCORE OVERVIEW"));
        assert!(report.contains("85/100"));
        assert!(report.contains("RECOMMENDATIONS"));
    }

    #[test]
    fn test_generate_summary_line() {
        let data = create_test_dashboard_data();
        let summary = generate_summary_line(&data);
        
        assert!(summary.contains("🟡 GOOD"));
        assert!(summary.contains("85/100"));
    }

    #[test] 
    fn test_export_dashboard_json() {
        let data = create_test_dashboard_data();
        let json = export_dashboard_json(&data).unwrap();
        
        assert!(json.contains("security_score"));
        assert!(json.contains("password_analyses"));
    }

    #[test]
    fn test_export_metrics_csv() {
        let data = create_test_dashboard_data();
        let csv = export_metrics_csv(&data).unwrap();
        
        assert!(csv.contains("entry_id,strength_score"));
        assert!(csv.contains("85,false,false,0,0"));
    }
}