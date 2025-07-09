//! Logger Discord pour envoyer les messages dans les salons configurés.

use anyhow::Result;
use lazy_static::lazy_static;
use serenity::client::Context;
use std::sync::{Arc, Mutex};

use super::VerbosityLevel;
use crate::config::BotConfig;

/// Types de logs
#[derive(Debug, Clone)]
pub enum LogType {
    Info,
    Warning,
    Error,
}

/// Logger Discord
pub struct DiscordLogger {
    _config: BotConfig,
    _verbosity: VerbosityLevel,
}

impl DiscordLogger {
    /// Crée un nouveau logger Discord
    pub fn new(config: BotConfig, verbosity: VerbosityLevel) -> Self {
        Self {
            _config: config,
            _verbosity: verbosity,
        }
    }

    /// Log un message simple
    pub async fn log(&self, _ctx: &Context, _log_type: LogType, _message: &str) -> Result<()> {
        // Pour l'instant, ne rien faire
        Ok(())
    }

    /// Log une interaction utilisateur
    pub async fn log_user_interaction(
        &self,
        _ctx: &Context,
        _user_tag: &str,
        _action: &str,
        _details: Option<&str>,
    ) -> Result<()> {
        // Pour l'instant, ne rien faire
        Ok(())
    }

    /// Log une déconnexion
    pub async fn log_kick(
        &self,
        _ctx: &Context,
        _user_tag: &str,
        _channel_name: &str,
        _reason: Option<&str>,
    ) -> Result<()> {
        // Pour l'instant, ne rien faire
        Ok(())
    }

    /// Log une erreur
    pub async fn log_error(
        &self,
        _ctx: &Context,
        _error_message: &str,
        _details: Option<&str>,
    ) -> Result<()> {
        // Pour l'instant, ne rien faire
        Ok(())
    }
}

// Logger global
lazy_static! {
    static ref GLOBAL_LOGGER: Arc<Mutex<Option<DiscordLogger>>> = Arc::new(Mutex::new(None));
}

/// Initialise le logger global
pub fn init_global_logger(logger: DiscordLogger) {
    let mut global_logger = GLOBAL_LOGGER.lock().unwrap();
    *global_logger = Some(logger);
}

/// Récupère le logger global
pub fn get_global_logger() -> Arc<Mutex<Option<DiscordLogger>>> {
    GLOBAL_LOGGER.clone()
}
