//! Module principal du bot Discord.

pub mod handler;
pub mod voice_manager;

pub use handler::BotHandler;
pub use voice_manager::VoiceChannelManager;
