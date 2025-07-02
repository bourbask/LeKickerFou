use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    model::{
        channel::{Channel, ChannelType},
        gateway::Ready,
        id::ChannelId,
    },
    prelude::*,
};

use anyhow::{Context as AnyhowContext, Result};
use chrono::Utc;
use clap::Parser;
use colored::*;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_cron_scheduler::{Job, JobScheduler};

use std::{env, fs, path::Path};

const CONFIG_FILE: &str = "bot_config.json";

/**
 * Arguments en ligne de commande pour la configuration du bot
 */
#[derive(Parser, Debug)]
#[command(name = "lekickerfou")]
#[command(version = "1.0.0")]
#[command(
    about = "Bot Discord pour déconnecter automatiquement les utilisateurs des salons vocaux"
)]
#[command(
    long_about = "Bot Discord qui surveille un salon vocal et déconnecte automatiquement tous les utilisateurs présents selon un planning configurable. Parfait pour forcer la fermeture d'un salon à certaines heures."
)]
#[command(after_help = "EXEMPLES:\n  \
    # Configuration initiale\n  \
    lekickerfou --channel 123456789\n\n  \
    # Avec salon de log et planning personnalisé (toutes les 30 secondes)\n  \
    lekickerfou --channel 123456789 --log-channel 987654321 --schedule \"*/30 * * * * *\"\n\n  \
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
struct Args {
    /// ID du salon vocal à surveiller (obligatoire pour une nouvelle configuration)
    #[arg(
        short = 'c',
        long = "channel",
        help = "ID du salon vocal à surveiller",
        long_help = "ID numérique du salon vocal Discord à surveiller. Tous les utilisateurs connectés à ce salon seront automatiquement déconnectés selon le planning défini."
    )]
    voice_channel_id: Option<u64>,

    /// ID du salon de log pour enregistrer les déconnexions (optionnel)
    #[arg(
        short = 'l',
        long = "log-channel",
        help = "ID du salon de log (optionnel)",
        long_help = "ID numérique du salon Discord où seront envoyés les messages de log des déconnexions. Si non spécifié, seuls les logs console seront affichés."
    )]
    log_channel_id: Option<u64>,

    /// Expression cron pour définir quand vérifier le salon vocal
    #[arg(
        short = 's',
        long = "schedule",
        default_value = "0 * * * * *",
        help = "Expression cron pour la fréquence de vérification",
        long_help = "Expression cron définissant quand vérifier et déconnecter les utilisateurs. Par défaut '0 * * * * *' (toutes les minutes). Exemples: '*/30 * * * * *' (toutes les 30 secondes), '0 0 22 * * *' (tous les jours à 22h)."
    )]
    cron_schedule: String,

    /// Chemin vers le fichier de configuration JSON
    #[arg(
        short = 'f',
        long = "config-file",
        default_value = CONFIG_FILE,
        help = "Chemin vers le fichier de configuration",
        long_help = "Chemin vers le fichier JSON contenant la configuration du bot. Le fichier sera créé automatiquement s'il n'existe pas. Permet d'avoir plusieurs configurations différentes."
    )]
    config_file: String,

    /// Exporter la configuration actuelle vers un fichier
    #[arg(
        long = "export",
        value_name = "FICHIER",
        help = "Exporter la configuration vers un fichier",
        long_help = "Exporte la configuration actuelle vers le fichier spécifié. Utile pour sauvegarder ou partager une configuration. Le bot s'arrête après l'export.",
        conflicts_with_all = ["import_from", "voice_channel_id", "log_channel_id"]
    )]
    export_to: Option<String>,

    /// Importer une configuration depuis un fichier
    #[arg(
        long = "import",
        value_name = "FICHIER", 
        help = "Importer une configuration depuis un fichier",
        long_help = "Importe une configuration depuis le fichier spécifié et la définit comme configuration active. Remplace la configuration actuelle. Le bot s'arrête après l'import.",
        conflicts_with_all = ["export_to", "voice_channel_id", "log_channel_id"]
    )]
    import_from: Option<String>,
}

/**
 * Importe une configuration depuis un fichier vers la config active
 */
