//! Gestion des commandes slash Discord.

use anyhow::Result;
use serenity::{
    all::{
        CommandDataOptionValue, CommandInteraction, CreateInteractionResponse,
        CreateInteractionResponseMessage, EditInteractionResponse,
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

/// Parse le niveau de permission depuis une string
fn parse_permission_level(level_str: &str) -> Option<PermissionLevel> {
    match level_str {
        "User" => Some(PermissionLevel::User),
        "Moderator" => Some(PermissionLevel::Moderator),
        "Admin" => Some(PermissionLevel::Admin),
        _ => None,
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
                    EditInteractionResponse::new()
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
                    EditInteractionResponse::new().content(success_message),
                )
                .await?;
        }
        Err(e) => {
            command
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content(format!("❌ **Erreur lors de la déconnexion**\n{}", e)),
                )
                .await?;
        }
    }

    Ok(())
}

/// Commande /permissions - Gestion complète des permissions
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

    let subcommand = command
        .data
        .options
        .first()
        .ok_or_else(|| anyhow::anyhow!("Aucune sous-commande spécifiée"))?;

    match subcommand.name.as_str() {
        "list" => handle_permissions_list(ctx, command, permission_validator).await,
        "add-user" => handle_permissions_add_user(ctx, command, permission_validator).await,
        "add-role" => handle_permissions_add_role(ctx, command, permission_validator).await,
        "remove-user" => handle_permissions_remove_user(ctx, command, permission_validator).await,
        "remove-role" => handle_permissions_remove_role(ctx, command, permission_validator).await,
        "check" => handle_permissions_check(ctx, command, permission_validator).await,
        _ => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("❌ **Sous-commande inconnue**")
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
            Ok(())
        }
    }
}

/// Extrait les options d'une sous-commande
fn get_subcommand_options(
    command: &CommandInteraction,
) -> Result<&[serenity::all::CommandDataOption]> {
    let subcommand = command
        .data
        .options
        .first()
        .ok_or_else(|| anyhow::anyhow!("Aucune sous-commande"))?;

    match &subcommand.value {
        CommandDataOptionValue::SubCommand(options) => Ok(options),
        _ => Err(anyhow::anyhow!("Format de sous-commande invalide")),
    }
}

/// /permissions list - Affiche toutes les permissions
async fn handle_permissions_list(
    ctx: &SerenityContext,
    command: &CommandInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    let permissions = permission_validator.whitelist_manager().load_or_create()?;

    let mut content = format!(
        "📋 **Permissions LeKickerFou**\n\n\
        📊 **Statistiques:**\n\
        • Version: {}\n\
        • Utilisateurs: {}\n\
        • Rôles: {}\n\
        • Dernière modification: {}\n",
        permissions.version,
        permissions.metadata.total_users,
        permissions.metadata.total_roles,
        permissions
            .metadata
            .last_modified
            .format("%Y-%m-%d %H:%M:%S UTC")
    );

    if let Some(modified_by) = &permissions.metadata.modified_by {
        content.push_str(&format!("• Modifié par: {}\n", modified_by));
    }

    // Utilisateurs
    if !permissions.permissions.users.is_empty() {
        content.push_str("\n👥 **Utilisateurs autorisés:**\n");
        for (user_id, level) in &permissions.permissions.users {
            content.push_str(&format!("• <@{}> → {}\n", user_id, level));
        }
    } else {
        content.push_str("\n👥 **Utilisateurs:** Aucun\n");
    }

    // Rôles
    if !permissions.permissions.roles.is_empty() {
        content.push_str("\n🏷️ **Rôles autorisés:**\n");
        for (role_id, level) in &permissions.permissions.roles {
            content.push_str(&format!("• <@&{}> → {}\n", role_id, level));
        }
    } else {
        content.push_str("\n🏷️ **Rôles:** Aucun\n");
    }

    content.push_str("\n💡 **Niveaux de permission:**\n• 👤 User: Commandes de consultation\n• 🛡️ Moderator: + Déconnexions manuelles\n• 👑 Admin: + Gestion des permissions");

    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(content)
            .ephemeral(true),
    );

    command.create_response(&ctx.http, response).await?;
    Ok(())
}

