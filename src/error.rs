//! Types d'erreurs personnalisés pour le bot.

use std::fmt;

/// Erreurs personnalisées du bot Discord
#[derive(Debug)]
pub enum BotError {
    /// Erreur de configuration
    InvalidConfig(String),
    /// Configuration manquante
    MissingConfig(String),
    /// Erreur Discord API
    DiscordError(String),
    /// Erreur de permissions
    PermissionError(String),
}

impl fmt::Display for BotError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BotError::InvalidConfig(msg) => write!(f, "Configuration invalide: {}", msg),
            BotError::MissingConfig(msg) => write!(f, "Configuration manquante: {}", msg),
            BotError::DiscordError(msg) => write!(f, "Erreur Discord: {}", msg),
            BotError::PermissionError(msg) => write!(f, "Erreur de permission: {}", msg),
        }
    }
}

impl std::error::Error for BotError {}