async fn import_configuration(source_file: &str, target_file: &str) -> Result<()> {
    if !Path::new(source_file).exists() {
        return Err(BotError::InvalidConfig(format!(
            "Fichier de configuration introuvable: {source_file}"
        ))
        .into());
    }

    // Valider la configuration avant import
    let config_content =
        fs::read_to_string(source_file).context("Impossible de lire le fichier source")?;

    let config: BotConfig = serde_json::from_str(&config_content)
        .context("Configuration invalide dans le fichier source")?;

    // Copier vers la config active
    fs::copy(source_file, target_file).context("Impossible d'importer la configuration")?;

    log_info(&format!(
        "✅ Configuration importée de {source_file} vers {target_file}"
    ));

    println!("\n📋 Configuration active :");
    println!("   • Salon vocal: {}", config.voice_channel_id);
    println!(
        "   • Salon de log: {}",
        config
            .log_channel_id
            .map_or("Aucun".to_string(), |id| id.to_string())
    );
    println!("   • Planning: {}", config.cron_schedule);
    println!("\n🚀 Vous pouvez maintenant lancer le bot avec :");
    println!("   ./lekickerfou");

    Ok(())
}

/**
 * Configuration du bot sauvegardée dans un fichier JSON
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BotConfig {
    voice_channel_id: ChannelId,
    log_channel_id: Option<ChannelId>,
    cron_schedule: String,
}

/**
 * Erreurs personnalisées pour une meilleure gestion des cas d'échec
 */
#[derive(Error, Debug)]
pub enum BotError {
    #[error("Configuration manquante: {0}")]
    MissingConfig(String),

    #[error("Le salon n'est pas un salon vocal")]
    InvalidChannelType,

    #[error("Le salon n'est pas un salon de serveur")]
    NotGuildChannel,

