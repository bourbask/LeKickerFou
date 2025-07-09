//! Gestion des niveaux de verbosité pour le logging.

use serde::{Deserialize, Serialize};

/// Niveaux de verbosité pour le logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VerbosityLevel {
    /// Niveau 0 - Déconnexions effectives uniquement
    Quiet = 0,
    /// Niveau 1 - + Changements critiques (config, erreurs)
    Normal = 1,
    /// Niveau 2 - + Tous les changements (avertissements, logs)
    Verbose = 2,
    /// Niveau 3 - + Toutes les interactions utilisateur
    Debug = 3,
}

impl VerbosityLevel {
    /// Crée un niveau de verbosité depuis le nombre de -v
    pub fn from_count(count: u8) -> Self {
        match count {
            0 => VerbosityLevel::Quiet,
            1 => VerbosityLevel::Normal,
            2 => VerbosityLevel::Verbose,
            _ => VerbosityLevel::Debug,
        }
    }

    /// Description textuelle du niveau
    pub fn description(&self) -> &'static str {
        match self {
            VerbosityLevel::Quiet => "Kicks uniquement",
            VerbosityLevel::Normal => "Changements critiques",
            VerbosityLevel::Verbose => "Tous les changements",
            VerbosityLevel::Debug => "Toutes les interactions",
        }
    }
}
