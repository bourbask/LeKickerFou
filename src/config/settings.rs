//! Gestion des paramètres de configuration et des fichiers JSON.

use std::{fs, path::Path, time::Duration};

use anyhow::{Context, Result};
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use serenity::model::id::ChannelId;

use crate::{utils::log_info, BotError};

use super::Args;

/// Configuration du bot sauvegardée dans un fichier JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    /// ID du salon vocal à surveiller
    pub voice_channel_id: ChannelId,
    /// ID optionnel du salon de log pour les notifications
    pub log_channel_id: Option<ChannelId>,
    /// ID optionnel du salon textuel pour les avertissements
    pub warning_channel_id: Option<ChannelId>,
    /// Heure de couvre-feu
    #[serde(with = "time_format")]
    pub curfew_time: NaiveTime,
    /// Délai d'attente après avertissement avant déconnexion
    pub warning_delay_seconds: u64,
    /// Mode avertissement uniquement (sans déconnexion)
    pub warning_only: bool,
}

mod time_format {
    use chrono::NaiveTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = time.format("%H:%M").to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveTime::parse_from_str(&s, "%H:%M").map_err(serde::de::Error::custom)
    }
}

impl BotConfig {
    /// Retourne le délai d'avertissement sous forme de Duration
    pub fn warning_delay(&self) -> Duration {
        Duration::from_secs(self.warning_delay_seconds)
    }

    /// Vérifie si les avertissements sont activés
    pub fn has_warnings_enabled(&self) -> bool {
        self.warning_channel_id.is_some()
    }

    /// Vérifie si le mode est "avertissement uniquement"
    pub fn is_warning_only_mode(&self) -> bool {
        self.warning_only
    }
}

/// Gestionnaire de configuration responsable du chargement, sauvegarde et manipulation des configurations
pub struct ConfigManager;

impl ConfigManager {
    /// Crée une nouvelle instance du gestionnaire de configuration
    pub fn new() -> Self {
        Self
    }

    /// Charge la configuration depuis les arguments CLI ou le fichier de config existant
    pub fn load_or_create_configuration(&self, args: &Args) -> Result<BotConfig> {
        if Path::new(&args.config_file).exists() {
            self.load_existing_configuration(args)
        } else {
            self.create_new_configuration(args)
        }
    }

    /// Charge une configuration existante depuis un fichier
    fn load_existing_configuration(&self, args: &Args) -> Result<BotConfig> {
        let config_content = fs::read_to_string(&args.config_file)
            .context("Impossible de lire le fichier de configuration")?;

        let mut config: BotConfig =
            serde_json::from_str(&config_content).context("Fichier de configuration invalide")?;

        // Mise à jour avec les arguments CLI si fournis
        if let Some(channel_id) = args.voice_channel_id {
            config.voice_channel_id = ChannelId::new(channel_id);
        }
        if let Some(log_id) = args.log_channel_id {
            config.log_channel_id = Some(ChannelId::new(log_id));
        }
        if let Some(warning_id) = args.warning_channel_id {
            config.warning_channel_id = Some(ChannelId::new(warning_id));
        }
        if let Some(time_str) = &args.curfew_time {
            config.curfew_time = NaiveTime::parse_from_str(time_str, "%H:%M")
                .context("Format d'heure invalide (utilisez HH:MM)")?;
        }

        config.warning_delay_seconds = args.warning_delay_seconds;
        config.warning_only = args.warning_only;

        self.save_configuration(&config, &args.config_file)?;
        log_info(&format!(
            "Configuration chargée depuis {}",
            args.config_file
        ));

        Ok(config)
    }