/// /permissions add-user - Ajoute un utilisateur
async fn handle_permissions_add_user(
    ctx: &SerenityContext,
    command: &CommandInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    let options = get_subcommand_options(command)?;

    let user_id = match &options[0].value {
        CommandDataOptionValue::User(id) => *id,
        _ => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("❌ **Erreur:** Utilisateur invalide")
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
            return Ok(());
        }
    };

    let level_str = match &options[1].value {
        CommandDataOptionValue::String(s) => s,
        _ => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("❌ **Erreur:** Niveau invalide")
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
            return Ok(());
        }
    };

    let level = match parse_permission_level(level_str) {
        Some(l) => l,
        None => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("❌ **Erreur:** Niveau de permission invalide")
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
            return Ok(());
        }
    };

    // Récupérer le nom de l'utilisateur pour affichage
    let user_name = if let Ok(user) = ctx.http.get_user(user_id).await {
        user.tag()
    } else {
        format!("Utilisateur {}", user_id)
    };

    match permission_validator.whitelist_manager().add_user(
        user_id,
        level,
        Some(command.user.tag()),
    ) {
        Ok(_) => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!(
                        "✅ **Utilisateur ajouté**\n\
                        👤 **Utilisateur:** {}\n\
                        🔑 **Niveau:** {}\n\
                        👨‍💻 **Ajouté par:** {}",
                        user_name,
                        level,
                        command.user.tag()
                    ))
                    .ephemeral(false),
            );
            command.create_response(&ctx.http, response).await?;
        }
        Err(e) => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("❌ **Erreur lors de l'ajout**\n{}", e))
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
        }
    }

    Ok(())
}

/// /permissions add-role - Ajoute un rôle
async fn handle_permissions_add_role(
    ctx: &SerenityContext,
    command: &CommandInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    let options = get_subcommand_options(command)?;

    let role_id = match &options[0].value {
        CommandDataOptionValue::Role(id) => *id,
        _ => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("❌ **Erreur:** Rôle invalide")
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
            return Ok(());
        }
    };

    let level_str = match &options[1].value {
        CommandDataOptionValue::String(s) => s,
        _ => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("❌ **Erreur:** Niveau invalide")
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
            return Ok(());
        }
    };

    let level = match parse_permission_level(level_str) {
        Some(l) => l,
        None => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("❌ **Erreur:** Niveau de permission invalide")
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
            return Ok(());
        }
    };

    match permission_validator.whitelist_manager().add_role(
        role_id,
        level,
        Some(command.user.tag()),
    ) {
        Ok(_) => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!(
                        "✅ **Rôle ajouté**\n\
                        🏷️ **Rôle:** <@&{}>\n\
                        🔑 **Niveau:** {}\n\
                        👨‍💻 **Ajouté par:** {}",
                        role_id,
                        level,
                        command.user.tag()
                    ))
                    .ephemeral(false),
            );
            command.create_response(&ctx.http, response).await?;
        }
        Err(e) => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("❌ **Erreur lors de l'ajout**\n{}", e))
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
        }
    }

    Ok(())
}

/// /permissions remove-user - Supprime un utilisateur
async fn handle_permissions_remove_user(
    ctx: &SerenityContext,
    command: &CommandInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    let options = get_subcommand_options(command)?;

    let user_id = match &options[0].value {
        CommandDataOptionValue::User(id) => *id,
        _ => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("❌ **Erreur:** Utilisateur invalide")
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
            return Ok(());
        }
    };

    // Récupérer le nom de l'utilisateur pour affichage
    let user_name = if let Ok(user) = ctx.http.get_user(user_id).await {
        user.tag()
    } else {
        format!("Utilisateur {}", user_id)
    };

    match permission_validator
        .whitelist_manager()
        .remove_user(user_id, Some(command.user.tag()))
    {
        Ok(true) => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!(
                        "✅ **Utilisateur supprimé**\n\
                        👤 **Utilisateur:** {}\n\
                        👨‍💻 **Supprimé par:** {}",
                        user_name,
                        command.user.tag()
                    ))
                    .ephemeral(false),
            );
            command.create_response(&ctx.http, response).await?;
        }
        Ok(false) => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!(
                        "⚠️ **Utilisateur non trouvé**\n{} n'était pas dans la whitelist.",
                        user_name
                    ))
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
        }
        Err(e) => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("❌ **Erreur lors de la suppression**\n{}", e))
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
        }
    }

    Ok(())
}

