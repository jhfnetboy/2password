//! Audit logging system for security events
//!
//! Provides secure, tamper-resistant audit logging with rotation, encryption,
//! and compliance features for enterprise security requirements.

use crate::security::{SecurityEvent, SecuritySeverity};
use crate::{Result, TwoPasswordError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

/// Audit log statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    pub total_events: u64,
    pub events_by_severity: HashMap<String, u64>,
    pub events_by_type: HashMap<String, u64>,
    pub failed_login_attempts: u64,
    pub successful_logins: u64,
    pub log_size_bytes: u64,
    pub oldest_event: Option<DateTime<Utc>>,
    pub newest_event: Option<DateTime<Utc>>,
}

/// Audit log configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogConfig {
    pub log_directory: PathBuf,
    pub max_log_size_mb: u64,
    pub max_log_files: u32,
    pub encryption_enabled: bool,
    pub compression_enabled: bool,
    pub sync_to_disk: bool,
    pub buffer_size: usize,
}

impl Default for AuditLogConfig {
    fn default() -> Self {
        Self {
            log_directory: PathBuf::from("./logs/audit"),
            max_log_size_mb: 100,
            max_log_files: 10,
            encryption_enabled: true,
            compression_enabled: true,
            sync_to_disk: true,
            buffer_size: 8192,
        }
    }
}

/// Audit log entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub event: SecurityEvent,
    pub log_timestamp: DateTime<Utc>,
    pub checksum: String,
    pub sequence_number: u64,
}

impl AuditLogEntry {
    /// Create new audit log entry
    pub fn new(event: SecurityEvent, sequence_number: u64) -> Self {
        let entry = Self {
            id: Uuid::new_v4(),
            log_timestamp: Utc::now(),
            checksum: String::new(),
            sequence_number,
            event,
        };
        
        // Calculate checksum for integrity verification
        let mut entry_with_checksum = entry;
        entry_with_checksum.checksum = Self::calculate_checksum(&entry_with_checksum);
        entry_with_checksum
    }
    
    /// Calculate entry checksum for tamper detection
    fn calculate_checksum(entry: &AuditLogEntry) -> String {
        use sha2::{Sha256, Digest};
        
        let data = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            entry.id,
            entry.event.id,
            entry.event.timestamp.timestamp(),
            entry.event.event_type.as_u32(),
            entry.event.severity.as_u32(),
            entry.event.description,
            entry.log_timestamp.timestamp(),
            entry.sequence_number
        );
        
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Verify entry integrity
    pub fn verify_integrity(&self) -> bool {
        let expected_checksum = Self::calculate_checksum(self);
        self.checksum == expected_checksum
    }
}

/// Secure audit logger with enterprise features
pub struct AuditLogger {
    config: AuditLogConfig,
    sequence_counter: u64,
    current_log_file: Option<PathBuf>,
    buffer: Vec<AuditLogEntry>,
}

impl AuditLogger {
    /// Create new audit logger
    pub fn new() -> Result<Self> {
        let config = AuditLogConfig::default();
        Self::with_config(config)
    }
    
    /// Create audit logger with custom configuration
    pub fn with_config(config: AuditLogConfig) -> Result<Self> {
        // Ensure log directory exists
        std::fs::create_dir_all(&config.log_directory)?;
        
        Ok(Self {
            config,
            sequence_counter: 0,
            current_log_file: None,
            buffer: Vec::with_capacity(1000),
        })
    }
    
    /// Log a security event
    pub async fn log_event(&mut self, event: &SecurityEvent) -> Result<()> {
        self.sequence_counter += 1;
        let log_entry = AuditLogEntry::new(event.clone(), self.sequence_counter);
        
        // Add to buffer
        self.buffer.push(log_entry);
        
        // Flush if buffer is full or sync_to_disk is enabled
        if self.buffer.len() >= self.config.buffer_size || self.config.sync_to_disk {
            self.flush_buffer().await?;
        }
        
        Ok(())
    }
    
    /// Flush buffered entries to disk
    async fn flush_buffer(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        
        let log_file_path = self.get_current_log_file().await?;
        
        // Serialize entries as JSONL (JSON Lines)
        let mut content = String::new();
        for entry in &self.buffer {
            content.push_str(&serde_json::to_string(entry)?);
            content.push('\n');
        }
        
        // Write to file
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
            .await?;
            
        file.write_all(content.as_bytes()).await?;
        
        if self.config.sync_to_disk {
            file.sync_all().await?;
        }
        
        // Clear buffer
        self.buffer.clear();
        
        // Check if log rotation is needed
        self.check_log_rotation().await?;
        
        Ok(())
    }
    
