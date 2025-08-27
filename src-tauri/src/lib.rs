// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::Mutex;
use tauri::State;

// Import 2Password core functionality
use twopassword::storage::{VaultManager, PasswordEntry, SearchOptions};
use twopassword::import_export::{ImportExportService, ImportFormat, ImportResult, ExportOptions};
use twopassword::auth::touchid;
use twopassword::password_health::{PasswordHealthService, DashboardData};

// Application state that will be shared across Tauri commands  
pub struct AppState {
    vault_manager: Mutex<VaultManager>,
    import_export: Mutex<ImportExportService>,
    password_health: Mutex<PasswordHealthService>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            vault_manager: Mutex::new(VaultManager::new()),
            import_export: Mutex::new(ImportExportService::new()),
            password_health: Mutex::new(PasswordHealthService::new()),
        }
    }
}

// Simple greeting command for testing
#[tauri::command]
fn greet() -> String {
    "Hello from 2Password!".to_string()
}

// Vault management commands
#[derive(serde::Serialize)]
struct VaultStatus {
    loaded: bool,
    path: Option<String>,
}

#[tauri::command]
async fn create_vault(
    state: State<'_, AppState>,
    path: String,
    password: String,
) -> Result<bool, String> {
    let mut vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    vault_manager
        .create_vault(&path, &password)
        .map_err(|e| format!("Failed to create vault: {}", e))?;

    Ok(true)
}

#[tauri::command]
async fn load_vault(
    state: State<'_, AppState>,
    path: String,
    password: String,
) -> Result<bool, String> {
    let mut vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    vault_manager
        .load_vault(&path, &password)
        .map_err(|e| format!("Failed to load vault: {}", e))?;

    Ok(true)
}

#[tauri::command]
async fn get_vault_status(state: State<'_, AppState>) -> Result<VaultStatus, String> {
    let vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    Ok(VaultStatus {
        loaded: vault_manager.is_vault_loaded(),
        path: None, // TODO: Add path tracking
    })
}

#[tauri::command]
async fn is_vault_loaded(state: State<'_, AppState>) -> Result<bool, String> {
    let vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    Ok(vault_manager.is_vault_loaded())
}

#[tauri::command]
async fn get_all_entries(
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(vault) = vault_manager.get_vault() {
        let entries = vault.get_all_entries();
        let json_entries: Vec<serde_json::Value> = entries
            .iter()
            .map(|entry| serde_json::to_value(entry))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to serialize entries: {}", e))?;
        Ok(json_entries)
    } else {
        Err("No vault loaded".to_string())
    }
}

