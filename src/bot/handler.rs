//! Gestionnaire d'événements Discord principal.

use chrono::{Local, NaiveTime, TimeZone};
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
    pub fn new(config: BotConfig) -> Self {
        Self { config }
    }

    /// Calcule la prochaine occurrence de l'heure de couvre-feu
    fn calculate_next_curfew_time(&self, curfew_time: NaiveTime) -> chrono::DateTime<Local> {
        let now = Local::now();
        let today_curfew = now.date_naive().and_time(curfew_time);

        let next_curfew = if now.time() >= curfew_time {
            // Si on a dépassé l'heure aujourd'hui, programmer pour demain
            let tomorrow = now.date_naive() + chrono::Days::new(1);
            tomorrow.and_time(curfew_time)
        } else {
            // Sinon programmer pour aujourd'hui
            today_curfew
        };

        Local.from_local_datetime(&next_curfew)
            .single()
            .expect("Heure invalide pour combinaison")
    }

    /// Calcule l'heure d'avertissement initial (couvre-feu - délai)
    fn calculate_initial_warning_time(&self, curfew_time: chrono::DateTime<Local>) -> Option<chrono::DateTime<Local>> {
        if !self.config.has_warnings_enabled() {
            return None;
        }

        let warning_delay = chrono::Duration::seconds(self.config.warning_delay_seconds as i64);
        Some(curfew_time - warning_delay)
    }

    /// Démarre la surveillance avec planification des tâches
    async fn start_curfew_monitoring(&self, ctx: SerenityContext) -> anyhow::Result<()> {
        let next_curfew = self.calculate_next_curfew_time(self.config.curfew_time);
        
        log_info(&format!(
            "Prochaine heure de couvre-feu programmée: {}",
            next_curfew.format("%Y-%m-%d %H:%M:%S")
        ));

        // Planifier l'avertissement initial si configuré
        if let Some(initial_warning_time) = self.calculate_initial_warning_time(next_curfew) {
            let config_warning = self.config.clone();
            let ctx_warning = ctx.clone();

            log_info(&format!(
                "Avertissement initial programmé pour: {}",
                initial_warning_time.format("%Y-%m-%d %H:%M:%S")
            ));

            // Calculer le délai jusqu'au warning initial
            let now = Local::now();
            let warning_delay = if initial_warning_time > now {
                (initial_warning_time - now).to_std().unwrap_or_default()
            } else {
                std::time::Duration::from_secs(0)
            };

            if warning_delay > std::time::Duration::from_secs(0) {
                tokio::spawn(async move {
                    tokio::time::sleep(warning_delay).await;
                    let manager = VoiceChannelManager::new(config_warning);
                    if let Err(e) = manager.send_initial_warning_if_needed(&ctx_warning).await {
                        log_error(&format!("Erreur lors de l'envoi d'avertissement initial: {e}"));
                    }
                });
            }
        }

        // Planifier le couvre-feu principal
        let config_curfew = self.config.clone();
        let ctx_curfew = ctx;
        let now = Local::now();
        let curfew_delay = if next_curfew > now {
            (next_curfew - now).to_std().unwrap_or_default()
        } else {
            std::time::Duration::from_secs(0)
        };

        tokio::spawn(async move {
            if curfew_delay > std::time::Duration::from_secs(0) {
                tokio::time::sleep(curfew_delay).await;
            }
            
            let manager = VoiceChannelManager::new(config_curfew.clone());

            match manager.handle_curfew_time(&ctx_curfew).await {
                Ok(disconnected_count) => {
                    if disconnected_count > 0 {
                        log_info(&format!("{disconnected_count} utilisateur(s) déconnecté(s) pour couvre-feu"));
                    } else if config_curfew.is_warning_only_mode() {
                        log_info("Couvre-feu en mode clément - Aucune déconnexion effectuée");
                    } else {
                        log_info("Aucun utilisateur à déconnecter - Salon vocal vide");
                    }
                }
                Err(e) => log_error(&format!("Erreur lors du couvre-feu: {e}")),
            }
        });

        log_info(&format!(
            "Surveillance démarrée - Couvre-feu à {} ({})",
            self.config.curfew_time.format("%H:%M"),
            if self.config.has_warnings_enabled() {
                format!("avec avertissement {} secondes avant", self.config.warning_delay_seconds)
            } else {
                "sans avertissement".to_string()
            }
        ));

        Ok(())
    }
}

#[async_trait]
impl EventHandler for BotHandler {
    /// Gestionnaire d'événement déclenché quand le bot est prêt
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

        if let Err(err) = self.start_curfew_monitoring(ctx).await {
            log_error(&format!(
                "Erreur lors du démarrage de la surveillance: {err}"
            ));
        }
    }
}
