// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::Mutex;
use tauri::State;

// Import 2Password core functionality
use twopassword::storage::{VaultManager, PasswordEntry, SearchOptions};
use twopassword::auth::touchid;

// Application state that will be shared across Tauri commands  
pub struct AppState {
    vault_manager: Mutex<VaultManager>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            vault_manager: Mutex::new(VaultManager::new()),
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
            check_touchid_available,
            authenticate_touchid
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}