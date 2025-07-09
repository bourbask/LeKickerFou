//! Gestionnaire de sauvegarde des configurations.

use crate::config::BotConfig;
use anyhow::Result;

/// Gestionnaire de sauvegarde des configurations
pub struct BackupManager;

impl BackupManager {
    /// Crée une nouvelle instance du gestionnaire de sauvegarde
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Crée une sauvegarde de la configuration actuelle
    pub async fn create_backup(
        &self,
        _config: &BotConfig,
        _created_by: Option<&str>,
        _reason: Option<&str>,
    ) -> Result<()> {
        // Pour l'instant, ne rien faire
        Ok(())
    }
}

impl Default for BackupManager {
    fn default() -> Self {
        BackupManager::new().expect("Impossible de créer le gestionnaire de sauvegarde")
    }
}
