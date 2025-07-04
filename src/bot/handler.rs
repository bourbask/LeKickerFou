//! Gestionnaire d'événements Discord principal.

use anyhow::Context;
use serenity::{
    async_trait,
    client::{Context as SerenityContext, EventHandler},
    model::gateway::Ready,
};

use crate::{
    config::BotConfig,
    utils::{log_error, log_info},
};

use super::VoiceChannelManager;

/// Structure principale du bot Discord gérant les événements
pub struct BotHandler {
    config: BotConfig,
}

impl BotHandler {
    /// Crée une nouvelle instance du gestionnaire d'événements
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration du bot à utiliser
    pub fn new(config: BotConfig) -> Self {
        Self { config }
    }

    /// Démarre la surveillance des salons vocaux avec une tâche cron
    ///
    /// # Arguments
    ///
    /// * `ctx` - Contexte Serenity pour les interactions Discord
    ///
    /// # Errors
    ///
    /// Retourne une erreur si :
    /// - Impossible de créer le planificateur de tâches
    /// - Expression cron invalide
    /// - Impossible d'ajouter ou démarrer la tâche
    async fn start_voice_monitoring(&self, ctx: SerenityContext) -> anyhow::Result<()> {
        use tokio_cron_scheduler::{Job, JobScheduler};

        let scheduler = JobScheduler::new()
            .await
            .context("Impossible de créer le planificateur de tâches")?;

        let config = self.config.clone();
        let cron_expr = config.cron_schedule.clone();

        let job = Job::new_async(&cron_expr, move |_uuid, _scheduler| {
            let ctx_clone = ctx.clone();
            let config_clone = config.clone();

            Box::pin(async move {
                match VoiceChannelManager::new(config_clone)
                    .check_and_disconnect_users(&ctx_clone)
                    .await
                {
                    Ok(disconnected_count) => {
                        if disconnected_count > 0 {
                            log_info(&format!(
                                "{disconnected_count} utilisateur(s) déconnecté(s)"
                            ));
                        }
                    }
                    Err(e) => log_error(&format!("Erreur lors de la vérification: {e}")),
                }
            })
        })
        .context("Expression cron invalide")?;

        scheduler
            .add(job)
            .await
            .context("Impossible d'ajouter la tâche au planificateur")?;

        scheduler
            .start()
            .await
            .context("Impossible de démarrer le planificateur")?;

        log_info(&format!(
            "Surveillance des salons vocaux démarrée (planning: {})",
            self.config.cron_schedule
        ));
        Ok(())
    }
}

#[async_trait]
impl EventHandler for BotHandler {
    /// Gestionnaire d'événement déclenché quand le bot est prêt
    ///
    /// Initialise le système de tâches planifiées et affiche les informations de connexion.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Contexte Serenity pour les interactions Discord
    /// * `ready` - Informations sur l'état de connexion du bot
    async fn ready(&self, ctx: SerenityContext, ready: Ready) {
        log_info(&format!(
            "Bot connecté sous {} (ID: {})",
            ready.user.name, ready.user.id
        ));

        log_info(&format!(
            "Configuration: Canal vocal {}, Canal de log {}",
            self.config.voice_channel_id,
            self.config
                .log_channel_id
                .map_or("Aucun".to_string(), |id| id.to_string())
        ));

        if let Err(err) = self.start_voice_monitoring(ctx).await {
            log_error(&format!(
                "Erreur lors du démarrage de la surveillance: {err}"
            ));
        }
    }
}
