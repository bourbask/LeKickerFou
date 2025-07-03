//! Module de gestion de la configuration du bot.

pub mod args;
pub mod settings;

pub use args::Args;
pub use settings::{BotConfig, ConfigManager};
