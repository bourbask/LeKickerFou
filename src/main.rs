//! Bot Discord pour déconnecter automatiquement les utilisateurs des salons vocaux

use anyhow::{Context, Result};
use clap::Parser;
use config::{Args, ConfigManager};
use serenity::{
    all::{CommandOptionType, CreateCommand, GatewayIntents, Interaction},
    client::{Client, Context as SerenityContext, EventHandler},
    model::gateway::Ready,
};
use std::env;
use tokio_cron_scheduler::{Job, JobScheduler};

mod config;
mod error;
mod handler;
mod history;
mod logging;
mod permissions;
mod scheduler;
mod utils;

use error::BotError;
use permissions::PermissionValidator;
use scheduler::execute_scheduled_kick;

/// Point d'entrée principal
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Configuration
    let config_manager = ConfigManager::new();
    let config = config_manager
        .load_or_create_configuration(&args)
        .context("Impossible de charger la configuration")?;

    println!("✅ Configuration chargée");
    println!("🔊 Salon surveillé: {}", config.voice_channel_id);
    println!("⏰ Planning: {}", config.cron_schedule);

    // Token Discord
    let token = env::var("DISCORD_TOKEN").context("Variable DISCORD_TOKEN manquante")?;

    // Client Discord
    let intents = GatewayIntents::GUILD_VOICE_STATES | GatewayIntents::GUILD_MESSAGES;

    let mut client = Client::builder(token, intents)
        .event_handler(DiscordEventHandler::new(config.clone()))
        .await
        .context("Impossible de créer le client Discord")?;

    println!("🚀 Démarrage du bot...");

    if let Err(e) = client.start().await {
        return Err(BotError::DiscordError(format!("Erreur du client: {}", e)).into());
    }

    Ok(())
}

/// Handler pour les événements Discord
struct DiscordEventHandler {
    config: config::BotConfig,
    scheduler_started: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl DiscordEventHandler {
    fn new(config: config::BotConfig) -> Self {
        Self {
            config,
            scheduler_started: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}

#[serenity::async_trait]
impl EventHandler for DiscordEventHandler {
    async fn ready(&self, ctx: SerenityContext, ready: Ready) {
        println!("✅ Bot connecté: {}", ready.user.tag());

        // Enregistrer les commandes slash
        let commands = vec![
            CreateCommand::new("status").description("Affiche le statut du bot"),
            CreateCommand::new("kick").description("Déconnecte manuellement tous les utilisateurs"),
            CreateCommand::new("permissions")
                .description("Gestion des permissions (Admin uniquement)")
                .add_option(serenity::all::CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "list",
                    "Affiche la liste complète des permissions",
                ))
                .add_option(
                    serenity::all::CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "add-user",
                        "Ajoute un utilisateur à la whitelist",
                    )
                    .add_sub_option(
                        serenity::all::CreateCommandOption::new(
                            CommandOptionType::User,
                            "user",
                            "L'utilisateur à ajouter",
                        )
                        .required(true),
                    )
                    .add_sub_option(
                        serenity::all::CreateCommandOption::new(
                            CommandOptionType::String,
                            "level",
                            "Niveau de permission",
                        )
                        .required(true)
                        .add_string_choice("User", "User")
                        .add_string_choice("Moderator", "Moderator")
                        .add_string_choice("Admin", "Admin"),
                    ),
                )
                .add_option(
                    serenity::all::CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "add-role",
                        "Ajoute un rôle à la whitelist",
                    )
                    .add_sub_option(
                        serenity::all::CreateCommandOption::new(
                            CommandOptionType::Role,
                            "role",
                            "Le rôle à ajouter",
                        )
                        .required(true),
                    )
                    .add_sub_option(
                        serenity::all::CreateCommandOption::new(
                            CommandOptionType::String,
                            "level",
                            "Niveau de permission",
                        )
                        .required(true)
                        .add_string_choice("User", "User")
                        .add_string_choice("Moderator", "Moderator")
                        .add_string_choice("Admin", "Admin"),
                    ),
                )
                .add_option(
                    serenity::all::CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "remove-user",
                        "Supprime un utilisateur de la whitelist",
                    )
                    .add_sub_option(
                        serenity::all::CreateCommandOption::new(
                            CommandOptionType::User,
                            "user",
                            "L'utilisateur à supprimer",
                        )
                        .required(true),
                    ),
                )
                .add_option(
                    serenity::all::CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "remove-role",
                        "Supprime un rôle de la whitelist",
                    )
                    .add_sub_option(
                        serenity::all::CreateCommandOption::new(
                            CommandOptionType::Role,
                            "role",
                            "Le rôle à supprimer",
                        )
                        .required(true),
                    ),
                )
                .add_option(
                    serenity::all::CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "check",
                        "Vérifie les permissions d'un utilisateur",
                    )
                    .add_sub_option(
                        serenity::all::CreateCommandOption::new(
                            CommandOptionType::User,
                            "user",
                            "L'utilisateur à vérifier",
                        )
                        .required(true),
                    ),
                ),
        ];

        if let Err(e) = ctx.http.create_global_commands(&commands).await {
            eprintln!("❌ Erreur lors de l'enregistrement des commandes: {}", e);
        } else {
            println!("✅ Commandes slash enregistrées");
        }

        // Démarrer le scheduler (une seule fois)
        if !self
            .scheduler_started
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            self.scheduler_started
                .store(true, std::sync::atomic::Ordering::Relaxed);

            let config_clone = self.config.clone();
            let ctx_clone = ctx.clone();

            tokio::spawn(async move {
                if let Err(e) = start_scheduler(ctx_clone, config_clone).await {
                    eprintln!("❌ Erreur du scheduler: {}", e);
                }
            });
        }
    }

    async fn interaction_create(&self, ctx: SerenityContext, interaction: Interaction) {
        let permission_validator = PermissionValidator::new();

        if let Err(e) = handler::handle_interaction(&ctx, &interaction, &permission_validator).await
        {
            eprintln!("Erreur interaction: {}", e);
        }
    }
}

/// Démarre le scheduler cron pour les déconnexions automatiques
async fn start_scheduler(ctx: SerenityContext, config: config::BotConfig) -> Result<()> {
    println!(
        "⏰ Démarrage du scheduler avec planning: {}",
        config.cron_schedule
    );

    let scheduler = JobScheduler::new().await?;

    // Cloner les valeurs avant de les utiliser dans la closure
    let cron_schedule = config.cron_schedule.clone();

    let job = Job::new_async(cron_schedule.as_str(), move |_uuid, _l| {
        let ctx = ctx.clone();
        let config = config.clone();

        Box::pin(async move {
            match execute_scheduled_kick(&ctx, &config).await {
                Ok(count) => {
                    if count > 0 {
                        println!("✅ Scheduler: {} utilisateurs déconnectés", count);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Erreur scheduler: {}", e);
                }
            }
        })
    })?;

    scheduler.add(job).await?;
    scheduler.start().await?;

    println!("✅ Scheduler démarré avec succès");

    // Maintenir le scheduler en vie
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
