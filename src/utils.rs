//! Fonctions utilitaires communes.

use std::time::SystemTime;

/// Affiche un message d'information dans la console avec timestamp
pub fn log_info(message: &str) {
    println!("[{}] ℹ️ {}", format_timestamp(), message);
}

/// Affiche un message d'erreur dans la console avec timestamp
pub fn log_error(message: &str) {
    eprintln!("[{}] ❌ {}", format_timestamp(), message);
}

/// Affiche un message d'avertissement dans la console avec timestamp
pub fn log_warning(message: &str) {
    println!("[{}] ⚠️ {}", format_timestamp(), message);
}

/// Formate un timestamp simple
fn format_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let datetime =
        chrono::DateTime::from_timestamp(now as i64, 0).unwrap_or_else(|| chrono::Utc::now());

    datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Valide une expression cron basique
pub fn validate_cron_expression(expression: &str) -> Result<(), String> {
    let parts: Vec<&str> = expression.split_whitespace().collect();

    if parts.len() != 6 {
        return Err("L'expression cron doit contenir exactement 6 parties".to_string());
    }

    Ok(())
}
