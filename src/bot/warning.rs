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
    "https://media3.giphy.com/media/v1.Y2lkPTc5MGI3NjExMmFrcjYxdzF1cjBmZjRxa3hoanFncG92bjhydXg2dWF3cTE2cnF4NyZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/mguPrVJAnEHIY/giphy.gif", 
    "https://media3.giphy.com/media/v1.Y2lkPTc5MGI3NjExeHdsa2p3MDF3OHBteWUzZW9zcXc1OTV4cjRrbXZpZDUyY3RqeGM2NyZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/Bn9Wp6ryjMc9Qn1J9C/giphy.gif",     
    "https://media1.giphy.com/media/v1.Y2lkPTc5MGI3NjExbWIxdXh2N3k0anV0cmVpbjJlZWs0YWYzbHZiZno5dDlhN210cWNjcyZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/ikckY3A4yfgX8IMrqr/giphy.gif", 
    "https://media4.giphy.com/media/v1.Y2lkPTc5MGI3NjExNXdkbWU3eGtqY29hNGowNzMwYmljanpkcjVxZmE0cTEzdmQ4bW44YyZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/6cozGW0B5vZWQvAzfz/giphy.gif", 
    "https://media0.giphy.com/media/v1.Y2lkPTc5MGI3NjExYWNkdDRsMzc0ZmczMmJzOGI2NHp3N3pzbG96b3M5Ynh1dWprNGt2NSZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/F0jAJOCyI1kA/giphy.gif",  
    "https://media1.giphy.com/media/v1.Y2lkPTc5MGI3NjExZXc2b2h1bmE0azZodGNheTl0bTlxZ2ZvMnVycjA1MmEwZWMwZnVteSZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/uGRvUhuW1qdTrsMsj0/giphy.gif", 
    "https://media4.giphy.com/media/v1.Y2lkPTc5MGI3NjExdzIxYTZ4NGEycDNuc2RwaWdtMzdmMDE1N2k5cm04MjFmZTdncjQ5cSZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/l3q2wMdhTXm84vbaw/giphy.gif",     
    "https://media2.giphy.com/media/v1.Y2lkPTc5MGI3NjExOTRnazhhcWFyeGEydXNsb2hzcXNzcHB0Ymc3Z3A3MXp2cTJwNXJsdCZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/j2qhdPCYIrWen8fzyX/giphy.gif", 
];

/// GIFs pour l'avertissement final (plus dramatiques)
const FINAL_WARNING_GIFS: &[&str] = &[
    "https://media1.giphy.com/media/v1.Y2lkPTc5MGI3NjExbXVsdjF0c2I2ZndzMmRxNGtlNGZzZzVxeXZ6YmozNjB0cXEwbmtpciZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/3oKIPwoeGErMmaI43S/giphy.gif", // dramatic countdown
    "https://media2.giphy.com/media/v1.Y2lkPTc5MGI3NjExeWp1eWZ2MWc2NnZqdDZtcGVqcjBxa2JqY3prY2ZhZm9mam5rb2NtaSZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/l0MYGb0LuZ3n7dRnO/giphy.gif", // "time's up"
    "https://media3.giphy.com/media/v1.Y2lkPTc5MGI3NjExbHZrdnV3eGFtOGVrZDF6b3JmMzhtOWNsMHNxdGdmeHJqdXcyNTl1NSZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/FspLvJQlQACXu/giphy.gif", // dramatic finger pointing
    "https://media4.giphy.com/media/v1.Y2lkPTc5MGI3NjExc2hqYWh5cHFia3JlZDNrejJsaGZvbGtoaGF6emdvY2NpODA3NDdlZyZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/l2QDM9Jnim1YVILXa/giphy.gif", // "you had your chance"
];

/// GIFs pour le mode clément (plus gentils)
const MERCIFUL_GIFS: &[&str] = &[
    "https://media1.giphy.com/media/v1.Y2lkPTc5MGI3NjExZXBxNXp0a2cza3EwcmFpcmF5ZXV2a3ppZjBlbXJhcGoxcTNlbjNxOSZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/26u4lOMA8JKSnL9Uk/giphy.gif", // "you're lucky"
    "https://media2.giphy.com/media/v1.Y2lkPTc5MGI3NjExbmxsZDB3bWVvMjNjZGxrazBpM3I0Yjl4dHF6cWt1cnlla2toOTVjZCZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/3o7TKwmnDgQb5jemjK/giphy.gif", // winking
    "https://media3.giphy.com/media/v1.Y2lkPTc5MGI3NjExcHZteTNjeTNwMng5MGFld3AzNmpudnJ0cDFhNDV4YzllZnF4cnRrMCZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/l0MYu38R0PPhIXe36/giphy.gif", // "this time only"
];