    #[error("Erreur Discord API: {0}")]
    DiscordApi(#[from] serenity::Error),

    #[error("Configuration invalide: {0}")]
    InvalidConfig(String),
}

/**
 * Structure principale du bot Discord gérant les événements
 */
struct Bot {
    config: BotConfig,
}

impl Bot {
    fn new(config: BotConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl EventHandler for Bot {
    /**
     * Gestionnaire d'événement déclenché quand le bot est prêt
     * Initialise le système de tâches planifiées
     */
    async fn ready(&self, ctx: Context, ready: Ready) {
        log_info(&format!(
            "Bot connecté sous {} (ID: {})",
            ready.user.name, ready.user.id
        ));

        log_info(&format!(
            "Configuration: Canal vocal {}, Canal de log {}",
            self.config.voice_channel_id,
            self.config
                .log_channel_id
                .map_or("Aucun".to_string(), |id| id.to_string())
        ));

        if let Err(err) = self.start_voice_monitoring(ctx).await {
            log_error(&format!(
                "Erreur lors du démarrage de la surveillance: {err}"
            ));
        }
    }
}

impl Bot {
    /**
     * Démarre la surveillance des salons vocaux avec une tâche cron
     */
    async fn start_voice_monitoring(&self, ctx: Context) -> Result<()> {
        let scheduler = JobScheduler::new()
            .await
            .context("Impossible de créer le planificateur de tâches")?;

        let config = self.config.clone();
        let cron_expr = config.cron_schedule.clone();

        let job = Job::new_async(&cron_expr, move |_uuid, _scheduler| {
            let ctx_clone = ctx.clone();
            let config_clone = config.clone();

            Box::pin(async move {
                match VoiceChannelManager::new(config_clone)
                    .check_and_disconnect_users(&ctx_clone)
                    .await
                {
                    Ok(disconnected_count) => {
                        if disconnected_count > 0 {
                            log_info(&format!(
                                "{disconnected_count} utilisateur(s) déconnecté(s)"
                            ));
                        }
                    }
                    Err(e) => log_error(&format!("Erreur lors de la vérification: {e}")),
                }
            })
        })
        .context("Expression cron invalide")?;

        scheduler
            .add(job)
            .await
            .context("Impossible d'ajouter la tâche au planificateur")?;

        scheduler
            .start()
            .await
            .context("Impossible de démarrer le planificateur")?;

        log_info(&format!(
            "Surveillance des salons vocaux démarrée (planning: {})",
            self.config.cron_schedule
        ));
        Ok(())
    }
}

/**
 * Gestionnaire responsable de la surveillance et de la déconnexion des utilisateurs
 */
struct VoiceChannelManager {
    config: BotConfig,
}

impl VoiceChannelManager {
    fn new(config: BotConfig) -> Self {
        Self { config }
    }

    /**
     * Vérifie le salon vocal configuré et déconnecte les utilisateurs si nécessaire
     */
    async fn check_and_disconnect_users(&self, ctx: &Context) -> Result<usize> {
        let guild_channel = self.get_voice_channel(ctx).await?;
        let members = self.get_voice_channel_members(ctx, &guild_channel).await?;

        if members.is_empty() {
            return Ok(0);
        }

        log_info(&format!(
            "{} membre(s) détecté(s) dans le salon '{}'",
            members.len(),
            guild_channel.name
        ));

        let mut disconnected_count = 0;

        for member in members {
            match self.disconnect_user(ctx, &member).await {
                Ok(()) => {
                    disconnected_count += 1;
                    self.log_disconnection_to_channel(ctx, &member.user.tag(), &guild_channel.name)
                        .await;
                }
                Err(e) => {
                    log_error(&format!(
                        "Impossible de déconnecter {}: {}",
                        member.user.tag(),
                        e
                    ));
                }
            }
        }

        Ok(disconnected_count)
    }

    /**
     * Récupère et valide le salon vocal configuré
     */
    async fn get_voice_channel(
        &self,
        ctx: &Context,
    ) -> Result<serenity::model::channel::GuildChannel> {
        let channel = self
            .config
            .voice_channel_id
            .to_channel(ctx)
            .await
            .context("Impossible de récupérer le salon")?;

        let Channel::Guild(guild_channel) = channel else {
            return Err(BotError::NotGuildChannel.into());
        };

        if guild_channel.kind != ChannelType::Voice {
            return Err(BotError::InvalidChannelType.into());
        }

        Ok(guild_channel)
    }

    /**
     * Récupère la liste des membres présents dans le salon vocal
     */
    async fn get_voice_channel_members(
        &self,
        ctx: &Context,
        channel: &serenity::model::channel::GuildChannel,
    ) -> Result<Vec<serenity::model::guild::Member>> {
        channel
            .members(ctx)
            .context("Impossible de récupérer les membres du salon")
    }

    /**
     * Déconnecte un utilisateur spécifique du salon vocal
     */
    async fn disconnect_user(
        &self,
        ctx: &Context,
        member: &serenity::model::guild::Member,
    ) -> Result<()> {
        let user_tag = member.user.tag();

        member
            .disconnect_from_voice(ctx)
            .await
            .context(format!("Échec de la déconnexion de {user_tag}"))?;

        log_info(&format!("✅ {user_tag} déconnecté avec succès"));
        Ok(())
    }

    /**
     * Log la déconnexion dans le canal de log configuré si disponible
     */
    async fn log_disconnection_to_channel(
        &self,
        ctx: &Context,
        user_tag: &str,
        channel_name: &str,
    ) {
        if let Some(log_channel_id) = self.config.log_channel_id {
            let log_message = format!("🔇 {user_tag} déconnecté du salon '{channel_name}'");

            if let Err(e) = log_channel_id.say(ctx, log_message).await {
                log_error(&format!("Impossible d'envoyer le log Discord: {e}"));
            }
        }
    }
}

/**
 * Point d'entrée principal de l'application
 */
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let args = Args::parse();

    // Gestion de l'import
    if let Some(import_file) = &args.import_from {
        return import_configuration(import_file, &args.config_file).await;
    }

    // Gestion de l'export
    if let Some(export_file) = &args.export_to {
        return export_configuration(&args.config_file, export_file).await;
    }

    let config =
        load_or_create_configuration(&args).context("Impossible de charger la configuration")?;

    let token = get_discord_token().context("Token Discord requis")?;

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(&token, intents)
        .event_handler(Bot::new(config))
        .await
        .context("Erreur lors de la création du client Discord")?;

    log_info("🚀 Démarrage du bot...");

    client
        .start()
        .await
        .context("Erreur lors du démarrage du bot")?;

    Ok(())
}

/**
 * Charge la configuration depuis les arguments CLI ou le fichier de config
 */
fn load_or_create_configuration(args: &Args) -> Result<BotConfig> {
    // Essayer de charger depuis le fichier existant
    if Path::new(&args.config_file).exists() {
        let config_content = fs::read_to_string(&args.config_file)
            .context("Impossible de lire le fichier de configuration")?;

        let mut config: BotConfig =
            serde_json::from_str(&config_content).context("Fichier de configuration invalide")?;

        // Remplacer par les arguments CLI si fournis
        if let Some(channel_id) = args.voice_channel_id {
            config.voice_channel_id = ChannelId::new(channel_id);
        }
        if let Some(log_id) = args.log_channel_id {
            config.log_channel_id = Some(ChannelId::new(log_id));
        }
        config.cron_schedule = args.cron_schedule.clone();

        // Sauvegarder la configuration mise à jour
        save_configuration(&config, &args.config_file)?;

        log_info(&format!(
            "Configuration chargée depuis {}",
            args.config_file
        ));
        return Ok(config);
    }

    // Créer une nouvelle configuration depuis les arguments CLI
    let voice_channel_id = args.voice_channel_id.ok_or_else(|| {
        BotError::MissingConfig(
            "ID du salon vocal requis (--channel ou configuration existante)".to_string(),
        )
    })?;

    let config = BotConfig {
        voice_channel_id: ChannelId::new(voice_channel_id),
        log_channel_id: args.log_channel_id.map(ChannelId::new),
        cron_schedule: args.cron_schedule.clone(),
    };

    // Sauvegarder la nouvelle configuration
    save_configuration(&config, &args.config_file)?;
    log_info(&format!(
        "Configuration créée et sauvegardée dans {}",
        args.config_file
    ));

    Ok(config)
}

/**
 * Sauvegarde la configuration dans un fichier JSON
 */
fn save_configuration(config: &BotConfig, file_path: &str) -> Result<()> {
    let config_json = serde_json::to_string_pretty(config)
        .context("Impossible de sérialiser la configuration")?;

    fs::write(file_path, config_json).context("Impossible d'écrire le fichier de configuration")?;

    Ok(())
}

/**
 * Exporte la configuration actuelle vers un fichier spécifique
 */
async fn export_configuration(source_file: &str, target_file: &str) -> Result<()> {
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

    // Afficher le contenu pour vérification
    let config_content = fs::read_to_string(target_file)?;
    let config: BotConfig = serde_json::from_str(&config_content)?;

    println!("\n📋 Configuration exportée :");
    println!("   • Salon vocal: {}", config.voice_channel_id);
    println!(
        "   • Salon de log: {}",
        config
            .log_channel_id
            .map_or("Aucun".to_string(), |id| id.to_string())
    );
    println!("   • Planning: {}", config.cron_schedule);
    println!("\n💡 Pour utiliser cette config ailleurs :");
    println!("   ./lekickerfou --import {target_file}");

    Ok(())
}

/**
 * Récupère le token Discord depuis les variables d'environnement
 */
fn get_discord_token() -> Result<String> {
    env::var("DISCORD_TOKEN")
        .map_err(|_| {
            println!("❓ Token Discord non trouvé dans DISCORD_TOKEN.");
            println!("💡 Vous pouvez :");
            println!("   1. Créer un fichier .env avec DISCORD_TOKEN=votre_token");
            println!("   2. Exporter la variable: export DISCORD_TOKEN=votre_token");
            println!("   3. Lancer avec: DISCORD_TOKEN=votre_token ./lekickerfou");

            BotError::MissingConfig("Token Discord requis".to_string())
        })
        .context("Token Discord manquant")
}

/**
 * Affiche un message d'information formaté avec horodatage
 */
fn log_info(msg: &str) {
    println!(
        "{} {} {}",
        "ℹ️".green(),
        Utc::now()
            .format("[%Y-%m-%d %H:%M:%S UTC]")
            .to_string()
            .dimmed(),
        msg
    );
}

/**
 * Affiche un message d'erreur formaté avec horodatage
 */
fn log_error(msg: &str) {
    eprintln!(
        "{} {} {}",
        "❌".red(),
        Utc::now()
            .format("[%Y-%m-%d %H:%M:%S UTC]")
            .to_string()
            .dimmed(),
        msg.red()
    );
}
