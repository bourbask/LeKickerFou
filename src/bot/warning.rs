//! Module de gestion des avertissements avant déconnexion.

use rand::seq::IteratorRandom;
use serenity::{
    builder::{CreateEmbed, CreateMessage},
    client::Context as SerenityContext,
    model::{guild::Member, Colour},
};

use crate::{
    config::BotConfig,
    utils::{log_error, log_info},
};

/// Collection de GIFs d'avertissement à rotation aléatoire
const WARNING_GIFS: &[&str] = &[
    "https://media3.giphy.com/media/v1.Y2lkPTc5MGI3NjExMmFrcjYxdzF1cjBmZjRxa3hoanFncG92bjhydXg2dWF3cTE2cnF4NyZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/mguPrVJAnEHIY/giphy.gif", // disney-disney-gif-pinocchio-jiminy-cricket
    "https://media3.giphy.com/media/v1.Y2lkPTc5MGI3NjExeHdsa2p3MDF3OHBteWUzZW9zcXc1OTV4cjRrbXZpZDUyY3RqeGM2NyZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/Bn9Wp6ryjMc9Qn1J9C/giphy.gif",     // travisband-travis-gnite-andy-dunlop
    "https://media1.giphy.com/media/v1.Y2lkPTc5MGI3NjExbWIxdXh2N3k0anV0cmVpbjJlZWs0YWYzbHZiZno5dDlhN210cWNjcyZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/ikckY3A4yfgX8IMrqr/giphy.gif", // uncanny-danny-go-to-sleep
    "https://media4.giphy.com/media/v1.Y2lkPTc5MGI3NjExNXdkbWU3eGtqY29hNGowNzMwYmljanpkcjVxZmE0cTEzdmQ4bW44YyZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/6cozGW0B5vZWQvAzfz/giphy.gif", // cameo-tired-sleep-parent
    "https://media0.giphy.com/media/v1.Y2lkPTc5MGI3NjExYWNkdDRsMzc0ZmczMmJzOGI2NHp3N3pzbG96b3M5Ynh1dWprNGt2NSZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/F0jAJOCyI1kA/giphy.gif",  // WWE "Time to sleep" 
    "https://media1.giphy.com/media/v1.Y2lkPTc5MGI3NjExZXc2b2h1bmE0azZodGNheTl0bTlxZ2ZvMnVycjA1MmEwZWMwZnVteSZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/uGRvUhuW1qdTrsMsj0/giphy.gif", // CoolCats-closed-shut-stay-out
    "https://media4.giphy.com/media/v1.Y2lkPTc5MGI3NjExdzIxYTZ4NGEycDNuc2RwaWdtMzdmMDE1N2k5cm04MjFmZTdncjQ5cSZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/l3q2wMdhTXm84vbaw/giphy.gif",     // cbc-funny-comedy
    "https://media2.giphy.com/media/v1.Y2lkPTc5MGI3NjExOTRnazhhcWFyeGEydXNsb2hzcXNzcHB0Ymc3Z3A3MXp2cTJwNXJsdCZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/j2qhdPCYIrWen8fzyX/giphy.gif", // halloween-staff-halloween-costumes
];

/// Messages d'avertissement à rotation aléatoire
const WARNING_MESSAGES: &[&str] = &[
    "⏰ **Bon allez, c'est l'heure !** ⏰\n\nIl est temps de déconnecter les gars. Vous avez **{delay} secondes** pour partir avant que je vous vire moi-même du vocal !",

    "🚨 **Dernière chance !** 🚨\n\nSérieusement, il faut aller dormir maintenant. Dans **{delay} secondes**, je kick tout le monde sans exception. Vous êtes prévenus !",

    "😴 **Allez au lit bordel !** 😴\n\nÇa fait des heures que vous êtes là-dessus ! Plus que **{delay} secondes** pour quitter le vocal, sinon c'est moi qui vous déconnecte de force !",

    "🔇 **Extinction des feux dans {delay} secondes** 🔇\n\nTout le monde dégage du vocal ! Plus personne ne doit traîner ici après ça !",

    "⚡ **Coupage imminent !** ⚡\n\nVous connaissez la chanson : il est tard, vous devez dormir. **{delay} secondes** pour partir gentiment avant que ça devienne moins sympa !",

    "🛑 **Stop, c'est fini !** 🛑\n\nLe vocal ferme dans **{delay} secondes**. Pas de négociation, pas d'exception. Tout le monde dehors !",

    "💀 **Vous allez morfler** 💀\n\nDans **{delay} secondes**, je vous dégage de là. Après dites pas que vous étiez pas prévenus !",

    "🎯 **Objectif : votre lit** 🎯\n\nVous avez **{delay} secondes** pour y aller par vous-mêmes. Sinon c'est moi qui vous aide à le retrouver !",

    "🔥 **Ça va chauffer !** 🔥\n\nDans **{delay} secondes**, je déconnecte tout ce petit monde. Maintenant vous savez ce qui vous attend !",

    "⚰️ **RIP vocal** ⚰️\n\nCe salon va mourir dans **{delay} secondes**. Évacuez tant qu'il est encore temps !"
];

