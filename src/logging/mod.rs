//! Module de logging pour Discord et console.

pub mod discord_logger;
pub mod levels;

pub use discord_logger::{get_global_logger, init_global_logger, DiscordLogger};
pub use levels::VerbosityLevel;