/// Messages d'avertissement initial
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

/// Messages d'avertissement final (juste avant kick)
const FINAL_WARNING_MESSAGES: &[&str] = &[
    "🔥 **JE VOUS L'AVAIS DIT !** 🔥\n\nVous avez pas voulu écouter, maintenant vous allez subir ! **10 secondes** et vous dégagez tous !",
    "⚡ **TROP TARD POUR PLEURER !** ⚡\n\nJ'vous avais prévenus mais vous m'avez ignoré ! Dans **10 secondes** c'est l'éjection totale !",
    "💀 **VOUS L'AUREZ VOULU !** 💀\n\nJ'ai été sympa, j'ai averti, mais vous restez là comme des mules ! **10 secondes** avant le carnage !",
    "🌪️ **LA TEMPÊTE ARRIVE !** 🌪️\n\nVous pensiez que je bluffais ? Eh ben non ! **10 secondes** avant que je vous sorte de là !",
    "🚨 **DERNIER AVERTISSEMENT !** 🚨\n\nC'était votre dernière chance et vous l'avez gâchée ! **10 secondes** avant disconnection forcée !",
    "⚰️ **RIP VOTRE TRANQUILLITÉ !** ⚰️\n\nVous m'avez forcé à être méchant ! Dans **10 secondes** je vous kick sans pitié !",
    "🎬 **FIN DU FILM !** 🎬\n\nLe générique va défiler sur vos têtes ! **10 secondes** avant le kick final !",
    "💣 **BOMBE À RETARDEMENT !** 💣\n\nTic-tac, tic-tac... **10 secondes** avant l'explosion ! J'vous avais dit de partir !"
];

