//! Module de gestion des déconnexions programmées selon le planning cron.

use anyhow::Result;
use serenity::{
    client::Context as SerenityContext,
    model::{guild::Member, id::ChannelId},
};

use crate::{
    config::BotConfig,
    utils::{log_error, log_info},
};

/// Exécute une déconnexion programmée selon la configuration
pub async fn execute_scheduled_kick(ctx: &SerenityContext, config: &BotConfig) -> Result<usize> {
    log_info("🔍 Vérification programmée du salon vocal...");

    // Récupérer tous les membres connectés au salon vocal surveillé
    let connected_members = get_members_in_voice_channel(ctx, config.voice_channel_id).await?;

    if connected_members.is_empty() {
        log_info("✅ Aucun utilisateur connecté - Rien à faire");
        return Ok(0);
    }

    log_info(&format!(
        "🎯 {} utilisateur(s) trouvé(s) dans le salon",
        connected_members.len()
    ));

    // Envoyer avertissement si configuré
    if let Some(warning_channel_id) = config.warning_channel_id {
        send_warning(
            &ctx,
            warning_channel_id,
            &connected_members,
            config.warning_delay_seconds,
        )
        .await?;

        if !config.warning_only {
            log_info(&format!(
                "⏳ Attente de {} secondes avant déconnexion...",
                config.warning_delay_seconds
            ));
            tokio::time::sleep(std::time::Duration::from_secs(config.warning_delay_seconds)).await;
        }
    }

    // Si mode warning-only, ne pas déconnecter
    if config.warning_only {
        log_info("💌 Mode avertissement seul - Aucune déconnexion");
        return Ok(connected_members.len());
    }

    // Déconnecter tous les utilisateurs
    let mut disconnected_count = 0;
    for member in &connected_members {
        match disconnect_member(ctx, member).await {
            Ok(_) => {
                disconnected_count += 1;
                log_info(&format!("🔇 {} déconnecté", member.user.tag()));
            }
            Err(e) => {
                log_error(&format!(
                    "❌ Échec déconnexion {}: {}",
                    member.user.tag(),
                    e
                ));
            }
        }
    }

    // Log dans Discord si configuré
    if let Some(log_channel_id) = config.log_channel_id {
        log_to_discord_channel(
            ctx,
            log_channel_id,
            &format!(
            "✅ **Nettoyage automatique terminé**\n{}/{} utilisateurs déconnectés du salon <#{}>",
            disconnected_count,
            connected_members.len(),
            config.voice_channel_id
        ),
        )
        .await?;
    }

    log_info(&format!(
        "✅ Déconnexion terminée: {}/{} utilisateurs",
        disconnected_count,
        connected_members.len()
    ));
    Ok(disconnected_count)
}

/// Déconnecte manuellement tous les utilisateurs (commande slash)
pub async fn execute_manual_kick(
    ctx: &SerenityContext,
    config: &BotConfig,
    initiator_tag: &str,
) -> Result<usize> {
    log_info(&format!(
        "🎯 Déconnexion manuelle initiée par {}",
        initiator_tag
    ));

    let connected_members = get_members_in_voice_channel(ctx, config.voice_channel_id).await?;

    if connected_members.is_empty() {
        log_info("✅ Aucun utilisateur à déconnecter");
        return Ok(0);
    }

    let mut disconnected_count = 0;
    for member in &connected_members {
        match disconnect_member(ctx, member).await {
            Ok(_) => {
                disconnected_count += 1;
                log_info(&format!("🔇 {} déconnecté manuellement", member.user.tag()));
            }
            Err(e) => {
                log_error(&format!(
                    "❌ Échec déconnexion {}: {}",
                    member.user.tag(),
                    e
                ));
            }
        }
    }

    // Log dans Discord
    if let Some(log_channel_id) = config.log_channel_id {
        log_to_discord_channel(
            ctx,
            log_channel_id,
            &format!(
                "🎯 **Déconnexion manuelle par {}**\n{} utilisateurs déconnectés du salon <#{}>",
                initiator_tag, disconnected_count, config.voice_channel_id
            ),
        )
        .await?;
    }

    log_info(&format!(
        "✅ Déconnexion manuelle terminée: {} utilisateurs",
        disconnected_count
    ));
    Ok(disconnected_count)
}

/// Récupère tous les membres connectés à un salon vocal spécifique
async fn get_members_in_voice_channel(
    ctx: &SerenityContext,
    channel_id: ChannelId,
) -> Result<Vec<Member>> {
    let mut connected_members = Vec::new();

    // Parcourir tous les serveurs mis en cache
    for guild_id in ctx.cache.guilds() {
        if let Some(guild) = ctx.cache.guild(guild_id) {
            // Parcourir les états vocaux pour trouver les utilisateurs dans le bon salon
            for (user_id, voice_state) in &guild.voice_states {
                if let Some(user_channel_id) = voice_state.channel_id {
                    if user_channel_id == channel_id {
                        // Récupérer le membre complet
                        if let Some(member) = guild.members.get(user_id) {
                            // Ignorer les bots
                            if !member.user.bot {
                                connected_members.push(member.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(connected_members)
}

/// Déconnecte un membre du salon vocal
async fn disconnect_member(ctx: &SerenityContext, member: &Member) -> Result<()> {
    member.disconnect_from_voice(&ctx.http).await?;
    Ok(())
}

/// Envoie un avertissement dans le salon configuré
async fn send_warning(
    ctx: &SerenityContext,
    warning_channel_id: ChannelId,
    members: &[Member],
    delay_seconds: u64,
) -> Result<()> {
    let mentions: Vec<String> = members
        .iter()
        .map(|m| format!("<@{}>", m.user.id))
        .collect();

    let warning_messages = [
        "🚨 **ALERTE ÉVACUATION** 🚨",
        "⏰ **ATTENTION : Fermeture du salon vocal dans {} secondes !**",
        "🎭 **DERNIER SERVICE** : Le salon va bientôt fermer !",
        "🚁 **ÉVACUATION D'URGENCE** : Préparez-vous à l'éjection !",
        "🎯 **T-MINUS {} SECONDES** avant téléportation forcée !",
    ];

    let message_index = (chrono::Utc::now().timestamp() as usize) % warning_messages.len();
    let base_message = warning_messages[message_index];

    let final_message = format!(
        "{}\n\n{}\n\n⏳ **Vous avez {} secondes** pour quitter le salon vocal volontairement !\n🤖 Sinon, déconnexion automatique...",
        if base_message.contains("{}") {
            base_message.replace("{}", &delay_seconds.to_string())
        } else {
            base_message.to_string()
        },
        mentions.join(" "),
        delay_seconds
    );

    warning_channel_id.say(&ctx.http, final_message).await?;
    log_info(&format!(
        "📢 Avertissement envoyé à {} utilisateurs",
        members.len()
    ));

    Ok(())
}

/// Log un message dans un salon Discord
async fn log_to_discord_channel(
    ctx: &SerenityContext,
    channel_id: ChannelId,
    message: &str,
) -> Result<()> {
    channel_id.say(&ctx.http, message).await?;
    Ok(())
}
