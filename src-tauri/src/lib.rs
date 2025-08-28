// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::Mutex;
use tauri::State;

// Import 2Password core functionality
use twopassword::storage::{VaultManager, PasswordEntry, SearchOptions};
use twopassword::import_export::{ImportExportService, ImportFormat, ImportResult, ExportOptions};
use twopassword::auth::{touchid, passkey::{PasskeyManager, PasskeyConfig}};
use twopassword::crypto::key_derivation::{MultiFactorInput, derive_master_key};
use twopassword::password_health::{PasswordHealthService, DashboardData};
use sha2::{Sha256, Digest};
use rand::{Rng, thread_rng};
use base64;

// Application state that will be shared across Tauri commands  
pub struct AppState {
    vault_manager: Mutex<VaultManager>,
    import_export: Mutex<ImportExportService>,
    password_health: Mutex<PasswordHealthService>,
    passkey_manager: Mutex<PasskeyManager>,
}

impl Default for AppState {
    fn default() -> Self {
        // Create default Passkey configuration
        let passkey_config = PasskeyConfig::default();
        let passkey_manager = PasskeyManager::new(passkey_config)
            .expect("Failed to create PasskeyManager");

        Self {
            vault_manager: Mutex::new(VaultManager::new()),
            import_export: Mutex::new(ImportExportService::new()),
            password_health: Mutex::new(PasswordHealthService::new()),
            passkey_manager: Mutex::new(passkey_manager),
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
    println!("🎯 create_vault command called with path: {}", path);
    println!("🔑 password length: {}", password.len());
    
    let mut vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| {
            println!("❌ Failed to acquire lock: {}", e);
            format!("Failed to acquire lock: {}", e)
        })?;

    println!("🔒 Successfully acquired vault manager lock");
    
    match vault_manager.create_vault(&path, &password) {
        Ok(_) => {
            println!("✅ Vault created successfully");
            Ok(true)
        }
        Err(e) => {
            println!("❌ Failed to create vault: {}", e);
            Err(format!("Failed to create vault: {}", e))
        }
    }
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
async fn close_vault(state: State<'_, AppState>) -> Result<bool, String> {
    println!("🔓 close_vault command called");
    
    let mut vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| {
            println!("❌ Failed to acquire lock: {}", e);
            format!("Failed to acquire lock: {}", e)
        })?;

    println!("🔒 Successfully acquired vault manager lock");
    
    // Save current vault before closing if it exists
    if vault_manager.is_vault_loaded() {
        println!("💾 Saving current vault before closing...");
        match vault_manager.save_vault() {
            Ok(_) => println!("✅ Vault saved successfully"),
            Err(e) => println!("⚠️ Warning: Failed to save vault: {}", e),
        }
    }
    
    // Close the vault
    vault_manager.close_vault();
    println!("✅ Vault closed successfully");
    
    Ok(true)
}

#[tauri::command]
async fn get_vault_status(state: State<'_, AppState>) -> Result<VaultStatus, String> {
    let vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let vault_path = if vault_manager.is_vault_loaded() {
        vault_manager.get_vault_path()
    } else {
        None
    };

    Ok(VaultStatus {
        loaded: vault_manager.is_vault_loaded(),
        path: vault_path,
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
    println!("🎯 add_entry command called with:");
    println!("   📝 title: {}", title);
    println!("   👤 username: {}", username);
    println!("   🔑 password: [PROTECTED {} chars]", password.len());
    println!("   🌐 url: {:?}", url);
    println!("   📄 notes: {:?}", notes.as_ref().map(|s| format!("{} chars", s.len())));
    println!("   🏷️ tags: {:?}", tags);
    
    println!("🔐 Acquiring vault manager lock...");
    let mut vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| {
            let error = format!("Failed to acquire lock: {}", e);
            println!("❌ Lock acquisition failed: {}", error);
            error
        })?;
    println!("✅ Vault manager lock acquired");

    if let Some(vault) = vault_manager.get_vault_mut() {
        println!("✅ Vault is loaded, creating new entry...");
        let entry = PasswordEntry::new_with_fields(
            title, 
            username, 
            password, 
            url, 
            notes, 
            tags.unwrap_or_default()
        );
        let entry_id = entry.id.to_string();
        println!("✅ Created entry with ID: {}", entry_id);
        
        println!("💾 Adding entry to vault...");
        vault.add_entry(entry);
        println!("✅ Entry added to vault");
        
        println!("💽 Saving vault to disk...");
        // 自动保存到磁盘
        vault_manager
            .save_vault()
            .map_err(|e| {
                let error = format!("Failed to save vault: {}", e);
                println!("❌ Vault save failed: {}", error);
                error
            })?;
        println!("✅ Vault saved to disk");
        
        println!("🎉 add_entry completed successfully, returning ID: {}", entry_id);
        Ok(entry_id)
    } else {
        let error = "No vault loaded".to_string();
        println!("❌ {}", error);
        Err(error)
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
        
        // 自动保存到磁盘
        vault_manager
            .save_vault()
            .map_err(|e| format!("Failed to save vault: {}", e))?;
            
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

// Passkey authentication commands
#[derive(serde::Serialize)]
struct PasskeyStatus {
    available: bool,
    registered: bool,
    username: Option<String>,
}

#[derive(serde::Serialize)]
struct PasskeyAuthResult {
    success: bool,
    auth_token: Option<String>,
    error: Option<String>,
}

#[tauri::command]
async fn check_passkey_available() -> Result<bool, String> {
    // Check if Touch ID/Face ID is available on macOS
    #[cfg(target_os = "macos")]
    {
        Ok(touchid::is_available())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(false)
    }
}

#[tauri::command]
async fn get_passkey_status(state: State<'_, AppState>) -> Result<PasskeyStatus, String> {
    let passkey_manager = state
        .passkey_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let credentials = passkey_manager.list_credentials();
    let primary_username = credentials.first().map(|c| c.username.clone());
    
    Ok(PasskeyStatus {
        available: touchid::is_available(),
        registered: !credentials.is_empty(),
        username: primary_username,
    })
}

#[tauri::command]
async fn register_passkey(
    state: State<'_, AppState>,
    username: String,
) -> Result<bool, String> {
    let mut passkey_manager = state
        .passkey_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    passkey_manager
        .register_credential(&username, None)
        .map_err(|e| format!("Failed to register Passkey: {}", e))?;

    Ok(true)
}

#[tauri::command]
async fn authenticate_passkey(
    state: State<'_, AppState>,
    username: Option<String>,
) -> Result<PasskeyAuthResult, String> {
    let mut passkey_manager = state
        .passkey_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    match passkey_manager.authenticate(username.as_deref()) {
        Ok(result) => Ok(PasskeyAuthResult {
            success: result.success,
            auth_token: if result.success {
                result.auth_token.map(|token| base64::encode(&token))
            } else {
                None
            },
            error: None,
        }),
        Err(e) => Ok(PasskeyAuthResult {
            success: false,
            auth_token: None,
            error: Some(format!("Authentication failed: {}", e)),
        }),
    }
}

#[tauri::command]
async fn create_vault_with_passkey(
    state: State<'_, AppState>,
    path: String,
    simple_password: String,
    username: String,
    icloud_id: Option<String>,
) -> Result<bool, String> {
    println!("🎯 create_vault_with_passkey called with path: {}", path);
    
    // First register Passkey if not already registered
    let mut passkey_manager = state
        .passkey_manager
        .lock()
        .map_err(|e| format!("Failed to acquire passkey manager lock: {}", e))?;

    // Check if user already has credentials
    let existing_credentials = passkey_manager.list_credentials();
    let has_credential = existing_credentials.iter()
        .any(|cred| cred.username == username);
        
    if !has_credential {
        passkey_manager
            .register_credential(&username, None)
            .map_err(|e| format!("Failed to register Passkey: {}", e))?;
    }

    // Authenticate with Passkey to get auth token
    let auth_result = passkey_manager
        .authenticate(Some(&username))
        .map_err(|e| format!("Passkey authentication failed: {}", e))?;

    if !auth_result.success {
        return Err("Passkey authentication failed".to_string());
    }

    drop(passkey_manager); // Release the lock

    // Derive master key using multi-factor approach
    let icloud_id_hash = if let Some(id) = icloud_id {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        hasher.finalize().to_vec()
    } else {
        vec![0u8; 32] // Default hash if iCloud ID not available
    };

    let random_salt: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();

    let multi_factor_input = MultiFactorInput {
        simple_password,
        passkey_auth_token: auth_result.auth_token.unwrap_or_else(|| vec![0u8; 32]),
        icloud_id_hash,
        random_salt,
    };

    let master_key = derive_master_key(&multi_factor_input, None)
        .map_err(|e| format!("Failed to derive master key: {}", e))?;

    // Convert master key to string for vault creation
    let master_key_string = base64::encode(&master_key);

    // Create vault with derived master key
    let mut vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire vault manager lock: {}", e))?;

    match vault_manager.create_vault(&path, &master_key_string) {
        Ok(_) => {
            println!("✅ Vault created successfully with Passkey integration");
            Ok(true)
        }
        Err(e) => {
            println!("❌ Failed to create vault: {}", e);
            Err(format!("Failed to create vault: {}", e))
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
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
            export_metrics_csv,
            check_passkey_available,
            get_passkey_status,
            register_passkey,
            authenticate_passkey,
            create_vault_with_passkey
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}