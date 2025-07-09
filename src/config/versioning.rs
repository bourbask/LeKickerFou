//! Gestionnaire de versioning et migration des configurations.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{config::BotConfig, utils::log_info, BotError};

/// Version actuelle du format de configuration
const CURRENT_CONFIG_VERSION: &str = "1.1.0";

/// Structure de configuration avec informations de version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedConfig {
    /// Version du format de configuration
    pub version: String,
    /// Configuration du bot
    pub config: BotConfig,
    /// Métadonnées de version
    pub version_metadata: VersionMetadata,
}

/// Métadonnées de version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMetadata {
    /// Version du bot qui a créé la configuration
    pub created_by_version: String,
    /// Date de création
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Dernière migration appliquée
    pub last_migration: Option<String>,
    /// Historique des migrations
    pub migration_history: Vec<MigrationRecord>,
}

/// Enregistrement d'une migration appliquée
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    /// Version source
    pub from_version: String,
    /// Version cible
    pub to_version: String,
    /// Date de la migration
    pub migrated_at: chrono::DateTime<chrono::Utc>,
    /// Description de la migration
    pub description: String,
}

/// Gestionnaire de versioning et migration des configurations
pub struct ConfigVersionManager;

impl ConfigVersionManager {
    /// Crée une nouvelle configuration versioned
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration du bot
    ///
    /// # Returns
    ///
    /// Configuration avec informations de version
    pub fn create_versioned_config(config: BotConfig) -> VersionedConfig {
        VersionedConfig {
            version: CURRENT_CONFIG_VERSION.to_string(),
            config,
            version_metadata: VersionMetadata {
                created_by_version: env!("CARGO_PKG_VERSION").to_string(),
                created_at: chrono::Utc::now(),
                last_migration: None,
                migration_history: Vec::new(),
            },
        }
    }

    /// Vérifie si une configuration nécessite une migration
    ///
    /// # Arguments
    ///
    /// * `config_version` - Version de la configuration
    ///
    /// # Returns
    ///
    /// True si une migration est nécessaire
    pub fn needs_migration(config_version: &str) -> bool {
        config_version != CURRENT_CONFIG_VERSION
    }

    /// Effectue la migration d'une configuration vers la version actuelle
    ///
    /// # Arguments
    ///
    /// * `versioned_config` - Configuration versioned à migrer
    ///
    /// # Returns
    ///
    /// Configuration migrée vers la version actuelle
    pub fn migrate_to_current(mut versioned_config: VersionedConfig) -> Result<VersionedConfig> {
        let original_version = versioned_config.version.clone();

        if !Self::needs_migration(&versioned_config.version) {
            return Ok(versioned_config); // Déjà à jour
        }

        log_info(&format!(
            "Migration de configuration: {} → {}",
            original_version, CURRENT_CONFIG_VERSION
        ));

        // Appliquer les migrations en séquence
        versioned_config = Self::migrate_from_version(versioned_config)?;

        // Mettre à jour les métadonnées de version
        versioned_config.version = CURRENT_CONFIG_VERSION.to_string();
        versioned_config.version_metadata.last_migration = Some(CURRENT_CONFIG_VERSION.to_string());

        // Ajouter un enregistrement de migration
        versioned_config
            .version_metadata
            .migration_history
            .push(MigrationRecord {
                from_version: original_version.clone(),
                to_version: CURRENT_CONFIG_VERSION.to_string(),
                migrated_at: chrono::Utc::now(),
                description: format!(
                    "Migration automatique de {} vers {}",
                    original_version, CURRENT_CONFIG_VERSION
                ),
            });

        log_info("Migration de configuration terminée avec succès");
        Ok(versioned_config)
    }

    /// Applique les migrations spécifiques selon la version source
    ///
    /// # Arguments
    ///
    /// * `versioned_config` - Configuration à migrer
    ///
    /// # Returns
    ///
    /// Configuration migrée
    fn migrate_from_version(mut versioned_config: VersionedConfig) -> Result<VersionedConfig> {
        match versioned_config.version.as_str() {
            "1.0.0" => {
                // Migration de 1.0.0 vers 1.1.0
                log_info("Application de la migration 1.0.0 → 1.1.0");

                // Exemple : Ajout de nouveaux champs avec valeurs par défaut
                // (Dans ce cas, tous les champs existaient déjà, donc pas de migration spécifique)

                versioned_config.version = "1.1.0".to_string();
            }
            _ => {
                return Err(BotError::InvalidConfig(format!(
                    "Version de configuration non supportée: {}",
                    versioned_config.version
                ))
                .into());
            }
        }

        Ok(versioned_config)
    }