/// Gestionnaire des messages d'avertissement
pub struct WarningManager {
    config: BotConfig,
}

impl WarningManager {
    /// Crée une nouvelle instance du gestionnaire d'avertissements
    pub fn new(config: BotConfig) -> Self {
        Self { config }
    }

    /// Envoie un avertissement aux utilisateurs présents dans le salon vocal
    ///
    /// # Arguments
    ///
    /// * `ctx` - Contexte Serenity pour les interactions Discord
    /// * `members` - Liste des membres présents dans le salon vocal
    /// * `voice_channel_name` - Nom du salon vocal concerné
    ///
    /// # Returns
    ///
    /// `true` si l'avertissement a été envoyé avec succès, `false` sinon
    pub async fn send_warning(
        &self,
        ctx: &SerenityContext,
        members: &[Member],
        voice_channel_name: &str,
    ) -> bool {
        let Some(warning_channel_id) = self.config.warning_channel_id else {
            return false;
        };

        if members.is_empty() {
            return false;
        }

        let warning_message = self.generate_warning_message(members);
        let gif_url = self.select_random_gif();

        let embed = CreateEmbed::new()
            .title("🚨 ALERTE COUVRE-FEU 🚨")
            .description(&warning_message)
            .color(Colour::from_rgb(255, 165, 0)) // Orange
            .image(gif_url)
            .footer(serenity::builder::CreateEmbedFooter::new(format!(
                "Salon concerné: {voice_channel_name} | Bot LeKickerFou"
            )))
            .timestamp(serenity::model::Timestamp::now());

        let message = CreateMessage::new().embed(embed).content({
            let mentions = members
                .iter()
                .map(|m| format!("<@{}>", m.user.id))
                .collect::<Vec<_>>()
                .join(" ");
            format!("👋 {mentions}")
        });

        match warning_channel_id.send_message(ctx, message).await {
            Ok(_) => {
                log_info(&format!(
                    "🚨 Avertissement envoyé à {} utilisateur(s) dans #{} via le salon d'avertissement",
                    members.len(),
                    voice_channel_name
                ));
                true
            }
            Err(e) => {
                log_error(&format!(
                    "Impossible d'envoyer l'avertissement dans le salon: {e}"
                ));
                false
            }
        }
    }

    /// Génère un message d'avertissement personnalisé
    fn generate_warning_message(&self, members: &[Member]) -> String {
        let mut rng = rand::rng();
        let base_message = WARNING_MESSAGES
            .iter()
            .choose(&mut rng)
            .unwrap_or(&WARNING_MESSAGES[0]);

        let delay = self.config.warning_delay_seconds;
        let message = base_message.replace("{delay}", &delay.to_string());

        let user_list = if members.len() == 1 {
            format!("**Cible désignée :** {}", members[0].display_name())
        } else {
            let names = members
                .iter()
                .map(|m| format!("• {}", m.display_name()))
                .collect::<Vec<_>>()
                .join("\n");
            format!("**Cibles désignées ({}) :**\n{}", members.len(), names)
        };

        let action_text = if self.config.is_warning_only_mode() {
            "💌 *Mode gentil activé - Aucune déconnexion ne sera effectuée, ceci n'est qu'un rappel amical !*"
        } else {
            "⚠️ *Cette menace est bien réelle ! Déconnexion automatique programmée !*"
        };

        format!("{message}\n\n{user_list}\n\n{action_text}")
    }

    /// Sélectionne un GIF aléatoire dans la collection
    fn select_random_gif(&self) -> &str {
        let mut rng = rand::rng();
        WARNING_GIFS
            .iter()
            .choose(&mut rng)
            .unwrap_or(&WARNING_GIFS[0])
    }

    /// Attendre le délai configuré avant de procéder à l'action
    pub async fn wait_warning_delay(&self) {
        let delay = self.config.warning_delay();
        log_info(&format!(
            "⏳ Attente de {} secondes avant action...",
            delay.as_secs()
        ));
        tokio::time::sleep(delay).await;
    }
}
