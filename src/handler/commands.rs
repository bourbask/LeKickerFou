//! Gestion des commandes slash Discord.

use anyhow::Result;
use serenity::{
    all::{
        CommandDataOptionValue, CommandInteraction, CommandOptionType, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
    client::Context as SerenityContext,
};

use crate::{
    config::ConfigManager,
    permissions::{PermissionLevel, PermissionResult, PermissionValidator},
    scheduler::execute_manual_kick,
    utils::{log_error, log_info},
};

/// Gère les commandes slash Discord
pub async fn handle_slash_command(
    ctx: &SerenityContext,
    command: &CommandInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    let command_name = &command.data.name;

    log_info(&format!(
        "🎯 Commande slash reçue: '{}' de {} ({})",
        command_name,
        command.user.tag(),
        command.user.id
    ));

    match command_name.as_str() {
        "status" => handle_status_command(ctx, command, permission_validator).await,
        "kick" => handle_kick_command(ctx, command, permission_validator).await,
        "permissions" => handle_permissions_command(ctx, command, permission_validator).await,
        _ => {
            log_error(&format!("Commande inconnue: {}", command_name));
            let interaction = serenity::all::Interaction::Command(command.clone());
            super::send_error_response(ctx, &interaction, "Commande inconnue").await
        }
    }
}

/// Vérifie les permissions avant d'exécuter une commande
async fn check_permission(
    ctx: &SerenityContext,
    command: &CommandInteraction,
    permission_validator: &PermissionValidator,
    required_level: PermissionLevel,
) -> Option<PermissionLevel> {
    let interaction = serenity::all::Interaction::Command(command.clone());

    match permission_validator
        .validate_interaction_permission(ctx, &interaction, required_level)
        .await
    {
        PermissionResult::Authorized(level) => {
            log_info(&format!(
                "✅ Permission accordée: {} pour {}",
                level,
                command.user.tag()
            ));
            Some(level)
        }
        PermissionResult::Unauthorized => {
            let message = permission_validator.permission_denied_message(required_level, None);
            let _ = super::send_error_response(ctx, &interaction, &message).await;
            None
        }
        PermissionResult::Error(err) => {
            log_error(&format!("Erreur validation permission: {}", err));
            let _ = super::send_error_response(
                ctx,
                &interaction,
                &format!("Erreur de permission: {}", err),
            )
            .await;
            None
        }
    }
}

/// Commande /status - Affiche le statut actuel du bot
async fn handle_status_command(
    ctx: &SerenityContext,
    command: &CommandInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    // Status accessible à tous
    if check_permission(ctx, command, permission_validator, PermissionLevel::User)
        .await
        .is_none()
    {
        return Ok(());
    }

    // Charger la config pour afficher les détails
    let config_manager = ConfigManager::new();
    let config = match config_manager.load_configuration_if_exists("bot_config.json") {
        Ok(config) => Some(config),
        Err(_) => None,
    };

    let status_message = if let Some(config) = config {
        format!(
            "🤖 **LeKickerFou v{}** - Bot actif\n\n\
            🔊 **Salon surveillé:** <#{}>\n\
            📝 **Salon de log:** {}\n\
            ⚠️ **Salon d'avertissement:** {}\n\
            ⏰ **Planning:** `{}`\n\
            💌 **Mode avertissement seul:** {}\n\
            ⏱️ **Délai d'avertissement:** {} secondes\n\n\
            🟢 **Bot opérationnel**",
            env!("CARGO_PKG_VERSION"),
            config.voice_channel_id,
            config
                .log_channel_id
                .map(|id| format!("<#{}>", id))
                .unwrap_or_else(|| "Aucun".to_string()),
            config
                .warning_channel_id
                .map(|id| format!("<#{}>", id))
                .unwrap_or_else(|| "Aucun".to_string()),
            config.cron_schedule,
            if config.warning_only {
                "✅ Oui"
            } else {
                "❌ Non"
            },
            config.warning_delay_seconds
        )
    } else {
        format!(
            "🤖 **LeKickerFou v{}** - Bot actif\n\n\
            ❌ **Aucune configuration trouvée**\n\
            🟢 **Bot opérationnel mais non configuré**",
            env!("CARGO_PKG_VERSION")
        )
    };

    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(status_message)
            .ephemeral(false),
    );

    command.create_response(&ctx.http, response).await?;
    Ok(())
}

/// Commande /kick - Déconnecte manuellement tous les utilisateurs du salon
async fn handle_kick_command(
    ctx: &SerenityContext,
    command: &CommandInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    // Kick nécessite au moins Moderator
    if check_permission(
        ctx,
        command,
        permission_validator,
        PermissionLevel::Moderator,
    )
    .await
    .is_none()
    {
        return Ok(());
    }

    // Répondre immédiatement pour éviter le timeout
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content("⏳ **Déconnexion en cours...**\nRecherche des utilisateurs connectés...")
            .ephemeral(false),
    );

    command.create_response(&ctx.http, response).await?;

    // Charger la configuration
    let config_manager = ConfigManager::new();
    let config = match config_manager.load_configuration_if_exists("bot_config.json") {
        Ok(config) => config,
        Err(e) => {
            command
                .edit_response(
                    &ctx.http,
                    serenity::all::EditInteractionResponse::new()
                        .content(format!("❌ **Erreur de configuration**\n{}", e)),
                )
                .await?;
            return Ok(());
        }
    };

    // Exécuter la déconnexion
    match execute_manual_kick(ctx, &config, &command.user.tag()).await {
        Ok(disconnected_count) => {
            let success_message = if disconnected_count == 0 {
                "✅ **Déconnexion terminée**\nAucun utilisateur n'était connecté au salon surveillé.".to_string()
            } else {
                format!(
                    "✅ **Déconnexion réussie**\n{} utilisateur(s) déconnecté(s) du salon <#{}>.",
                    disconnected_count, config.voice_channel_id
                )
            };

            command
                .edit_response(
                    &ctx.http,
                    serenity::all::EditInteractionResponse::new().content(success_message),
                )
                .await?;
        }
        Err(e) => {
            command
                .edit_response(
                    &ctx.http,
                    serenity::all::EditInteractionResponse::new()
                        .content(format!("❌ **Erreur lors de la déconnexion**\n{}", e)),
                )
                .await?;
        }
    }

    Ok(())
}

/// Commande /permissions - Gestion des permissions
async fn handle_permissions_command(
    ctx: &SerenityContext,
    command: &CommandInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    // Permissions nécessite Admin
    if check_permission(ctx, command, permission_validator, PermissionLevel::Admin)
        .await
        .is_none()
    {
        return Ok(());
    }

    // Pour l'instant, juste afficher la whitelist
    match permission_validator.whitelist_manager().display_summary() {
        Ok(_) => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("📋 **Résumé des permissions affiché dans la console**\nImplémentation complète à venir...")
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
        }
        Err(e) => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("❌ **Erreur**\n{}", e))
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
        }
    }

    Ok(())
}
