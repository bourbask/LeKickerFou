//! Définition des arguments en ligne de commande.

use clap::Parser;

const CONFIG_FILE: &str = "bot_config.json";

/// Arguments en ligne de commande pour la configuration du bot
#[derive(Parser, Debug)]
#[command(name = "lekickerfou")]
#[command(version = "1.1.0")]
#[command(
    about = "Bot Discord pour déconnecter automatiquement les utilisateurs des salons vocaux"
)]
#[command(
    long_about = "Bot Discord qui surveille un salon vocal et déconnecte automatiquement tous les utilisateurs présents à une heure de couvre-feu configurée. Peut aussi envoyer des avertissements avant déconnexion."
)]
pub struct Args {
    /// ID du salon vocal à surveiller (obligatoire pour une nouvelle configuration)
    #[arg(
        short = 'c',
        long = "channel",
        help = "ID du salon vocal à surveiller"
    )]
    pub voice_channel_id: Option<u64>,

    /// ID du salon de log pour enregistrer les déconnexions (optionnel)
    #[arg(
        short = 'l',
        long = "log-channel",
        help = "ID du salon de log (optionnel)"
    )]
    pub log_channel_id: Option<u64>,

    /// ID du salon textuel pour envoyer les avertissements (optionnel)
    #[arg(
        short = 'w',
        long = "warning-channel",
        help = "ID du salon d'avertissement (optionnel)"
    )]
    pub warning_channel_id: Option<u64>,

    /// Heure de couvre-feu au format HH:MM (24h)
    #[arg(
        short = 't',
        long = "curfew-time",
        help = "Heure de couvre-feu au format HH:MM (ex: 22:30)"
    )]
    pub curfew_time: Option<String>,

    /// Délai en secondes avant la déconnexion après l'avertissement
    #[arg(
        long = "warning-delay",
        default_value = "600",
        help = "Délai en secondes avant la déconnexion après avertissement"
    )]
    pub warning_delay_seconds: u64,

    /// Mode avertissement uniquement (pas de déconnexion)
    #[arg(
        long = "warning-only",
        help = "Envoyer uniquement l'avertissement sans déconnecter"
    )]
    pub warning_only: bool,

    /// Chemin vers le fichier de configuration JSON
    #[arg(
        short = 'f',
        long = "config-file",
        default_value = CONFIG_FILE,
        help = "Chemin vers le fichier de configuration"
    )]
    pub config_file: String,

    /// Exporter la configuration actuelle vers un fichier
    #[arg(
        long = "export",
        value_name = "FICHIER",
        help = "Exporter la configuration vers un fichier",
        conflicts_with_all = ["import_from", "voice_channel_id", "log_channel_id", "warning_channel_id"]
    )]
    pub export_to: Option<String>,

    /// Importer une configuration depuis un fichier
    #[arg(
        long = "import",
        value_name = "FICHIER",
        help = "Importer une configuration depuis un fichier",
        conflicts_with_all = ["export_to", "voice_channel_id", "log_channel_id", "warning_channel_id"]
    )]
    pub import_from: Option<String>,
}