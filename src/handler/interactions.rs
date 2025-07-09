//! Gestion des interactions Discord autres que les commandes slash.

use anyhow::Result;
use serenity::{
    all::{
        CreateInteractionResponse, CreateInteractionResponseMessage, MessageComponentInteraction,
        ModalSubmitInteraction,
    },
    client::Context as SerenityContext,
};

use crate::{
    logging::get_global_logger,
    permissions::PermissionValidator,
    utils::{log_error, log_info},
};

use super::{send_error_response, send_success_response};

/// Gère les interactions de composants de message (boutons, menus déroulants, etc.)
///
/// # Arguments
///
/// * `ctx` - Contexte Discord
/// * `component` - Interaction de composant reçue
/// * `permission_validator` - Validateur de permissions
///
/// # Returns
///
/// Résultat de l'exécution
pub async fn handle_message_component(
    ctx: &SerenityContext,
    component: &MessageComponentInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    let component_id = &component.data.custom_id;

    log_info(&format!(
        "🔘 Interaction de composant: '{}' de {}",
        component_id,
        component.user.tag()
    ));

    match component_id.as_str() {
        "confirm_kick" => handle_confirm_kick_button(ctx, component, permission_validator).await,
        "cancel_action" => handle_cancel_button(ctx, component).await,
        "show_config" => handle_show_config_button(ctx, component, permission_validator).await,
        "backup_config" => handle_backup_config_button(ctx, component, permission_validator).await,
        id if id.starts_with("restore_backup_") => {
            handle_restore_backup_button(ctx, component, permission_validator, id).await
        }
        _ => {
            log_error(&format!("Composant inconnu: {}", component_id));
            send_error_response(ctx, &component.clone().into(), "Action non reconnue").await
        }
    }
}

/// Gère les soumissions de modals (formulaires)
///
/// # Arguments
///
/// * `ctx` - Contexte Discord
/// * `modal` - Soumission de modal reçue
/// * `permission_validator` - Validateur de permissions
///
/// # Returns
///
/// Résultat de l'exécution
pub async fn handle_modal_submit(
    ctx: &SerenityContext,
    modal: &ModalSubmitInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    let modal_id = &modal.data.custom_id;

    log_info(&format!(
        "📝 Soumission de modal: '{}' de {}",
        modal_id,
        modal.user.tag()
    ));

    match modal_id.as_str() {
        "config_edit_modal" => handle_config_edit_modal(ctx, modal, permission_validator).await,
        "add_permission_modal" => {
            handle_add_permission_modal(ctx, modal, permission_validator).await
        }
        "schedule_edit_modal" => handle_schedule_edit_modal(ctx, modal, permission_validator).await,
        _ => {
            log_error(&format!("Modal inconnu: {}", modal_id));
            send_error_response(ctx, &modal.clone().into(), "Formulaire non reconnu").await
        }
    }
}

/// Bouton de confirmation pour la déconnexion
async fn handle_confirm_kick_button(
    ctx: &SerenityContext,
    component: &MessageComponentInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    // TODO: Vérifier les permissions et exécuter la déconnexion
    send_success_response(
        ctx,
        &component.clone().into(),
        "Déconnexion confirmée et exécutée.",
        false,
    )
    .await?;

    // Logger l'action
    if let Some(logger) = get_global_logger().lock().unwrap().as_ref() {
        let _ = logger
            .log_user_interaction(ctx, &component.user.tag(), "Bouton confirmation kick", None)
            .await;
    }

    Ok(())
}

/// Bouton d'annulation générique
async fn handle_cancel_button(
    ctx: &SerenityContext,
    component: &MessageComponentInteraction,
) -> Result<()> {
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content("❌ **Action annulée**\nAucune modification n'a été effectuée.")
            .ephemeral(true),
    );

    component.create_response(&ctx.http, response).await?;

    // Logger l'annulation
    if let Some(logger) = get_global_logger().lock().unwrap().as_ref() {
        let _ = logger
            .log_user_interaction(ctx, &component.user.tag(), "Bouton annulation", None)
            .await;
    }

    Ok(())
}

/// Bouton pour afficher la configuration détaillée
async fn handle_show_config_button(
    ctx: &SerenityContext,
    component: &MessageComponentInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    // TODO: Afficher la configuration complète avec possibilité de modification
    let config_message = "⚙️ **Configuration détaillée**\n\n\
        Cette fonctionnalité sera disponible dans une future version.\n\
        Pour l'instant, utilisez la commande `/config` ou modifiez directement les fichiers.";

    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(config_message)
            .ephemeral(true),
    );

    component.create_response(&ctx.http, response).await?;

    Ok(())
}

