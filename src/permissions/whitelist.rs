//! Gestionnaire de la whitelist des utilisateurs et rôles autorisés.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serenity::model::id::{RoleId, UserId};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Niveaux de permission disponibles
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// Consultation uniquement
    User = 0,
    /// Modification de configuration sauf permissions
    Moderator = 1,
    /// Accès complet incluant gestion des permissions
    Admin = 2,
}

impl std::fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionLevel::User => write!(f, "👤 User"),
            PermissionLevel::Moderator => write!(f, "🛡️ Moderator"),
            PermissionLevel::Admin => write!(f, "👑 Admin"),
        }
    }
}

/// Métadonnées sur les permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionMetadata {
    pub last_modified: DateTime<Utc>,
    pub total_users: usize,
    pub total_roles: usize,
    pub modified_by: Option<String>,
}

/// Permissions d'un groupe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupPermissions {
    /// Utilisateurs spécifiques avec leurs IDs
    pub users: HashMap<UserId, PermissionLevel>,
    /// Rôles avec leurs niveaux de permission
    pub roles: HashMap<RoleId, PermissionLevel>,
}

impl GroupPermissions {
    /// Crée un nouveau groupe de permissions vide
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            roles: HashMap::new(),
        }
    }

    /// Ajoute un utilisateur à la whitelist
    pub fn add_user(&mut self, user_id: UserId, level: PermissionLevel) {
        self.users.insert(user_id, level);
    }

    /// Ajoute un rôle à la whitelist
    pub fn add_role(&mut self, role_id: RoleId, level: PermissionLevel) {
        self.roles.insert(role_id, level);
    }

    /// Supprime un utilisateur de la whitelist
    pub fn remove_user(&mut self, user_id: &UserId) -> bool {
        self.users.remove(user_id).is_some()
    }

    /// Supprime un rôle de la whitelist
    pub fn remove_role(&mut self, role_id: &RoleId) -> bool {
        self.roles.remove(role_id).is_some()
    }

    /// Récupère le niveau d'un utilisateur
    pub fn get_user_level(&self, user_id: &UserId) -> Option<PermissionLevel> {
        self.users.get(user_id).copied()
    }

    /// Récupère le niveau d'un rôle
    pub fn get_role_level(&self, role_id: &RoleId) -> Option<PermissionLevel> {
        self.roles.get(role_id).copied()
    }
}

impl Default for GroupPermissions {
    fn default() -> Self {
        Self::new()
    }
}

/// Structure complète des permissions utilisateurs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    /// Version du format de permissions
    pub version: String,
    /// Permissions par niveau
    pub permissions: GroupPermissions,
    /// Métadonnées
    pub metadata: PermissionMetadata,
}

impl UserPermissions {
    /// Crée une nouvelle structure de permissions vide
    pub fn new() -> Self {
        Self {
            version: "1.1.0".to_string(),
            permissions: GroupPermissions::new(),
            metadata: PermissionMetadata {
                last_modified: Utc::now(),
                total_users: 0,
                total_roles: 0,
                modified_by: None,
            },
        }
    }

    /// Met à jour les métadonnées
    pub fn update_metadata(&mut self, modified_by: Option<String>) {
        self.metadata.last_modified = Utc::now();
        self.metadata.total_users = self.permissions.users.len();
        self.metadata.total_roles = self.permissions.roles.len();
        self.metadata.modified_by = modified_by;
    }
}

impl Default for UserPermissions {
    fn default() -> Self {
        Self::new()
    }
}

/// Gestionnaire de la whitelist des utilisateurs autorisés
pub struct WhitelistManager {
    file_path: String,
}

impl WhitelistManager {
    /// Crée une nouvelle instance du gestionnaire de whitelist
    pub fn new() -> Self {
        Self {
            file_path: "whitelist.json".to_string(),
        }
    }

    /// Charge la whitelist depuis le fichier ou crée une whitelist vide
    pub fn load_or_create(&self) -> Result<UserPermissions> {
        if Path::new(&self.file_path).exists() {
            self.load()
        } else {
            let permissions = UserPermissions::new();
            self.save(&permissions)?;
            println!("✅ Fichier whitelist créé: {}", self.file_path);
            Ok(permissions)
        }
    }

    /// Charge la whitelist depuis le fichier
    pub fn load(&self) -> Result<UserPermissions> {
        let content = fs::read_to_string(&self.file_path)?;
        let permissions: UserPermissions = serde_json::from_str(&content)?;
        Ok(permissions)
    }

    /// Sauvegarde la whitelist dans le fichier
    pub fn save(&self, permissions: &UserPermissions) -> Result<()> {
        let content = serde_json::to_string_pretty(permissions)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    /// Ajoute un utilisateur à la whitelist
    pub fn add_user(
        &self,
        user_id: UserId,
        level: PermissionLevel,
        modified_by: Option<String>,
    ) -> Result<()> {
        let mut permissions = self.load_or_create()?;
        permissions.permissions.add_user(user_id, level);
        permissions.update_metadata(modified_by);
        self.save(&permissions)?;
        println!("✅ Utilisateur {} ajouté avec niveau {}", user_id, level);
        Ok(())
    }

    /// Ajoute un rôle à la whitelist
    pub fn add_role(
        &self,
        role_id: RoleId,
        level: PermissionLevel,
        modified_by: Option<String>,
    ) -> Result<()> {
        let mut permissions = self.load_or_create()?;
        permissions.permissions.add_role(role_id, level);
        permissions.update_metadata(modified_by);
        self.save(&permissions)?;
        println!("✅ Rôle {} ajouté avec niveau {}", role_id, level);
        Ok(())
    }

    /// Supprime un utilisateur de la whitelist
    pub fn remove_user(&self, user_id: UserId, modified_by: Option<String>) -> Result<bool> {
        let mut permissions = self.load_or_create()?;
        let removed = permissions.permissions.remove_user(&user_id);
        if removed {
            permissions.update_metadata(modified_by);
            self.save(&permissions)?;
            println!("✅ Utilisateur {} supprimé de la whitelist", user_id);
        }
        Ok(removed)
    }

    /// Supprime un rôle de la whitelist
    pub fn remove_role(&self, role_id: RoleId, modified_by: Option<String>) -> Result<bool> {
        let mut permissions = self.load_or_create()?;
        let removed = permissions.permissions.remove_role(&role_id);
        if removed {
            permissions.update_metadata(modified_by);
            self.save(&permissions)?;
            println!("✅ Rôle {} supprimé de la whitelist", role_id);
        }
        Ok(removed)
    }
}

impl Default for WhitelistManager {
    fn default() -> Self {
        Self::new()
    }
}
