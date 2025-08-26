// API utilities for communicating with Tauri backend
import { invoke } from "@tauri-apps/api/core";
import { 
  PasswordEntry, 
  VaultStatus, 
  PasswordGeneratorConfig,
  TauriAPI 
} from "../types";

export class TauriAPIClient implements TauriAPI {
  // Basic commands
  async greet(): Promise<string> {
    return await invoke('greet');
  }
  
  // Vault management
  async create_vault(path: string, password: string): Promise<boolean> {
    return await invoke('create_vault', { path, password });
  }
  
  async load_vault(path: string, password: string): Promise<boolean> {
    return await invoke('load_vault', { path, password });
  }
  
  async get_vault_status(): Promise<VaultStatus> {
    return await invoke('get_vault_status');
  }
  
  async save_vault(): Promise<boolean> {
    return await invoke('save_vault');
  }
  
  async close_vault(): Promise<boolean> {
    return await invoke('close_vault');
  }
  
  // Entry management
  async get_all_entries(): Promise<PasswordEntry[]> {
    return await invoke('get_all_entries');
  }
  
  async add_entry(
    title: string, 
    username: string, 
    password: string, 
    url?: string, 
    notes?: string
  ): Promise<string> {
    return await invoke('add_entry', { title, username, password, url, notes });
  }
  
  async remove_entry(entry_id: string): Promise<boolean> {
    return await invoke('remove_entry', { entry_id });
  }
  
  async search_entries(query: string): Promise<PasswordEntry[]> {
    return await invoke('search_entries', { query });
  }
  
  // Password generation
  async generate_password(config: PasswordGeneratorConfig = {}): Promise<string> {
    const {
      length = 16,
      include_symbols = true,
      include_numbers = true,
      include_uppercase = true,
      include_lowercase = true
    } = config;
    
    return await invoke('generate_password', {
      length,
      include_symbols,
      include_numbers,
      include_uppercase,
      include_lowercase
    });
  }
  
  // Touch ID
  async check_touchid_available(): Promise<boolean> {
    return await invoke('check_touchid_available');
  }
  
  async authenticate_touchid(reason: string): Promise<boolean> {
    return await invoke('authenticate_touchid', { reason });
  }
}

// Export singleton instance
export const api = new TauriAPIClient();

// Export individual functions for convenience
export const {
  greet,
  create_vault,
  load_vault,
  get_vault_status,
  save_vault,
  close_vault,
  get_all_entries,
  add_entry,
  remove_entry,
  search_entries,
  generate_password,
  check_touchid_available,
  authenticate_touchid
} = api;