//! Command Line Interface for TwoPassword

use crate::Result;
use clap::{Parser, Subcommand};

pub mod commands;

/// TwoPassword - A secure password manager with Touch ID integration
#[derive(Parser)]
#[command(name = "twopassword")]
#[command(about = "A secure password manager with Touch ID integration")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// Vault file path (default: ~/Library/Application Support/TwoPassword/vault.enc)
    #[arg(long, global = true)]
    pub vault: Option<std::path::PathBuf>,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new vault
    Init {
        /// Use Touch ID for authentication (macOS only)
        #[arg(long)]
        touch_id: bool,
    },

    /// Unlock the vault
    Unlock {
        /// Try Touch ID first (macOS only)
        #[arg(long)]
        touch_id: bool,
    },

    /// Add a new password entry
    Add {
        /// Entry title
        title: String,
        /// Username
        #[arg(short, long)]
        username: String,
        /// Password (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,
        /// Website URL
        #[arg(short, long)]
        url: Option<String>,
        /// Notes
        #[arg(short, long)]
        notes: Option<String>,
    },

    /// Get a password entry
    Get {
        /// Search query (title or URL)
        query: String,
    },

    /// List all entries
    List {
        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,
    },

    /// Update an entry
    Update {
        /// Entry ID or title
        identifier: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New username
        #[arg(short, long)]
        username: Option<String>,
        /// New password
        #[arg(short, long)]
        password: Option<String>,
        /// New URL
        #[arg(long)]
        url: Option<String>,
        /// New notes
        #[arg(short, long)]
        notes: Option<String>,
    },

    /// Remove an entry
    Remove {
        /// Entry ID or title
        identifier: String,
        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Generate a secure password
    Generate {
        /// Password length
        #[arg(short, long, default_value = "16")]
        length: usize,
        /// Include uppercase letters
        #[arg(long, default_value = "true")]
        uppercase: bool,
        /// Include lowercase letters
        #[arg(long, default_value = "true")]
        lowercase: bool,
        /// Include numbers
        #[arg(long, default_value = "true")]
        numbers: bool,
        /// Include symbols
        #[arg(long, default_value = "true")]
        symbols: bool,
    },

    /// Show vault status
    Status,

    /// Lock the vault
    Lock,

    /// Export vault (for backup)
    Export {
        /// Export file path
        #[arg(short, long)]
        output: std::path::PathBuf,
        /// Export format (json, csv)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Import entries from file
    Import {
        /// Import file path
        #[arg(short, long)]
        input: std::path::PathBuf,
        /// Import format (json, csv)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
}

/// Main CLI runner
pub struct CliRunner {
    vault_manager: crate::storage::VaultManager,
    auth_manager: crate::auth::AuthManager,
}

impl CliRunner {
    /// Create a new CLI runner
    pub fn new() -> Self {
        Self {
            vault_manager: crate::storage::VaultManager::new(),
            auth_manager: crate::auth::AuthManager::new(),
        }
    }

    /// Run the CLI with the given arguments
    pub async fn run(&mut self, cli: Cli) -> Result<()> {
        // Set up logging level
        if cli.verbose {
            std::env::set_var("RUST_LOG", "debug");
        }

        // Get vault path
        let vault_path = cli.vault.unwrap_or_else(|| self.get_default_vault_path());

        // Execute command
        match cli.command {
            Commands::Init { touch_id } => {
                commands::init::run(
                    &mut self.vault_manager,
                    &self.auth_manager,
                    &vault_path,
                    touch_id,
                )
                .await
            }
            Commands::Unlock { touch_id } => {
                commands::unlock::run(
                    &mut self.vault_manager,
                    &self.auth_manager,
                    &vault_path,
                    touch_id,
                )
                .await
            }
            Commands::Add {
                title,
                username,
                password,
                url,
                notes,
            } => {
                commands::add::run(
                    &mut self.vault_manager,
                    title,
                    username,
                    password,
                    url,
                    notes,
                )
                .await
            }
            Commands::Get { query } => commands::get::run(&self.vault_manager, query).await,
            Commands::List { tag } => commands::list::run(&self.vault_manager, tag).await,
            Commands::Update {
                identifier,
                title,
                username,
                password,
                url,
                notes,
            } => {
                commands::update::run(
                    &mut self.vault_manager,
                    identifier,
                    title,
                    username,
                    password,
                    url,
                    notes,
                )
                .await
            }
            Commands::Remove { identifier, force } => {
                commands::remove::run(&mut self.vault_manager, identifier, force).await
            }
            Commands::Generate {
                length,
                uppercase,
                lowercase,
                numbers,
                symbols,
            } => commands::generate::run(length, uppercase, lowercase, numbers, symbols).await,
            Commands::Status => commands::status::run(&self.vault_manager).await,
            Commands::Lock => commands::lock::run(&mut self.vault_manager).await,
            Commands::Export { output, format } => {
                commands::export::run(&self.vault_manager, output, format).await
            }
            Commands::Import { input, format } => {
                commands::import::run(&mut self.vault_manager, input, format).await
            }
        }
    }

    /// Get default vault path
    fn get_default_vault_path(&self) -> std::path::PathBuf {
        if let Some(dirs) = directories::UserDirs::new() {
            let app_support = dirs
                .home_dir()
                .join("Library")
                .join("Application Support")
                .join(crate::config::APP_NAME);

            std::fs::create_dir_all(&app_support).ok();
            app_support.join(crate::config::VAULT_FILE_NAME)
        } else {
            std::path::PathBuf::from(crate::config::VAULT_FILE_NAME)
        }
    }
}

impl Default for CliRunner {
    fn default() -> Self {
        Self::new()
    }
}