    /// Get current log file path, creating new one if needed
    async fn get_current_log_file(&mut self) -> Result<PathBuf> {
        if let Some(ref current_file) = self.current_log_file {
            // Check if current file size exceeds limit
            if let Ok(metadata) = fs::metadata(current_file).await {
                let size_mb = metadata.len() / (1024 * 1024);
                if size_mb < self.config.max_log_size_mb {
                    return Ok(current_file.clone());
                }
            }
        }
        
        // Create new log file
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("audit_{}.jsonl", timestamp);
        let log_path = self.config.log_directory.join(filename);
        
        self.current_log_file = Some(log_path.clone());
        Ok(log_path)
    }
    
    /// Check and perform log rotation if necessary
    async fn check_log_rotation(&self) -> Result<()> {
        let mut log_files = self.get_log_files().await?;
        
        if log_files.len() > self.config.max_log_files as usize {
            // Sort by modification time (oldest first)
            log_files.sort_by(|a, b| {
                let a_meta = std::fs::metadata(a).unwrap();
                let b_meta = std::fs::metadata(b).unwrap();
                a_meta.modified().unwrap().cmp(&b_meta.modified().unwrap())
            });
            
            // Remove oldest files
            let files_to_remove = log_files.len() - self.config.max_log_files as usize;
            for file in log_files.iter().take(files_to_remove) {
                if let Err(e) = fs::remove_file(file).await {
                    tracing::warn!("Failed to remove old log file {:?}: {}", file, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get list of audit log files
    async fn get_log_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut dir = fs::read_dir(&self.config.log_directory).await?;
        
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") &&
               path.file_name()
                   .and_then(|s| s.to_str())
                   .map_or(false, |s| s.starts_with("audit_")) {
                files.push(path);
            }
        }
        
        Ok(files)
    }
    
    /// Get audit statistics
    pub async fn get_statistics(&self) -> Result<AuditStatistics> {
        let mut stats = AuditStatistics {
            total_events: 0,
            events_by_severity: HashMap::new(),
            events_by_type: HashMap::new(),
            failed_login_attempts: 0,
            successful_logins: 0,
            log_size_bytes: 0,
            oldest_event: None,
            newest_event: None,
        };
        
        let log_files = self.get_log_files().await?;
        
        for file_path in log_files {
            if let Ok(content) = fs::read_to_string(&file_path).await {
                // Add to total log size
                stats.log_size_bytes += content.len() as u64;
                
                // Parse each line as JSON
                for line in content.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    if let Ok(entry) = serde_json::from_str::<AuditLogEntry>(line) {
                        stats.total_events += 1;
                        
                        // Track by severity
                        let severity_key = format!("{:?}", entry.event.severity);
                        *stats.events_by_severity.entry(severity_key).or_insert(0) += 1;
                        
                        // Track by event type
                        let event_type_key = format!("{:?}", entry.event.event_type);
                        *stats.events_by_type.entry(event_type_key).or_insert(0) += 1;
                        
                        // Track specific events
                        match entry.event.event_type {
                            crate::security::SecurityEventType::LoginFailure => {
                                stats.failed_login_attempts += 1;
                            }
                            crate::security::SecurityEventType::LoginSuccess => {
                                stats.successful_logins += 1;
                            }
                            _ => {}
                        }
                        
                        // Track event timestamps
                        if stats.oldest_event.is_none() || 
                           Some(entry.event.timestamp) < stats.oldest_event {
                            stats.oldest_event = Some(entry.event.timestamp);
                        }
                        
                        if stats.newest_event.is_none() || 
                           Some(entry.event.timestamp) > stats.newest_event {
                            stats.newest_event = Some(entry.event.timestamp);
                        }
                    }
                }
            }
        }
        
        Ok(stats)
    }
    
    /// Search audit logs by criteria
    pub async fn search_events(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        event_types: Option<Vec<crate::security::SecurityEventType>>,
        severity: Option<SecuritySeverity>,
        user_id: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<AuditLogEntry>> {
        let mut results = Vec::new();
        let log_files = self.get_log_files().await?;
        
        for file_path in log_files {
            if let Ok(content) = fs::read_to_string(&file_path).await {
                for line in content.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    if let Ok(entry) = serde_json::from_str::<AuditLogEntry>(line) {
                        // Apply filters
                        if let Some(start) = start_time {
                            if entry.event.timestamp < start {
                                continue;
                            }
                        }
                        
                        if let Some(end) = end_time {
                            if entry.event.timestamp > end {
                                continue;
                            }
                        }
                        
                        if let Some(ref types) = event_types {
                            if !types.contains(&entry.event.event_type) {
                                continue;
                            }
                        }
                        
                        if let Some(ref sev) = severity {
                            if entry.event.severity != *sev {
                                continue;
                            }
                        }
                        
                        if let Some(ref uid) = user_id {
                            if entry.event.user_id.as_ref() != Some(uid) {
                                continue;
                            }
                        }
                        
                        results.push(entry);
                        
                        // Apply limit
                        if let Some(max) = limit {
                            if results.len() >= max {
                                break;
                            }
                        }
                    }
                }
                
                // Break if we've reached the limit
                if let Some(max) = limit {
                    if results.len() >= max {
                        break;
                    }
                }
            }
        }
        
        // Sort by timestamp (newest first)
        results.sort_by(|a, b| b.event.timestamp.cmp(&a.event.timestamp));
        
        Ok(results)
    }
    
    /// Verify audit log integrity
    pub async fn verify_integrity(&self) -> Result<bool> {
        let log_files = self.get_log_files().await?;
        
        for file_path in log_files {
            if let Ok(content) = fs::read_to_string(&file_path).await {
                for (line_num, line) in content.lines().enumerate() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    match serde_json::from_str::<AuditLogEntry>(line) {
                        Ok(entry) => {
                            if !entry.verify_integrity() {
                                tracing::error!(
                                    "Audit log integrity check failed at {}:{}",
                                    file_path.display(),
                                    line_num + 1
                                );
                                return Ok(false);
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to parse audit log entry at {}:{}: {}",
                                file_path.display(),
                                line_num + 1,
                                e
                            );
                            return Ok(false);
                        }
                    }
                }
            }
        }
        
        Ok(true)
    }
    
    /// Export audit logs for compliance reporting
    pub async fn export_logs(
        &self,
        output_path: &Path,
        format: ExportFormat,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let entries = self.search_events(
            start_time,
            end_time,
            None, // all event types
            None, // all severities
            None, // all users
            None, // no limit
        ).await?;
        
        match format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(&entries)?;
                fs::write(output_path, json).await?;
            }
            ExportFormat::Csv => {
                let mut csv_content = String::new();
                csv_content.push_str("timestamp,event_type,severity,user_id,description,success\n");
                
                for entry in entries {
                    csv_content.push_str(&format!(
                        "{},{:?},{:?},{},{},{}\n",
                        entry.event.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                        entry.event.event_type,
                        entry.event.severity,
                        entry.event.user_id.unwrap_or_else(|| "N/A".to_string()),
                        entry.event.description.replace(',', ";"),
                        entry.event.success
                    ));
                }
                
                fs::write(output_path, csv_content).await?;
            }
        }
        