/// Messages pour le mode clément (pas de kick)
const MERCIFUL_MESSAGES: &[&str] = &[
    "😌 **BON, ÇA PASSE POUR CETTE FOIS...** 😌\n\nVous avez de la chance, je suis en mode gentil aujourd'hui ! Mais la prochaine fois, je serai moins clément !",
    "🎭 **SPECTACLE TERMINÉ !** 🎭\n\nJ'espère que vous avez apprécié la représentation ! Heureusement pour vous, c'était juste du théâtre... cette fois !",
    "🦸‍♂️ **SUPER-HÉROS EN CONGÉ !** 🦸‍♂️\n\nMon pouvoir de kick est en maintenance ! Vous pouvez dormir tranquilles... pour le moment !",
    "🎪 **CIRQUE FERMÉ !** 🎪\n\nLe numéro est terminé ! J'espère que ça vous a fait peur au moins ! Allez, filez au lit maintenant !",
    "👑 **GRÂCE ROYALE !** 👑\n\nSa Majesté LeKickerFou vous accorde son pardon ! Mais ne comptez pas sur ma clémence éternellement !",
    "🌟 **ÉTOILE FILANTE !** 🌟\n\nVotre vœu a été exaucé : pas de kick ce soir ! Mais les étoiles filantes sont rares... Méfiez-vous !",
    "🎁 **CADEAU DE LA MAISON !** 🎁\n\nConsidérez ça comme un bonus ! La prochaine fois, je ne serai peut-être pas d'aussi bonne humeur !",
    "🔮 **BOULE DE CRISTAL CLÉMENTE !** 🔮\n\nLes astres sont alignés en votre faveur ce soir ! Mais demain... qui sait ? 😏"
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

    /// Envoie l'avertissement initial
    pub async fn send_initial_warning(
        &self,
        ctx: &SerenityContext,
        members: &[Member],
        voice_channel_name: &str,
    ) -> bool {
        if members.is_empty() {
            return false;
        }

        let warning_message = self.generate_initial_warning_message(members);
        let gif_url = Self::select_random_gif(WARNING_GIFS);

        self.send_warning_embed(
            ctx,
            members,
            voice_channel_name,
            "🚨 ALERTE COUVRE-FEU 🚨",
            &warning_message,
            &gif_url,
            Colour::from_rgb(255, 165, 0), // Orange
        ).await
    }

    /// Envoie l'avertissement final (juste avant kick)
    pub async fn send_final_warning(
        &self,
        ctx: &SerenityContext,
        members: &[Member],
        voice_channel_name: &str,
    ) -> bool {
        if members.is_empty() {
            return false;
        }

        let warning_message = self.generate_final_warning_message(members);
        let gif_url = Self::select_random_gif(FINAL_WARNING_GIFS);

        self.send_warning_embed(
            ctx,
            members,
            voice_channel_name,
            "💀 SENTENCE FINALE 💀",
            &warning_message,
            &gif_url,
            Colour::from_rgb(220, 20, 60), // Rouge crimson
        ).await
    }

    /// Envoie un message de grâce (mode warning-only)
    pub async fn send_merciful_message(
        &self,
        ctx: &SerenityContext,
        members: &[Member],
        voice_channel_name: &str,
    ) -> bool {
        if members.is_empty() {
            return false;
        }

        let merciful_message = self.generate_merciful_message(members);
        let gif_url = Self::select_random_gif(MERCIFUL_GIFS);

        self.send_warning_embed(
            ctx,
            members,
            voice_channel_name,
            "🌟 GRÂCE ACCORDÉE 🌟",
            &merciful_message,
            &gif_url,
            Colour::from_rgb(50, 205, 50), // Vert lime
        ).await
    }

    /// Fonction générique pour envoyer des embeds d'avertissement
    async fn send_warning_embed(
        &self,
        ctx: &SerenityContext,
        members: &[Member],
        voice_channel_name: &str,
        title: &str,
        message: &str,
        gif_url: &str,
        color: Colour,
    ) -> bool {
        let Some(warning_channel_id) = self.config.warning_channel_id else {
            return false;
        };

        let embed = CreateEmbed::new()
            .title(title)
            .description(message)
            .color(color)
            .image(gif_url)
            .footer(serenity::builder::CreateEmbedFooter::new(format!(
                "Salon concerné: {voice_channel_name} | Bot LeKickerFou"
            )))
            .timestamp(serenity::model::Timestamp::now());

        let message_builder = CreateMessage::new().embed(embed).content({
            let mentions = members
                .iter()
                .map(|m| format!("<@{}>", m.user.id))
                .collect::<Vec<_>>()
                .join(" ");
            format!("👋 {mentions}")
        });

        match warning_channel_id.send_message(ctx, message_builder).await {
            Ok(_) => {
                log_info(&format!(
                    "🚨 Message envoyé à {} utilisateur(s) dans #{} via le salon d'avertissement",
                    members.len(),
                    voice_channel_name
                ));
                true
            }
            Err(e) => {
                log_error(&format!(
                    "Impossible d'envoyer le message dans le salon: {}",
                    e
                ));
                false
            }
        }
    }

    /// Génère un message d'avertissement initial
    fn generate_initial_warning_message(&self, members: &[Member]) -> String {
        let mut rng = rand::rng();
        let base_message = WARNING_MESSAGES
            .iter()
            .choose(&mut rng)
            .unwrap_or(&WARNING_MESSAGES[0]);

        let delay = self.config.warning_delay_seconds;
        let message = base_message.replace("{delay}", &delay.to_string());

        let user_list = self.format_user_list(members);
        let action_text = if self.config.is_warning_only_mode() {
            "💌 *Mode gentil activé - Aucune déconnexion ne sera effectuée, ceci n'est qu'un rappel amical !*"
        } else {
            "⚠️ *Cette menace est bien réelle ! Déconnexion automatique programmée !*"
        };

        format!("{message}\n\n{user_list}\n\n{action_text}")
    }

    /// Génère un message d'avertissement final
    fn generate_final_warning_message(&self, members: &[Member]) -> String {
        let mut rng = rand::rng();
        let base_message = FINAL_WARNING_MESSAGES
            .iter()
            .choose(&mut rng)
            .unwrap_or(&FINAL_WARNING_MESSAGES[0]);

        let user_list = self.format_user_list(members);
        let action_text = "🔥 *Vous avez eu votre chance... Maintenant c'est l'heure de payer !*";

        format!("{}\n\n{}\n\n{}", base_message, user_list, action_text)
    }

    /// Génère un message de grâce
    fn generate_merciful_message(&self, members: &[Member]) -> String {
        let mut rng = rand::rng();
        let base_message = MERCIFUL_MESSAGES
            .iter()
            .choose(&mut rng)
            .unwrap_or(&MERCIFUL_MESSAGES[0]);

        let user_list = self.format_user_list(members);
        let action_text = "😇 *Profitez bien de cette clémence... Elle ne durera peut-être pas !*";

        format!("{}\n\n{}\n\n{}", base_message, user_list, action_text)
    }

    /// Formate la liste des utilisateurs
    fn format_user_list(&self, members: &[Member]) -> String {
        if members.len() == 1 {
            format!("**Cible désignée :** {}", members[0].display_name())
        } else {
            let names = members
                .iter()
                .map(|m| format!("• {}", m.display_name()))
                .collect::<Vec<_>>()
                .join("\n");
            format!("**Cibles désignées ({}) :**\n{}", members.len(), names)
        }
    }

    /// Sélectionne un GIF aléatoire dans une collection
    fn select_random_gif(gifs: &[&str]) -> String {
        let mut rng = rand::rng();
        gifs.iter()
            .choose(&mut rng)
            .unwrap_or(&gifs[0])
            .to_string()
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