/// /permissions remove-role - Supprime un rôle
async fn handle_permissions_remove_role(
    ctx: &SerenityContext,
    command: &CommandInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    let options = get_subcommand_options(command)?;

    let role_id = match &options[0].value {
        CommandDataOptionValue::Role(id) => *id,
        _ => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("❌ **Erreur:** Rôle invalide")
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
            return Ok(());
        }
    };

    match permission_validator
        .whitelist_manager()
        .remove_role(role_id, Some(command.user.tag()))
    {
        Ok(true) => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!(
                        "✅ **Rôle supprimé**\n\
                        🏷️ **Rôle:** <@&{}>\n\
                        👨‍💻 **Supprimé par:** {}",
                        role_id,
                        command.user.tag()
                    ))
                    .ephemeral(false),
            );
            command.create_response(&ctx.http, response).await?;
        }
        Ok(false) => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!(
                        "⚠️ **Rôle non trouvé**\n<@&{}> n'était pas dans la whitelist.",
                        role_id
                    ))
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
        }
        Err(e) => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("❌ **Erreur lors de la suppression**\n{}", e))
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
        }
    }

    Ok(())
}

/// /permissions check - Vérifie les permissions d'un utilisateur
async fn handle_permissions_check(
    ctx: &SerenityContext,
    command: &CommandInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    let options = get_subcommand_options(command)?;

    let user_id = match &options[0].value {
        CommandDataOptionValue::User(id) => *id,
        _ => {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("❌ **Erreur:** Utilisateur invalide")
                    .ephemeral(true),
            );
            command.create_response(&ctx.http, response).await?;
            return Ok(());
        }
    };

    // Créer une interaction fictive pour tester les permissions de cet utilisateur
    let mut test_command = command.clone();
    test_command.user.id = user_id;
    let test_interaction = serenity::all::Interaction::Command(test_command);

    // Vérifier les permissions pour chaque niveau
    let user_permission = permission_validator
        .validate_interaction_permission(ctx, &test_interaction, PermissionLevel::User)
        .await;
    let mod_permission = permission_validator
        .validate_interaction_permission(ctx, &test_interaction, PermissionLevel::Moderator)
        .await;
    let admin_permission = permission_validator
        .validate_interaction_permission(ctx, &test_interaction, PermissionLevel::Admin)
        .await;

    // Récupérer le nom de l'utilisateur
    let user_name = if let Ok(user) = ctx.http.get_user(user_id).await {
        user.tag()
    } else {
        format!("Utilisateur {}", user_id)
    };

    let permissions = permission_validator.whitelist_manager().load_or_create()?;

    let mut content = format!(
        "🔍 **Vérification des permissions**\n👤 **Utilisateur:** {}\n\n",
        user_name
    );

    // Permission directe utilisateur
    if let Some(level) = permissions.permissions.get_user_level(&user_id) {
        content.push_str(&format!("👤 **Permission directe:** {}\n", level));
    } else {
        content.push_str("👤 **Permission directe:** Aucune\n");
    }

    // Permissions par rôles
    if let Some(guild_id) = command.guild_id {
        match permission_validator
            .get_member_roles(ctx, guild_id, user_id)
            .await
        {
            Ok(roles) => {
                let mut role_permissions = Vec::new();
                for role_id in roles {
                    if let Some(level) = permissions.permissions.get_role_level(&role_id) {
                        role_permissions.push(format!("<@&{}> → {}", role_id, level));
                    }
                }
                if !role_permissions.is_empty() {
                    content.push_str(&format!(
                        "🏷️ **Rôles autorisés:**\n{}\n",
                        role_permissions.join("\n")
                    ));
                } else {
                    content.push_str("🏷️ **Rôles autorisés:** Aucun\n");
                }
            }
            Err(_) => content.push_str("🏷️ **Rôles:** Impossible de récupérer\n"),
        }
    }

    // Résultat final
    content.push_str("\n🎯 **Accès autorisé:**\n");
    content.push_str(&format!(
        "• 👤 User: {}\n",
        if user_permission.is_authorized() {
            "✅"
        } else {
            "❌"
        }
    ));
    content.push_str(&format!(
        "• 🛡️ Moderator: {}\n",
        if mod_permission.is_authorized() {
            "✅"
        } else {
            "❌"
        }
    ));
    content.push_str(&format!(
        "• 👑 Admin: {}\n",
        if admin_permission.is_authorized() {
            "✅"
        } else {
            "❌"
        }
    ));

    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(content)
            .ephemeral(true),
    );

    command.create_response(&ctx.http, response).await?;
    Ok(())
}
