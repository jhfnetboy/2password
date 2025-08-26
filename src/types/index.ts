// Type definitions for 2Password GUI application

export interface PasswordEntry {
  id: string;
  title: string;
  username: string;
  password: string;
  url?: string;
  notes?: string;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface VaultMetadata {
  format_version: number;
  created_at: string;
  updated_at: string;
  entry_count: number;
}

export interface AppState {
  isVaultLoaded: boolean;
  currentVaultPath?: string;
  touchIdEnabled: boolean;
  autoLockEnabled: boolean;
  autoLockTimeout: number; // minutes
}

export type VaultCreationStep = 
  | "welcome"
  | "location"
  | "password"
  | "touchid"
  | "complete";

export interface CreateVaultData {
  path: string;
  masterPassword: string;
  confirmPassword: string;
  enableTouchId: boolean;
}

export interface VaultStatus {
  loaded: boolean;
  path?: string;
}

export interface PasswordGeneratorConfig {
  length?: number;
  include_symbols?: boolean;
  include_numbers?: boolean;
  include_uppercase?: boolean;
  include_lowercase?: boolean;
}

export interface AdvancedSearchOptions {
  query?: string;
  tags?: string[];
  created_after?: string;
  created_before?: string;
}

// Tauri API interface
export interface TauriAPI {
  // Basic commands
  greet(): Promise<string>;
  
  // Vault management
  create_vault(path: string, password: string): Promise<boolean>;
  load_vault(path: string, password: string): Promise<boolean>;
  get_vault_status(): Promise<VaultStatus>;
  save_vault(): Promise<boolean>;
  close_vault(): Promise<boolean>;
  
  // Entry management
  get_all_entries(): Promise<PasswordEntry[]>;
  add_entry(title: string, username: string, password: string, url?: string, notes?: string, tags?: string[]): Promise<string>;
  remove_entry(entry_id: string): Promise<boolean>;
  search_entries(query: string): Promise<PasswordEntry[]>;
  
  // Advanced search and tag management
  advanced_search(options: AdvancedSearchOptions): Promise<PasswordEntry[]>;
  get_all_tags(): Promise<string[]>;
  add_tag_to_entry(entry_id: string, tag: string): Promise<boolean>;
  remove_tag_from_entry(entry_id: string, tag: string): Promise<boolean>;
  
  // Password generation
  generate_password(config?: PasswordGeneratorConfig): Promise<string>;
  
  // Touch ID
  check_touchid_available(): Promise<boolean>;
  authenticate_touchid(reason: string): Promise<boolean>;
}