#[tauri::command]
async fn add_entry(
    state: State<'_, AppState>,
    title: String,
    username: String,
    password: String,
    url: Option<String>,
    notes: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<String, String> {
    let mut vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(vault) = vault_manager.get_vault_mut() {
        let entry = PasswordEntry::new_with_fields(
            title, 
            username, 
            password, 
            url, 
            notes, 
            tags.unwrap_or_default()
        );
        let entry_id = entry.id.to_string();
        vault.add_entry(entry);
        Ok(entry_id)
    } else {
        Err("No vault loaded".to_string())
    }
}

#[tauri::command]
async fn check_touchid_available() -> Result<bool, String> {
    Ok(touchid::is_available())
}

#[tauri::command]
async fn authenticate_touchid(reason: String) -> Result<bool, String> {
    touchid::authenticate(&reason)
        .map_err(|e| format!("Touch ID authentication failed: {}", e))
}

#[tauri::command]
async fn remove_entry(state: State<'_, AppState>, entry_id: String) -> Result<bool, String> {
    let mut vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(vault) = vault_manager.get_vault_mut() {
        let id = uuid::Uuid::parse_str(&entry_id)
            .map_err(|e| format!("Invalid UUID: {}", e))?;
        vault
            .remove_entry(&id)
            .map_err(|e| format!("Failed to remove entry: {}", e))?;
        Ok(true)
    } else {
        Err("No vault loaded".to_string())
    }
}

#[tauri::command]
async fn search_entries(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<serde_json::Value>, String> {
    let vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(vault) = vault_manager.get_vault() {
        let entries = vault.search_by_title(&query);
        let json_entries: Vec<serde_json::Value> = entries
            .iter()
            .map(|entry| serde_json::to_value(entry))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to serialize entries: {}", e))?;
        Ok(json_entries)
    } else {
        Err("No vault loaded".to_string())
    }
}

#[tauri::command]
async fn save_vault(state: State<'_, AppState>) -> Result<bool, String> {
    let mut vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    vault_manager
        .save_vault()
        .map_err(|e| format!("Failed to save vault: {}", e))?;

    Ok(true)
}

#[tauri::command]
async fn close_vault(state: State<'_, AppState>) -> Result<bool, String> {
    let mut vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    vault_manager.close_vault();
    Ok(true)
}

// Generate secure password
#[tauri::command]
async fn generate_password(
    length: Option<u32>,
    include_symbols: Option<bool>,
    include_numbers: Option<bool>,
    include_uppercase: Option<bool>,
    include_lowercase: Option<bool>,
) -> Result<String, String> {
    use twopassword::storage::entry::generate_password as gen_password;
    
    gen_password(
        length.unwrap_or(16) as usize,
        include_symbols.unwrap_or(true),
        include_numbers.unwrap_or(true),
        include_uppercase.unwrap_or(true),
        include_lowercase.unwrap_or(true),
    )
    .map_err(|e| format!("Failed to generate password: {}", e))
}

// Import/Export commands

// Detect file format
#[tauri::command]
async fn detect_import_format(
    state: State<'_, AppState>,
    filename: String,
    content: String,
) -> Result<String, String> {
    let import_export = state
        .import_export
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    match import_export.detect_format(&filename, &content) {
        Ok(format) => Ok(format!("{:?}", format)),
        Err(e) => Err(format!("Format detection failed: {}", e)),
    }
}

// Import passwords from file
#[derive(serde::Deserialize)]
struct ImportRequest {
    content: String,
    format: String,
    duplicate_check: bool,
}

#[tauri::command]
async fn import_passwords(
    state: State<'_, AppState>,
    request: ImportRequest,
) -> Result<serde_json::Value, String> {
    let mut vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire vault lock: {}", e))?;

    let import_export = state
        .import_export
        .lock()
        .map_err(|e| format!("Failed to acquire import lock: {}", e))?;

    if let Some(vault) = vault_manager.get_vault() {
        // Parse format
        let format = match request.format.as_str() {
            "CSV" => ImportFormat::CSV,
            "LastPass" => ImportFormat::LastPass,
            "Chrome" => ImportFormat::Chrome,
            "Bitwarden" => ImportFormat::Bitwarden,
            "Firefox" => ImportFormat::Firefox,
            "OnePassword" => ImportFormat::OnePassword,
            _ => return Err(format!("Unsupported format: {}", request.format)),
        };

        // Get existing entries for duplicate check
        let existing_entries: Vec<PasswordEntry> = vault.get_all_entries().into_iter().cloned().collect();
        
        // Import passwords
        match import_export.import_passwords(&request.content, format, &existing_entries) {
            Ok((entries, result)) => {
                // Add entries to vault if successful
                if !entries.is_empty() {
                    drop(import_export); // Release lock before modifying vault
                    if let Some(vault) = vault_manager.get_vault_mut() {
                        for entry in entries {
                            vault.add_entry(entry);
                        }
                    }
                }
                
                Ok(serde_json::to_value(result)
                    .map_err(|e| format!("Failed to serialize result: {}", e))?)
            }
            Err(e) => Err(format!("Import failed: {}", e)),
        }
    } else {
        Err("No vault loaded".to_string())
    }
}

// Export passwords to specified format
#[derive(serde::Deserialize)]
struct ExportRequest {
    format: String,
    include_passwords: bool,
    include_notes: bool,
    include_tags: bool,
    include_metadata: bool,
}

#[tauri::command]
async fn export_passwords(
    state: State<'_, AppState>,
    request: ExportRequest,
) -> Result<String, String> {
    let vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire vault lock: {}", e))?;

    let import_export = state
        .import_export
        .lock()
        .map_err(|e| format!("Failed to acquire export lock: {}", e))?;

    if let Some(vault) = vault_manager.get_vault() {
        // Parse format
        let format = match request.format.as_str() {
            "CSV" => ImportFormat::CSV,
            "Bitwarden" => ImportFormat::Bitwarden,
            _ => return Err(format!("Unsupported export format: {}", request.format)),
        };

        let options = ExportOptions {
            format,
            include_passwords: request.include_passwords,
            include_notes: request.include_notes,
            include_tags: request.include_tags,
            include_metadata: request.include_metadata,
        };

        let entries: Vec<PasswordEntry> = vault.get_all_entries().into_iter().cloned().collect();
        
        match import_export.export_passwords(&entries, &options) {
            Ok(exported_data) => Ok(exported_data),
            Err(e) => Err(format!("Export failed: {}", e)),
        }
    } else {
        Err("No vault loaded".to_string())
    }
}

// Get supported import formats
#[tauri::command]
async fn get_supported_formats() -> Result<Vec<serde_json::Value>, String> {
    let formats = vec![
        serde_json::json!({
            "name": "CSV",
            "description": "Generic CSV format",
            "extensions": ["csv"],
            "import": true,
            "export": true
        }),
        serde_json::json!({
            "name": "LastPass",
            "description": "LastPass CSV export",
            "extensions": ["csv"],
            "import": true,
            "export": false
        }),
        serde_json::json!({
            "name": "Chrome",
            "description": "Chrome passwords CSV",
            "extensions": ["csv"],
            "import": true,
            "export": false
        }),
        serde_json::json!({
            "name": "Bitwarden",
            "description": "Bitwarden JSON export",
            "extensions": ["json"],
            "import": true,
            "export": true
        }),
        serde_json::json!({
            "name": "Firefox",
            "description": "Firefox JSON export",
            "extensions": ["json"],
            "import": true,
            "export": false
        }),
        serde_json::json!({
            "name": "OnePassword",
            "description": "1Password export (basic)",
            "extensions": ["1pif", "csv"],
            "import": true,
            "export": false
        }),
    ];

    Ok(formats)
}

// Advanced search command
#[derive(serde::Deserialize)]
struct AdvancedSearchRequest {
    query: Option<String>,
    tags: Option<Vec<String>>,
    created_after: Option<String>,
    created_before: Option<String>,
}

#[tauri::command]
async fn advanced_search(
    state: State<'_, AppState>,
    request: AdvancedSearchRequest,
) -> Result<Vec<serde_json::Value>, String> {
    let vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(vault) = vault_manager.get_vault() {
        let mut search_options = SearchOptions::default();
        search_options.query = request.query;
        search_options.tags = request.tags.unwrap_or_default();

        // Parse date strings if provided
        if let Some(after_str) = request.created_after {
            search_options.created_after = chrono::DateTime::parse_from_rfc3339(&after_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .ok();
        }
        
        if let Some(before_str) = request.created_before {
            search_options.created_before = chrono::DateTime::parse_from_rfc3339(&before_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .ok();
        }

        let entries = vault.advanced_search(&search_options);
        let json_entries: Vec<serde_json::Value> = entries
            .iter()
            .map(|entry| serde_json::to_value(entry))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to serialize entries: {}", e))?;
        Ok(json_entries)
    } else {
        Err("No vault loaded".to_string())
    }
}

// Get all tags command
#[tauri::command]
async fn get_all_tags(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(vault) = vault_manager.get_vault() {
        Ok(vault.get_all_tags())
    } else {
        Err("No vault loaded".to_string())
    }
}

// Add tag to entry command
#[tauri::command]
async fn add_tag_to_entry(
    state: State<'_, AppState>,
    entry_id: String,
    tag: String,
) -> Result<bool, String> {
    let mut vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(vault) = vault_manager.get_vault_mut() {
        let id = uuid::Uuid::parse_str(&entry_id)
            .map_err(|e| format!("Invalid UUID: {}", e))?;
        
        if let Some(entry) = vault.get_entry_mut(&id) {
            entry.add_tag(tag);
            Ok(true)
        } else {
            Err("Entry not found".to_string())
        }
    } else {
        Err("No vault loaded".to_string())
    }
}

// Remove tag from entry command
#[tauri::command]
async fn remove_tag_from_entry(
    state: State<'_, AppState>,
    entry_id: String,
    tag: String,
) -> Result<bool, String> {
    let mut vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(vault) = vault_manager.get_vault_mut() {
        let id = uuid::Uuid::parse_str(&entry_id)
            .map_err(|e| format!("Invalid UUID: {}", e))?;
        
        if let Some(entry) = vault.get_entry_mut(&id) {
            entry.remove_tag(&tag);
            Ok(true)
        } else {
            Err("Entry not found".to_string())
        }
    } else {
        Err("No vault loaded".to_string())
    }
}

// Password Health Dashboard commands

// Generate comprehensive password health dashboard
#[tauri::command]
async fn generate_password_health_dashboard(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    // Extract entries first, then release the lock
    let entries: Vec<PasswordEntry> = {
        let vault_manager = state
            .vault_manager
            .lock()
            .map_err(|e| format!("Failed to acquire vault lock: {}", e))?;

        if let Some(vault) = vault_manager.get_vault() {
            vault.get_all_entries().into_iter().cloned().collect()
        } else {
            return Err("No vault loaded".to_string());
        }
    }; // vault_manager lock released here

    // Clone the PasswordHealthService to avoid holding the lock
    let password_health_service = {
        let password_health = state
            .password_health
            .lock()
            .map_err(|e| format!("Failed to acquire health lock: {}", e))?;
            
        // We need to clone or take ownership to avoid the Send issue
        // For now, let's create a new instance
        twopassword::password_health::PasswordHealthService::new()
    };

    // Now we can await without holding any locks
    let dashboard_data = password_health_service.generate_dashboard(&entries).await
        .map_err(|e| format!("Failed to generate dashboard: {}", e))?;

    serde_json::to_value(dashboard_data)
        .map_err(|e| format!("Failed to serialize dashboard: {}", e))
}

// Analyze single password strength
#[tauri::command]
async fn analyze_password_strength(
    state: State<'_, AppState>,
    password: String,
) -> Result<serde_json::Value, String> {
    // Create a new instance to avoid the Send issue
    let password_health_service = twopassword::password_health::PasswordHealthService::new();

    let analysis = password_health_service.analyze_password(&password)
        .map_err(|e| format!("Failed to analyze password: {}", e))?;

    serde_json::to_value(analysis)
        .map_err(|e| format!("Failed to serialize analysis: {}", e))
}

// Check password against breaches
#[tauri::command]
async fn check_password_breach(
    state: State<'_, AppState>,
    password: String,
) -> Result<serde_json::Value, String> {
    // Create a new instance to avoid the Send issue
    let password_health_service = twopassword::password_health::PasswordHealthService::new();

    let breach_result = password_health_service.check_breach(&password).await
        .map_err(|e| format!("Failed to check breach: {}", e))?;

    serde_json::to_value(breach_result)
        .map_err(|e| format!("Failed to serialize breach result: {}", e))
}

// Generate security summary report
#[tauri::command]
async fn generate_security_summary(
    state: State<'_, AppState>,
) -> Result<String, String> {
    use twopassword::password_health::dashboard::generate_summary_line;
    
    // Extract entries first, then release the lock
    let entries: Vec<PasswordEntry> = {
        let vault_manager = state
            .vault_manager
            .lock()
            .map_err(|e| format!("Failed to acquire vault lock: {}", e))?;

        if let Some(vault) = vault_manager.get_vault() {
            vault.get_all_entries().into_iter().cloned().collect()
        } else {
            return Err("No vault loaded".to_string());
        }
    };

    // Create a new instance and generate dashboard
    let password_health_service = twopassword::password_health::PasswordHealthService::new();
    let dashboard_data = password_health_service.generate_dashboard(&entries).await
        .map_err(|e| format!("Failed to generate summary: {}", e))?;

    Ok(generate_summary_line(&dashboard_data))
}

// Generate full dashboard report as text
#[tauri::command]
async fn generate_dashboard_report(
    state: State<'_, AppState>,
) -> Result<String, String> {
    use twopassword::password_health::dashboard::generate_dashboard_report;
    
    // Extract entries first, then release the lock
    let entries: Vec<PasswordEntry> = {
        let vault_manager = state
            .vault_manager
            .lock()
            .map_err(|e| format!("Failed to acquire vault lock: {}", e))?;

        if let Some(vault) = vault_manager.get_vault() {
            vault.get_all_entries().into_iter().cloned().collect()
        } else {
            return Err("No vault loaded".to_string());
        }
    };

    // Create a new instance and generate dashboard
    let password_health_service = twopassword::password_health::PasswordHealthService::new();
    let dashboard_data = password_health_service.generate_dashboard(&entries).await
        .map_err(|e| format!("Failed to generate dashboard: {}", e))?;

    generate_dashboard_report(&dashboard_data)
        .map_err(|e| format!("Failed to generate report: {}", e))
}

// Export dashboard data as JSON
#[tauri::command]
async fn export_dashboard_json(
    state: State<'_, AppState>,
) -> Result<String, String> {
    use twopassword::password_health::dashboard::export_dashboard_json;
    
    // Extract entries first, then release the lock
    let entries: Vec<PasswordEntry> = {
        let vault_manager = state
            .vault_manager
            .lock()
            .map_err(|e| format!("Failed to acquire vault lock: {}", e))?;

        if let Some(vault) = vault_manager.get_vault() {
            vault.get_all_entries().into_iter().cloned().collect()
        } else {
            return Err("No vault loaded".to_string());
        }
    };

    // Create a new instance and generate dashboard
    let password_health_service = twopassword::password_health::PasswordHealthService::new();
    let dashboard_data = password_health_service.generate_dashboard(&entries).await
        .map_err(|e| format!("Failed to generate dashboard: {}", e))?;

    export_dashboard_json(&dashboard_data)
        .map_err(|e| format!("Failed to export JSON: {}", e))
}

// Export security metrics as CSV
#[tauri::command]
async fn export_metrics_csv(
    state: State<'_, AppState>,
) -> Result<String, String> {
    use twopassword::password_health::dashboard::export_metrics_csv;
    
    // Extract entries first, then release the lock
    let entries: Vec<PasswordEntry> = {
        let vault_manager = state
            .vault_manager
            .lock()
            .map_err(|e| format!("Failed to acquire vault lock: {}", e))?;

        if let Some(vault) = vault_manager.get_vault() {
            vault.get_all_entries().into_iter().cloned().collect()
        } else {
            return Err("No vault loaded".to_string());
        }
    };

    // Create a new instance and generate dashboard
    let password_health_service = twopassword::password_health::PasswordHealthService::new();
    let dashboard_data = password_health_service.generate_dashboard(&entries).await
        .map_err(|e| format!("Failed to generate dashboard: {}", e))?;

    export_metrics_csv(&dashboard_data)
        .map_err(|e| format!("Failed to export CSV: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            create_vault,
            load_vault,
            get_vault_status,
            is_vault_loaded,
            get_all_entries,
            add_entry,
            remove_entry,
            search_entries,
            advanced_search,
            get_all_tags,
            add_tag_to_entry,
            remove_tag_from_entry,
            save_vault,
            close_vault,
            generate_password,
            detect_import_format,
            import_passwords,
            export_passwords,
            get_supported_formats,
            check_touchid_available,
            authenticate_touchid,
            generate_password_health_dashboard,
            analyze_password_strength,
            check_password_breach,
            generate_security_summary,
            generate_dashboard_report,
            export_dashboard_json,
            export_metrics_csv
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}