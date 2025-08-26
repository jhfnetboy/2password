// Tauri library exports for GUI application

pub use twopassword::*;

// Re-export common types for frontend use
pub mod gui {
    pub use twopassword::storage::{PasswordEntry, VaultManager};
    pub use twopassword::auth::{recovery::RecoveryManager, touchid};
    pub use twopassword::crypto::MasterKey;
    pub use twopassword::{Result, TwoPasswordError};
}