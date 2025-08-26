// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{Manager, State};
use twopassword::storage::VaultManager;
use twopassword::Result as TwoPasswordResult;

// Application state that will be shared across Tauri commands
pub struct AppState {
    vault_manager: Mutex<VaultManager>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            vault_manager: Mutex::new(VaultManager::new()),
        }
    }
}

// Tauri commands that will be exposed to the frontend
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
async fn is_vault_loaded(state: State<'_, AppState>) -> Result<bool, String> {
    let vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    Ok(vault_manager.is_vault_loaded())
}

#[tauri::command]
async fn close_vault(state: State<'_, AppState>) -> Result<(), String> {
    let mut vault_manager = state
        .vault_manager
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    vault_manager.close_vault();
    Ok(())
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
        let mut entry = twopassword::storage::PasswordEntry::new(title, username, password);
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
async fn check_touchid_available() -> Result<bool, String> {
    Ok(twopassword::auth::touchid::is_available())
}

#[tauri::command]
async fn authenticate_touchid(reason: String) -> Result<bool, String> {
    twopassword::auth::touchid::authenticate(&reason)
        .map_err(|e| format!("Touch ID authentication failed: {}", e))
}

fn main() {
    // Initialize tracing for debugging
    tracing_subscriber::fmt::init();

    // Initialize 2Password core library
    if let Err(e) = twopassword::init() {
        eprintln!("Failed to initialize 2Password: {}", e);
        std::process::exit(1);
    }

    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            create_vault,
            load_vault,
            is_vault_loaded,
            close_vault,
            get_all_entries,
            add_entry,
            remove_entry,
            search_entries,
            save_vault,
            check_touchid_available,
            authenticate_touchid
        ])
        .setup(|app| {
            // Setup application
            tracing::info!("2Password GUI started");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}