// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::Mutex;
use tauri::State;

// Import 2Password core functionality
use twopassword::storage::{VaultManager, PasswordEntry};
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
) -> Result<String, String> {
    let mut vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(vault) = vault_manager.get_vault_mut() {
        let mut entry = PasswordEntry::new(title, username, password);
        entry.url = url;
        entry.notes = notes;
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
            save_vault,
            close_vault,
            generate_password,
            check_touchid_available,
            authenticate_touchid
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}