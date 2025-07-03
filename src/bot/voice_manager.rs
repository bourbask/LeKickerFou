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

/// Gestionnaire responsable de la surveillance et de la déconnexion des utilisateurs
pub struct VoiceChannelManager {
    config: BotConfig,
}

impl VoiceChannelManager {
    /// Crée une nouvelle instance du gestionnaire de salons vocaux
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration contenant les paramètres du salon à surveiller
    pub fn new(config: BotConfig) -> Self {
        Self { config }
    }

    /// Vérifie le salon vocal configuré et déconnecte les utilisateurs si nécessaire
    ///
    /// # Arguments
    ///
    /// * `ctx` - Contexte Serenity pour les interactions Discord
    ///
    /// # Returns
    ///
    /// Le nombre d'utilisateurs déconnectés avec succès
    ///
    /// # Errors
    ///
    /// Retourne une erreur si :
    /// - Impossible de récupérer le salon vocal
    /// - Impossible de récupérer la liste des membres
    /// - Erreurs lors des déconnexions individuelles (non bloquantes)
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

    /// Récupère et valide le salon vocal configuré
    ///
    /// # Arguments
    ///
    /// * `ctx` - Contexte Serenity pour les interactions Discord
    ///
    /// # Returns
    ///
    /// Le salon vocal validé
    ///
    /// # Errors
    ///
    /// Retourne une erreur si :
    /// - Le salon est introuvable
    /// - Le salon n'est pas un salon de serveur Discord
    /// - Le salon n'est pas de type vocal
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
    ///
    /// # Arguments
    ///
    /// * `ctx` - Contexte Serenity pour les interactions Discord
    /// * `channel` - Le salon vocal dont récupérer les membres
    ///
    /// # Returns
    ///
    /// Liste des membres connectés au salon vocal
    ///
    /// # Errors
    ///
    /// Retourne une erreur si impossible de récupérer les membres via l'API Discord
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
    ///
    /// # Arguments
    ///
    /// * `ctx` - Contexte Serenity pour les interactions Discord
    /// * `member` - Le membre à déconnecter
    ///
    /// # Errors
    ///
    /// Retourne une erreur si la déconnexion échoue via l'API Discord
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
    ///
    /// # Arguments
    ///
    /// * `ctx` - Contexte Serenity pour les interactions Discord
    /// * `user_tag` - Tag de l'utilisateur déconnecté (nom#discriminant)
    /// * `channel_name` - Nom du salon vocal d'où l'utilisateur a été déconnecté
    ///
    /// # Note
    ///
    /// Cette méthode ne retourne pas d'erreur pour éviter d'interrompre le processus
    /// de déconnexion en cas de problème avec les logs Discord.
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