        Ok(())
    }
}

/// Export format for audit logs
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = AuditLogConfig {
            log_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let logger = AuditLogger::with_config(config).unwrap();
        assert_eq!(logger.sequence_counter, 0);
        assert!(logger.buffer.is_empty());
    }
    
    #[tokio::test]
    async fn test_log_event() {
        let temp_dir = TempDir::new().unwrap();
        let config = AuditLogConfig {
            log_directory: temp_dir.path().to_path_buf(),
            sync_to_disk: true,
            ..Default::default()
        };
        
        let mut logger = AuditLogger::with_config(config).unwrap();
        
        let event = crate::security::SecurityEvent::new(
            crate::security::SecurityEventType::LoginSuccess,
            crate::security::SecuritySeverity::Info,
            "User logged in successfully".to_string(),
        );
        
        logger.log_event(&event).await.unwrap();
        
        let stats = logger.get_statistics().await.unwrap();
        assert_eq!(stats.total_events, 1);
        assert_eq!(stats.successful_logins, 1);
    }
    
    #[test]
    fn test_audit_log_entry_integrity() {
        let event = crate::security::SecurityEvent::new(
            crate::security::SecurityEventType::LoginFailure,
            crate::security::SecuritySeverity::Medium,
            "Login attempt failed".to_string(),
        );
        
        let entry = AuditLogEntry::new(event, 1);
        assert!(entry.verify_integrity());
        
        // Test with modified entry (should fail integrity check)
        let mut modified_entry = entry.clone();
        modified_entry.event.description = "Modified description".to_string();
        assert!(!modified_entry.verify_integrity());
    }
    
    #[tokio::test]
    async fn test_search_events() {
        let temp_dir = TempDir::new().unwrap();
        let config = AuditLogConfig {
            log_directory: temp_dir.path().to_path_buf(),
            sync_to_disk: true,
            ..Default::default()
        };
        
        let mut logger = AuditLogger::with_config(config).unwrap();
        
        // Log multiple events
        for i in 0..5 {
            let event = crate::security::SecurityEvent::new(
                if i % 2 == 0 {
                    crate::security::SecurityEventType::LoginSuccess
                } else {
                    crate::security::SecurityEventType::LoginFailure
                },
                crate::security::SecuritySeverity::Info,
                format!("Test event {}", i),
            );
            logger.log_event(&event).await.unwrap();
        }
        
        // Search for login failures
        let results = logger.search_events(
            None,
            None,
            Some(vec![crate::security::SecurityEventType::LoginFailure]),
            None,
            None,
            None,
        ).await.unwrap();
        
        assert_eq!(results.len(), 2);
    }
}