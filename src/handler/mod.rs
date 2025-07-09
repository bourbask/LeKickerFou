//! Module de gestion des interactions et événements Discord.

pub mod commands;

use anyhow::Result;
use serenity::{
    all::{CreateInteractionResponse, CreateInteractionResponseMessage, Interaction},
    client::Context as SerenityContext,
};

use crate::{config::BotConfig, permissions::PermissionValidator, utils::log_error};

pub use commands::*;

/// Handler principal pour les événements Discord
#[allow(dead_code)]
pub struct BotHandler {
    config: BotConfig,
    permission_validator: PermissionValidator,
}

impl BotHandler {
    /// Crée une nouvelle instance du handler
    #[allow(dead_code)]
    pub fn new(config: BotConfig, permission_validator: PermissionValidator) -> Self {
        Self {
            config,
            permission_validator,
        }
    }
}

/// Gère une interaction Discord (commande slash, bouton, etc.)
pub async fn handle_interaction(
    ctx: &SerenityContext,
    interaction: &Interaction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    match interaction {
        Interaction::Command(command) => {
            handle_slash_command(ctx, command, permission_validator).await
        }
        _ => {
            log_error("Type d'interaction non supporté reçu");
            Ok(())
        }
    }
}

/// Envoie une réponse d'erreur à une interaction
pub async fn send_error_response(
    ctx: &SerenityContext,
    interaction: &Interaction,
    error_message: &str,
) -> Result<()> {
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(format!("❌ **Erreur**\n{}", error_message))
            .ephemeral(true),
    );

    match interaction {
        Interaction::Command(command) => {
            command.create_response(&ctx.http, response).await?;
        }
        _ => {
            log_error("Impossible de répondre à ce type d'interaction");
        }
    }

    Ok(())
}