    /// Crée une nouvelle configuration depuis les arguments CLI
    fn create_new_configuration(&self, args: &Args) -> Result<BotConfig> {
        let voice_channel_id = args.voice_channel_id.ok_or_else(|| {
            BotError::MissingConfig(
                "ID du salon vocal requis (--channel ou configuration existante)".to_string(),
            )
        })?;

        let curfew_time_str = args.curfew_time.as_ref().ok_or_else(|| {
            BotError::MissingConfig(
                "Heure de couvre-feu requise (--curfew-time HH:MM)".to_string(),
            )
        })?;

        let curfew_time = NaiveTime::parse_from_str(curfew_time_str, "%H:%M")
            .context("Format d'heure invalide (utilisez HH:MM)")?;

        let config = BotConfig {
            voice_channel_id: ChannelId::new(voice_channel_id),
            log_channel_id: args.log_channel_id.map(ChannelId::new),
            warning_channel_id: args.warning_channel_id.map(ChannelId::new),
            curfew_time,
            warning_delay_seconds: args.warning_delay_seconds,
            warning_only: args.warning_only,
        };

        self.save_configuration(&config, &args.config_file)?;
        log_info(&format!(
            "Configuration créée et sauvegardée dans {}",
            args.config_file
        ));

        Ok(config)
    }

    /// Sauvegarde une configuration dans un fichier JSON
    fn save_configuration(&self, config: &BotConfig, file_path: &str) -> Result<()> {
        let config_json = serde_json::to_string_pretty(config)
            .context("Impossible de sérialiser la configuration")?;

        fs::write(file_path, config_json)
            .context("Impossible d'écrire le fichier de configuration")?;

        Ok(())
    }

    /// Importe une configuration depuis un fichier vers la configuration active
    pub async fn import_configuration(&self, source_file: &str, target_file: &str) -> Result<()> {
        if !Path::new(source_file).exists() {
            return Err(BotError::InvalidConfig(format!(
                "Fichier de configuration introuvable: {source_file}"
            ))
            .into());
        }

        let config_content =
            fs::read_to_string(source_file).context("Impossible de lire le fichier source")?;

        let config: BotConfig = serde_json::from_str(&config_content)
            .context("Configuration invalide dans le fichier source")?;

        fs::copy(source_file, target_file).context("Impossible d'importer la configuration")?;

        log_info(&format!(
            "✅ Configuration importée de {source_file} vers {target_file}"
        ));

        self.display_configuration_summary(&config);
        self.display_usage_instructions();

        Ok(())
    }

    /// Exporte la configuration actuelle vers un fichier spécifique
    pub async fn export_configuration(&self, source_file: &str, target_file: &str) -> Result<()> {
        if !Path::new(source_file).exists() {
            return Err(BotError::InvalidConfig(format!(
                "Aucune configuration à exporter depuis {source_file}"
            ))
            .into());
        }

        fs::copy(source_file, target_file).context("Impossible d'exporter la configuration")?;

        log_info(&format!(
            "✅ Configuration exportée de {source_file} vers {target_file}"
        ));

        let config_content = fs::read_to_string(target_file)?;
        let config: BotConfig = serde_json::from_str(&config_content)?;

        self.display_exported_configuration(&config, target_file);

        Ok(())
    }

    /// Affiche un résumé de la configuration actuelle
    fn display_configuration_summary(&self, config: &BotConfig) {
        println!("\n📋 Configuration active :");
        println!("   • Salon vocal: {}", config.voice_channel_id);
        println!(
            "   • Salon de log: {}",
            config
                .log_channel_id
                .map_or("Aucun".to_string(), |id| id.to_string())
        );
        println!(
            "   • Salon d'avertissement: {}",
            config
                .warning_channel_id
                .map_or("Aucun".to_string(), |id| id.to_string())
        );
        println!("   • Heure de couvre-feu: {}", config.curfew_time.format("%H:%M"));
        println!(
            "   • Délai d'avertissement: {} secondes",
            config.warning_delay_seconds
        );
        println!(
            "   • Mode avertissement seul: {}",
            if config.warning_only { "Oui" } else { "Non" }
        );
    }

    /// Affiche les instructions d'utilisation après un import
    fn display_usage_instructions(&self) {
        println!("\n🚀 Vous pouvez maintenant lancer le bot avec :");
        println!("   ./lekickerfou");
    }

    /// Affiche les détails de la configuration exportée
    fn display_exported_configuration(&self, config: &BotConfig, target_file: &str) {
        self.display_configuration_summary(config);
        println!("\n💡 Pour utiliser cette config ailleurs :");
        println!("   ./lekickerfou --import {target_file}");
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
