//! Utilitaires partagés pour le logging et la gestion des tokens.

use std::env;

use anyhow::Context;
use chrono::Utc;
use colored::*;

use crate::BotError;

/// Récupère le token Discord depuis les variables d'environnement
///
/// # Errors
///
/// Retourne une erreur si la variable d'environnement `DISCORD_TOKEN` n'est pas définie.
///
/// # Examples
///
/// ```
/// std::env::set_var("DISCORD_TOKEN", "your_token_here");
/// match lekickerfou::utils::get_discord_token() {
///     Ok(token) => println!("Token récupéré: {}", &token[..10]), // Affiche les 10 premiers caractères
///     Err(_) => println!("Pas de token"),
/// }
/// std::env::remove_var("DISCORD_TOKEN"); // Nettoyer après le test
/// ```
pub fn get_discord_token() -> anyhow::Result<String> {
    env::var("DISCORD_TOKEN")
        .map_err(|_| {
            println!("❓ Token Discord non trouvé dans DISCORD_TOKEN.");
            println!("💡 Vous pouvez :");
            println!("   1. Créer un fichier .env avec DISCORD_TOKEN=votre_token");
            println!("   2. Exporter la variable: export DISCORD_TOKEN=votre_token");
            println!("   3. Lancer avec: DISCORD_TOKEN=votre_token ./lekickerfou");

            BotError::MissingConfig("Token Discord requis".to_string())
        })
        .context("Token Discord manquant")
}

/// Affiche un message d'information formaté avec horodatage
///
/// # Arguments
///
/// * `msg` - Le message à afficher
///
/// # Examples
///
/// ```
/// // Les fonction de logging sont conçues pour l'output console
/// // et ne retournent rien de testable directement
/// lekickerfou::utils::log_info("Bot démarré avec succès");
/// // Cela affichera: ℹ️ [2024-01-01 12:00:00 UTC] Bot démarré avec succès
/// ```
pub fn log_info(msg: &str) {
    println!(
        "{} {} {}",
        "ℹ️".green(),
        Utc::now()
            .format("[%Y-%m-%d %H:%M:%S UTC]")
            .to_string()
            .dimmed(),
        msg
    );
}

/// Affiche un message d'erreur formaté avec horodatage
///
/// # Arguments
///
/// * `msg` - Le message d'erreur à afficher
///
/// # Examples
///
/// ```
/// // Les fonction de logging sont conçues pour l'output console  
/// // et ne retournent rien de testable directement
/// lekickerfou::utils::log_error("Impossible de se connecter au serveur");
/// // Cela affichera: ❌ [2024-01-01 12:00:00 UTC] Impossible de se connecter au serveur (en rouge)
/// ```
pub fn log_error(msg: &str) {
    eprintln!(
        "{} {} {}",
        "❌".red(),
        Utc::now()
            .format("[%Y-%m-%d %H:%M:%S UTC]")
            .to_string()
            .dimmed(),
        msg.red()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_discord_token_when_set() {
        env::set_var("DISCORD_TOKEN", "test_token_123");
        let result = get_discord_token();
        env::remove_var("DISCORD_TOKEN");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_token_123");
    }

    #[test]
    fn test_get_discord_token_when_missing() {
        env::remove_var("DISCORD_TOKEN");
        let result = get_discord_token();

        assert!(result.is_err());
    }

    #[test]
    fn test_log_functions_dont_panic() {
        // Test que les fonctions de log ne paniquent pas
        log_info("Test message");
        log_error("Test error message");
        // Ces tests vérifient juste qu'aucune panique ne se produit
    }
}