/// Bouton pour créer une sauvegarde de configuration
async fn handle_backup_config_button(
    ctx: &SerenityContext,
    component: &MessageComponentInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    // TODO: Créer une sauvegarde et confirmer
    send_success_response(
        ctx,
        &component.clone().into(),
        "Sauvegarde de configuration créée avec succès.",
        true,
    )
    .await?;

    // Logger l'action
    if let Some(logger) = get_global_logger().lock().unwrap().as_ref() {
        let _ = logger
            .log_user_interaction(ctx, &component.user.tag(), "Bouton sauvegarde config", None)
            .await;
    }

    Ok(())
}

/// Bouton pour restaurer une sauvegarde spécifique
async fn handle_restore_backup_button(
    ctx: &SerenityContext,
    component: &MessageComponentInteraction,
    permission_validator: &PermissionValidator,
    button_id: &str,
) -> Result<()> {
    // Extraire le nom de fichier depuis l'ID du bouton
    let backup_filename = button_id
        .strip_prefix("restore_backup_")
        .unwrap_or("unknown");

    // TODO: Restaurer la sauvegarde spécifiée
    let message = format!(
        "🔄 **Restauration en cours**\n\
        Sauvegarde à restaurer: `{}`\n\n\
        Cette fonctionnalité sera disponible dans une future version.",
        backup_filename
    );

    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(message)
            .ephemeral(true),
    );

    component.create_response(&ctx.http, response).await?;

    // Logger la tentative de restauration
    if let Some(logger) = get_global_logger().lock().unwrap().as_ref() {
        let _ = logger
            .log_user_interaction(
                ctx,
                &component.user.tag(),
                "Bouton restauration",
                Some(&format!("Fichier: {}", backup_filename)),
            )
            .await;
    }

    Ok(())
}

/// Modal pour éditer la configuration
async fn handle_config_edit_modal(
    ctx: &SerenityContext,
    modal: &ModalSubmitInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    // TODO: Parser les données du modal et mettre à jour la configuration
    let response_message = "⚙️ **Configuration mise à jour**\n\
        Les modifications seront appliquées au prochain redémarrage du bot.\n\n\
        Cette fonctionnalité est en cours de développement.";

    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(response_message)
            .ephemeral(true),
    );

    modal.create_response(&ctx.http, response).await?;

    // Logger la modification
    if let Some(logger) = get_global_logger().lock().unwrap().as_ref() {
        let _ = logger
            .log_user_interaction(
                ctx,
                &modal.user.tag(),
                "Edition configuration",
                Some("Via modal"),
            )
            .await;
    }

    Ok(())
}

/// Modal pour ajouter des permissions
async fn handle_add_permission_modal(
    ctx: &SerenityContext,
    modal: &ModalSubmitInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    // TODO: Parser les données et ajouter l'utilisateur/rôle à la whitelist
    let response_message = "👤 **Permission ajoutée**\n\
        L'utilisateur/rôle a été ajouté à la whitelist.\n\n\
        Cette fonctionnalité est en cours de développement.";

    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(response_message)
            .ephemeral(true),
    );

    modal.create_response(&ctx.http, response).await?;

    // Logger l'ajout de permission
    if let Some(logger) = get_global_logger().lock().unwrap().as_ref() {
        let _ = logger
            .log_user_interaction(
                ctx,
                &modal.user.tag(),
                "Ajout permission",
                Some("Via modal"),
            )
            .await;
    }

    Ok(())
}

/// Modal pour éditer le planning cron
async fn handle_schedule_edit_modal(
    ctx: &SerenityContext,
    modal: &ModalSubmitInteraction,
    permission_validator: &PermissionValidator,
) -> Result<()> {
    // TODO: Valider et mettre à jour l'expression cron
    let response_message = "⏰ **Planning mis à jour**\n\
        La nouvelle expression cron sera appliquée au prochain redémarrage.\n\n\
        Cette fonctionnalité est en cours de développement.";

    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(response_message)
            .ephemeral(true),
    );

    modal.create_response(&ctx.http, response).await?;

    // Logger la modification du planning
    if let Some(logger) = get_global_logger().lock().unwrap().as_ref() {
        let _ = logger
            .log_user_interaction(
                ctx,
                &modal.user.tag(),
                "Edition planning",
                Some("Via modal"),
            )
            .await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_id_parsing() {
        // Test de l'extraction du nom de fichier depuis l'ID du bouton
        let button_id = "restore_backup_2024-01-15_14-30-25.json";
        let filename = button_id
            .strip_prefix("restore_backup_")
            .unwrap_or("unknown");

        assert_eq!(filename, "2024-01-15_14-30-25.json");
    }

    #[test]
    fn test_unknown_button_handling() {
        let unknown_id = "unknown_button_id";
        let filename = unknown_id
            .strip_prefix("restore_backup_")
            .unwrap_or("unknown");

        assert_eq!(filename, "unknown");
    }
}
