//! Types d'erreurs personnalisés pour une gestion fine des cas d'échec.

use thiserror::Error;

/// Erreurs spécifiques au fonctionnement du bot Discord
#[derive(Error, Debug)]
pub enum BotError {
    /// Configuration manquante ou incomplète
    #[error("Configuration manquante: {0}")]
    MissingConfig(String),

    /// Le salon spécifié n'est pas un salon vocal
    #[error("Le salon n'est pas un salon vocal")]
    InvalidChannelType,

    /// Le salon spécifié n'appartient pas à un serveur Discord
    #[error("Le salon n'est pas un salon de serveur")]
    NotGuildChannel,

    /// Erreur provenant de l'API Discord via Serenity
    #[error("Erreur Discord API: {0}")]
    DiscordApi(#[from] serenity::Error),

    /// Configuration invalide ou corrompue
    #[error("Configuration invalide: {0}")]
    InvalidConfig(String),
}
