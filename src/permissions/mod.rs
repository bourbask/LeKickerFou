//! Module de gestion des permissions et authentification des utilisateurs.

pub mod validator;
pub mod whitelist;

pub use validator::PermissionValidator;
pub use whitelist::{PermissionLevel, WhitelistManager};

/// Résultat d'une vérification de permission
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionResult {
    /// Utilisateur autorisé avec le niveau spécifié
    Authorized(PermissionLevel),
    /// Utilisateur non autorisé
    Unauthorized,
    /// Erreur lors de la vérification
    Error(String),
}

impl PermissionResult {
    /// Vérifie si l'utilisateur est autorisé
    pub fn is_authorized(&self) -> bool {
        matches!(self, PermissionResult::Authorized(_))
    }

    /// Récupère le niveau de permission si autorisé
    pub fn permission_level(&self) -> Option<PermissionLevel> {
        match self {
            PermissionResult::Authorized(level) => Some(*level),
            _ => None,
        }
    }
}
