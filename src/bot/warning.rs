//! Module de gestion des avertissements avant déconnexion.

use rand::seq::IteratorRandom;
use serenity::{
    builder::{CreateEmbed, CreateMessage},
    client::Context as SerenityContext,
    model::{guild::Member, Colour},
};

use crate::{config::BotConfig, utils::{log_error, log_info}};

/// Collection de GIFs d'avertissement à rotation aléatoire
const WARNING_GIFS: &[&str] = &[
    "https://media.giphy.com/media/l3q2XhfQ8oCkm1Ts4/giphy.gif", // Homer Simpson qui dort
    "https://media.giphy.com/media/BLmBG0HPIoaoo/giphy.gif",     // Chat qui baille
    "https://media.giphy.com/media/3og0IMJcSI8p6hYQXS/giphy.gif", // Personne qui s'endort devant un ordi
    "https://media.giphy.com/media/KbhCl6QjGhsf48CvPW/giphy.gif", // Réveil qui sonne
    "https://media.giphy.com/media/l0MYOrDnJFJZT6NdC/giphy.gif",  // "Time to sleep" 
    "https://media.giphy.com/media/xT9IgAINmcY6UwRQc0/giphy.gif", // Horloge qui tourne
    "https://media.giphy.com/media/nnFwGHgE4Mk5W/giphy.gif",     // Bébé qui dort paisiblement
    "https://media.giphy.com/media/3o7ZeQBhbVGnELP4bK/giphy.gif", // "Good night" vintage
];

/// Messages d'avertissement à rotation aléatoire
const WARNING_MESSAGES: &[&str] = &[
    "🛏️ **C'est l'heure du dodo !** 🛏️\n\nL'heure du coucher a sonné ! Tous les noctambules encore en vocal, vous avez **{delay} secondes** pour dire bonne nuit avant que le marchand de sable ne vous emporte de force ! 😴✨",
    
    "🌙 **Alerte couvre-feu !** 🌙\n\nOyez oyez ! Il est maintenant l'heure officielle de rejoindre Morphée ! Les derniers résistants ont **{delay} secondes** pour quitter les lieux avant évacuation forcée ! 🚨💤",
    
    "🕐 **Tic-tac, tic-tac...** 🕐\n\nLe temps s'écoule et vos paupières s'alourdissent... Plus que **{delay} secondes** avant que le bot ne vous aide à aller au lit ! (Que vous le vouliez ou non 😈)",
    
    "🧙‍♂️ **Sortilège de sommeil imminent !** 🧙‍♂️\n\nAttention mes chers insomniaques ! Dans **{delay} secondes**, un puissant sort de déconnexion sera lancé sur ce salon ! Fuyez tant qu'il est encore temps ! ⚡",
    
    "🎭 **Dernier appel avant la fermeture !** 🎭\n\nComme au théâtre : 'Mesdames et Messieurs, dans **{delay} secondes**, le rideau tombera sur ce salon vocal. Merci de récupérer vos affaires et de quitter tranquillement les lieux !' 🎪",
    
    "🦸‍♂️ **Capitaine Couche-Tôt à votre service !** 🦸‍♂️\n\nJe vois encore des super-héros qui traînent dans les parages ! Vous avez **{delay} secondes** pour regagner votre QG (votre lit) avant que je n'intervienne ! 💪😴"
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
            .footer(serenity::builder::CreateEmbedFooter::new(
                format!("Salon concerné: {} | Bot LeKickerFou", voice_channel_name)
            ))
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
                    "Impossible d'envoyer l'avertissement dans le salon: {}",
                    e
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

        format!("{}\n\n{}\n\n{}", message, user_list, action_text)
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
        log_info(&format!("⏳ Attente de {} secondes avant action...", delay.as_secs()));
        tokio::time::sleep(delay).await;
    }
}
