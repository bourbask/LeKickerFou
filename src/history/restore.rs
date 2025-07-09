//! Gestionnaire de restauration des configurations.

use crate::config::BotConfig;
use anyhow::Result;

/// Gestionnaire de restauration des configurations
pub struct RestoreManager;

impl RestoreManager {
    /// Crée une nouvelle instance du gestionnaire de restauration
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Restaure une configuration depuis une sauvegarde spécifique
    pub async fn restore_backup(
        &self,
        _backup_filename: &str,
        _target_config_file: &str,
    ) -> Result<BotConfig> {
        // Pour l'instant, retourner une config par défaut
        use serenity::model::id::ChannelId;

        Ok(BotConfig {
            voice_channel_id: ChannelId::new(123456789012345678),
            log_channel_id: None,
            warning_channel_id: None,
            warning_delay_seconds: 60,
            warning_only: false,
            cron_schedule: "0 * * * * *".to_string(),
        })
    }
}

impl Default for RestoreManager {
    fn default() -> Self {
        Self::new().expect("Impossible de créer le gestionnaire de restauration")
    }
}