    /// Extrait la configuration simple depuis une configuration versioned
    ///
    /// # Arguments
    ///
    /// * `versioned_config` - Configuration versioned
    ///
    /// # Returns
    ///
    /// Configuration simple du bot
    pub fn extract_config(versioned_config: VersionedConfig) -> BotConfig {
        versioned_config.config
    }

    /// Crée une configuration versioned depuis du contenu JSON
    ///
    /// # Arguments
    ///
    /// * `content` - Contenu JSON de la configuration
    ///
    /// # Returns
    ///
    /// Configuration versioned, migrée si nécessaire
    pub fn from_json_content(content: &str) -> Result<VersionedConfig> {
        // Tenter de désérialiser comme configuration versioned
        if let Ok(versioned_config) = serde_json::from_str::<VersionedConfig>(content) {
            return Self::migrate_to_current(versioned_config);
        }

        // Fallback: tenter comme configuration simple (legacy)
        let legacy_config: BotConfig =
            serde_json::from_str(content).context("Format de configuration invalide")?;

        log_info("Configuration legacy détectée, conversion en format versioned");

        let mut versioned_config = Self::create_versioned_config(legacy_config);

        // Marquer comme migré depuis une version legacy (assumer 1.0.0)
        versioned_config
            .version_metadata
            .migration_history
            .push(MigrationRecord {
                from_version: "1.0.0-legacy".to_string(),
                to_version: CURRENT_CONFIG_VERSION.to_string(),
                migrated_at: chrono::Utc::now(),
                description: "Migration depuis format legacy vers format versioned".to_string(),
            });

        Ok(versioned_config)
    }

    /// Affiche les informations de version d'une configuration
    ///
    /// # Arguments
    ///
    /// * `versioned_config` - Configuration versioned
    pub fn display_version_info(versioned_config: &VersionedConfig) {
        println!("📋 Informations de version:");
        println!("   • Version config: {}", versioned_config.version);
        println!(
            "   • Créé par version: {}",
            versioned_config.version_metadata.created_by_version
        );
        println!(
            "   • Date création: {}",
            versioned_config
                .version_metadata
                .created_at
                .format("%Y-%m-%d %H:%M:%S UTC")
        );

        if let Some(last_migration) = &versioned_config.version_metadata.last_migration {
            println!("   • Dernière migration: {}", last_migration);
        }

        if !versioned_config
            .version_metadata
            .migration_history
            .is_empty()
        {
            println!("   • Historique migrations:");
            for migration in &versioned_config.version_metadata.migration_history {
                println!(
                    "     - {} → {} ({})",
                    migration.from_version,
                    migration.to_version,
                    migration.migrated_at.format("%Y-%m-%d %H:%M:%S")
                );
                println!("       {}", migration.description);
            }
        }
    }

    /// Retourne la version actuelle du format de configuration
    pub fn current_version() -> &'static str {
        CURRENT_CONFIG_VERSION
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serenity::model::id::ChannelId;

    #[test]
    fn test_versioned_config_creation() {
        let config = BotConfig {
            voice_channel_id: ChannelId::new(123),
            log_channel_id: None,
            warning_channel_id: None,
            warning_delay_seconds: 60,
            warning_only: false,
            cron_schedule: "0 * * * * *".to_string(),
        };

        let versioned = ConfigVersionManager::create_versioned_config(config);
        assert_eq!(versioned.version, CURRENT_CONFIG_VERSION);
        assert!(!versioned.version_metadata.created_by_version.is_empty());
    }

    #[test]
    fn test_migration_detection() {
        assert!(ConfigVersionManager::needs_migration("1.0.0"));
        assert!(!ConfigVersionManager::needs_migration(
            CURRENT_CONFIG_VERSION
        ));
    }

    #[test]
    fn test_config_extraction() {
        let config = BotConfig {
            voice_channel_id: ChannelId::new(123),
            log_channel_id: None,
            warning_channel_id: None,
            warning_delay_seconds: 60,
            warning_only: false,
            cron_schedule: "0 * * * * *".to_string(),
        };

        let versioned = ConfigVersionManager::create_versioned_config(config.clone());
        let extracted = ConfigVersionManager::extract_config(versioned);

        assert_eq!(extracted.voice_channel_id, config.voice_channel_id);
        assert_eq!(extracted.cron_schedule, config.cron_schedule);
    }
}
