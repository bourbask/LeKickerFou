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
    long_about = "Bot Discord qui surveille un salon vocal et déconnecte automatiquement tous les utilisateurs présents selon un planning configurable. Peut aussi envoyer des avertissements avant déconnexion."
)]
#[command(after_help = "EXEMPLES:\n  \
    # Configuration initiale\n  \
    lekickerfou --channel 123456789\n\n  \
    # Avec salon de log et planning personnalisé (toutes les 30 secondes)\n  \
    lekickerfou --channel 123456789 --log-channel 987654321 --schedule \"*/30 * * * * *\"\n\n  \
    # Avec avertissement avant déconnexion\n  \
    lekickerfou --channel 123456789 --warning-channel 555666777\n\n  \
    # Avertissement uniquement (sans déconnexion)\n  \
    lekickerfou --channel 123456789 --warning-channel 555666777 --warning-only\n\n  \
    # Avertissement avec délai personnalisé (5 minutes)\n  \
    lekickerfou --channel 123456789 --warning-channel 555666777 --warning-delay 300\n\n  \
    # Export de la configuration\n  \
    lekickerfou --export production-config.json\n\n  \
    # Import d'une configuration\n  \
    lekickerfou --import production-config.json\n\n  \
    # Utilisation d'un fichier de config personnalisé\n  \
    lekickerfou --config-file my-config.json --channel 123456789\n\n\
VARIABLES D'ENVIRONNEMENT:\n  \
    DISCORD_TOKEN    Token du bot Discord (obligatoire)\n\n\
FICHIERS:\n  \
    bot_config.json  Fichier de configuration par défaut")]
pub struct Args {
    /// ID du salon vocal à surveiller (obligatoire pour une nouvelle configuration)
    #[arg(
        short = 'c',
        long = "channel",
        help = "ID du salon vocal à surveiller",
        long_help = "ID numérique du salon vocal Discord à surveiller. Tous les utilisateurs connectés à ce salon seront automatiquement déconnectés selon le planning défini."
    )]
    pub voice_channel_id: Option<u64>,

    /// ID du salon de log pour enregistrer les déconnexions (optionnel)
    #[arg(
        short = 'l',
        long = "log-channel",
        help = "ID du salon de log (optionnel)",
        long_help = "ID numérique du salon Discord où seront envoyés les messages de log des déconnexions. Si non spécifié, seuls les logs console seront affichés."
    )]
    pub log_channel_id: Option<u64>,

    /// ID du salon textuel pour envoyer les avertissements (optionnel)
    #[arg(
        short = 'w',
        long = "warning-channel",
        help = "ID du salon d'avertissement (optionnel)",
        long_help = "ID numérique du salon Discord textuel où seront envoyés les messages d'avertissement avant déconnexion. Les utilisateurs seront mentionnés avec un message comique."
    )]
    pub warning_channel_id: Option<u64>,

    /// Délai en secondes avant la déconnexion après l'avertissement
    #[arg(
        long = "warning-delay",
        default_value = "60",
        help = "Délai en secondes avant la déconnexion après avertissement",
        long_help = "Nombre de secondes à attendre après avoir envoyé l'avertissement avant de procéder à la déconnexion. Par défaut 60 secondes."
    )]
    pub warning_delay_seconds: u64,

    /// Mode avertissement uniquement (pas de déconnexion)
    #[arg(
        long = "warning-only",
        help = "Envoyer uniquement l'avertissement sans déconnecter",
        long_help = "Si activé, le bot n'enverra que l'avertissement sans déconnecter les utilisateurs. Utile pour un mode 'gentil' de rappel."
    )]
    pub warning_only: bool,

    /// Expression cron pour définir quand vérifier le salon vocal
    #[arg(
        short = 's',
        long = "schedule",
        default_value = "0 * * * * *",
        help = "Expression cron pour la fréquence de vérification",
        long_help = "Expression cron définissant quand vérifier et déconnecter les utilisateurs. Par défaut '0 * * * * *' (toutes les minutes). Exemples: '*/30 * * * * *' (toutes les 30 secondes), '0 0 22 * * *' (tous les jours à 22h)."
    )]
    pub cron_schedule: String,

    /// Chemin vers le fichier de configuration JSON
    #[arg(
        short = 'f',
        long = "config-file",
        default_value = CONFIG_FILE,
        help = "Chemin vers le fichier de configuration",
        long_help = "Chemin vers le fichier JSON contenant la configuration du bot. Le fichier sera créé automatiquement s'il n'existe pas. Permet d'avoir plusieurs configurations différentes."
    )]
    pub config_file: String,

    /// Exporter la configuration actuelle vers un fichier
    #[arg(
        long = "export",
        value_name = "FICHIER",
        help = "Exporter la configuration vers un fichier",
        long_help = "Exporte la configuration actuelle vers le fichier spécifié. Utile pour sauvegarder ou partager une configuration. Le bot s'arrête après l'export.",
        conflicts_with_all = ["import_from", "voice_channel_id", "log_channel_id", "warning_channel_id"]
    )]
    pub export_to: Option<String>,

    /// Importer une configuration depuis un fichier
    #[arg(
        long = "import",
        value_name = "FICHIER",
        help = "Importer une configuration depuis un fichier",
        long_help = "Importe une configuration depuis le fichier spécifié et la définit comme configuration active. Remplace la configuration actuelle. Le bot s'arrête après l'import.",
        conflicts_with_all = ["export_to", "voice_channel_id", "log_channel_id", "warning_channel_id"]
    )]
    pub import_from: Option<String>,
}
