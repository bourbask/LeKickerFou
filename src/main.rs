use anyhow::{Context, Result};
use clap::Parser;
use dotenv::dotenv;
use serenity::all::GatewayIntents;
use serenity::Client;

use lekickerfou::{
    bot::BotHandler,
    config::{Args, ConfigManager},
    utils::{get_discord_token, log_info},
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let args = Args::parse();
    let config_manager = ConfigManager::new();

    // Gestion de l'import de configuration
    if let Some(import_file) = &args.import_from {
        return config_manager
            .import_configuration(import_file, &args.config_file)
            .await;
    }

    // Gestion de l'export de configuration
    if let Some(export_file) = &args.export_to {
        return config_manager
            .export_configuration(&args.config_file, export_file)
            .await;
    }

    // Chargement de la configuration
    let config = config_manager
        .load_or_create_configuration(&args)
        .context("Impossible de charger la configuration")?;

    // Récupération du token Discord
    let token = get_discord_token().context("Token Discord requis")?;

    // Configuration des intents Discord
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;

    // Création du client Discord
    let mut client = Client::builder(&token, intents)
        .event_handler(BotHandler::new(config))
        .await
        .context("Erreur lors de la création du client Discord")?;

    log_info("🚀 Démarrage du bot...");

    client
        .start()
        .await
        .context("Erreur lors du démarrage du bot")?;

    Ok(())
}
