//! Bibliothèque LeKickerFou - Bot Discord de gestion des salons vocaux

pub mod config;
pub mod error;
pub mod handler;
pub mod history;
pub mod logging;
pub mod permissions;
pub mod scheduler;
pub mod utils;

// Ré-exports principaux
pub use config::{Args, BotConfig, ConfigManager};
pub use error::BotError;

/// Version actuelle du bot
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
