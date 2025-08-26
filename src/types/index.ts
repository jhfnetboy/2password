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