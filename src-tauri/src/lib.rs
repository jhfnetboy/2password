// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::Mutex;
use tauri::{Manager, State};

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
            get_all_entries,
            add_entry,
            check_touchid_available,
            authenticate_touchid
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}