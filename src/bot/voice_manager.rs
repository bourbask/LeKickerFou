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

    /// Envoie l'avertissement initial si des utilisateurs sont présents
    pub async fn send_initial_warning_if_needed(&self, ctx: &SerenityContext) -> Result<()> {
        let guild_channel = self.get_voice_channel(ctx).await?;
        let members = self.get_voice_channel_members(ctx, &guild_channel).await?;

        if !members.is_empty() {
            log_info(&format!(
                "{} membre(s) détecté(s) pour avertissement initial dans '{}'",
                members.len(),
                guild_channel.name
            ));

            self.warning_manager
                .send_initial_warning(ctx, &members, &guild_channel.name)
                .await;
        } else {
            log_info("Aucun utilisateur présent pour l'avertissement initial");
        }

        Ok(())
    }

    /// Gère l'heure du couvre-feu (warning final + kick ou grâce)
    pub async fn handle_curfew_time(&self, ctx: &SerenityContext) -> Result<usize> {
        let guild_channel = self.get_voice_channel(ctx).await?;
        let members = self.get_voice_channel_members(ctx, &guild_channel).await?;

        if members.is_empty() {
            log_info("Aucun utilisateur présent à l'heure du couvre-feu");
            return Ok(0);
        }

        log_info(&format!(
            "{} membre(s) toujours présent(s) à l'heure du couvre-feu dans '{}'",
            members.len(),
            guild_channel.name
        ));

        if self.config.is_warning_only_mode() {
            // Mode clément : envoyer un message de grâce
            self.warning_manager
                .send_merciful_message(ctx, &members, &guild_channel.name)
                .await;
            
            log_info("Mode clément activé - Message de grâce envoyé");
            return Ok(0);
        }

        // Mode kick : avertissement final puis déconnexion
        log_info("Envoi de l'avertissement final...");
        self.warning_manager
            .send_final_warning(ctx, &members, &guild_channel.name)
            .await;

        // Attendre 10 secondes avant le kick
        log_info("⏳ Attente de 10 secondes avant déconnexion...");
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        // Vérifier qui est encore là et déconnecter
        let final_members = self.get_voice_channel_members(ctx, &guild_channel).await?;
        if final_members.is_empty() {
            log_info("Tous les utilisateurs ont quitté après l'avertissement final !");
            return Ok(0);
        }

        log_info(&format!(
            "{} utilisateur(s) toujours présent(s) - Début des déconnexions",
            final_members.len()
        ));

        self.disconnect_members(ctx, &final_members, &guild_channel.name)
            .await
    }

    /// Vérifie le salon vocal et déconnecte tous les utilisateurs présents (usage direct)
    pub async fn check_and_disconnect_users(&self, ctx: &SerenityContext) -> Result<usize> {
        let guild_channel = self.get_voice_channel(ctx).await?;
        let members = self.get_voice_channel_members(ctx, &guild_channel).await?;

        if members.is_empty() {
            return Ok(0);
        }

        log_info(&format!(
            "{} membre(s) détecté(s) dans le salon '{}' - Début des déconnexions directes",
            members.len(),
            guild_channel.name
        ));

        self.disconnect_members(ctx, &members, &guild_channel.name)
            .await
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
            let log_message = format!("🔇 {user_tag} déconnecté du salon '{channel_name}' (couvre-feu)");

            if let Err(e) = log_channel_id.say(ctx, log_message).await {
                log_error(&format!("Impossible d'envoyer le log Discord: {e}"));
            }
        }
    }
}
