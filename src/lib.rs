//! # LeKickerFou
//!
//! Bot Discord pour déconnecter automatiquement les utilisateurs des salons vocaux
//! selon un planning configurable défini par des expressions cron.

pub mod bot;
pub mod config;
pub mod error;
pub mod utils;

pub use error::BotError;
