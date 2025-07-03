//! Gestionnaire des salons vocaux et déconnexions automatiques.

use anyhow::{Context, Result};
use serenity::{
    client::Context as SerenityContext,
    model::{
        channel::GuildChannel,
        channel::{Channel, ChannelType},
        guild::Member,
    },
};

use crate::{
    config::BotConfig,
    utils::{log_error, log_info},
    BotError,
};

use super::warning::WarningManager;

/// Gestionnaire responsable de la surveillance et de la déconnexion des utilisateurs
pub struct VoiceChannelManager {
    config: BotConfig,
    warning_manager: WarningManager,
}

impl VoiceChannelManager {
    /// Crée une nouvelle instance du gestionnaire de salons vocaux
    pub fn new(config: BotConfig) -> Self {
        let warning_manager = WarningManager::new(config.clone());
        Self {
            config,
            warning_manager,
        }
    }

    /// Vérifie le salon vocal configuré et gère avertissement puis déconnexion
    pub async fn check_and_disconnect_users(&self, ctx: &SerenityContext) -> Result<usize> {
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

        // Phase 1: Envoyer l'avertissement si configuré
        if self.config.has_warnings_enabled() {
            let warning_sent = self
                .warning_manager
                .send_warning(ctx, &members, &guild_channel.name)
                .await;

            if warning_sent {
                // Attendre le délai configuré
                self.warning_manager.wait_warning_delay().await;

                // Vérifier si on doit s'arrêter là (mode warning-only)
                if self.config.is_warning_only_mode() {
                    log_info("Mode avertissement uniquement - Aucune déconnexion effectuée");
                    return Ok(0);
                }

                // Re-vérifier qui est encore présent après le délai
                let remaining_members = self.get_voice_channel_members(ctx, &guild_channel).await?;

                if remaining_members.is_empty() {
                    log_info(
                        "Tous les utilisateurs ont quitté d'eux-mêmes après l'avertissement !",
                    );
                    return Ok(0);
                }

                log_info(&format!(
                    "{} utilisateur(s) toujours présent(s) après l'avertissement - Début des déconnexions",
                    remaining_members.len()
                ));

                return self
                    .disconnect_members(ctx, &remaining_members, &guild_channel.name)
                    .await;
            }
        }

        // Phase 2: Déconnexion directe (si pas d'avertissement ou échec d'envoi)
        if !self.config.is_warning_only_mode() {
            self.disconnect_members(ctx, &members, &guild_channel.name)
                .await
        } else {
            Ok(0)
        }
    }

    /// Déconnecte une liste de membres et log les résultats
    async fn disconnect_members(
        &self,
        ctx: &SerenityContext,
        members: &[Member],
        channel_name: &str,
    ) -> Result<usize> {
        let mut disconnected_count = 0;

        for member in members {
            match self.disconnect_user(ctx, member).await {
                Ok(()) => {
                    disconnected_count += 1;
                    self.log_disconnection_to_channel(ctx, &member.user.tag(), channel_name)
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

    /// Récupère et valide le salon vocal configuré
    async fn get_voice_channel(&self, ctx: &SerenityContext) -> Result<GuildChannel> {
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

    /// Récupère la liste des membres présents dans le salon vocal
    async fn get_voice_channel_members(
        &self,
        ctx: &SerenityContext,
        channel: &GuildChannel,
    ) -> Result<Vec<Member>> {
        channel
            .members(ctx)
            .context("Impossible de récupérer les membres du salon")
    }

    /// Déconnecte un utilisateur spécifique du salon vocal
    async fn disconnect_user(&self, ctx: &SerenityContext, member: &Member) -> Result<()> {
        let user_tag = member.user.tag();

        member
            .disconnect_from_voice(ctx)
            .await
            .context(format!("Échec de la déconnexion de {user_tag}"))?;

        log_info(&format!("✅ {user_tag} déconnecté avec succès"));
        Ok(())
    }

    /// Log la déconnexion dans le canal de log configuré si disponible
    async fn log_disconnection_to_channel(
        &self,
        ctx: &SerenityContext,
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
