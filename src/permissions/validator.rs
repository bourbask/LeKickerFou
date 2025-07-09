//! Validateur de permissions pour les interactions Discord.

use anyhow::Result;
use serenity::{all::Interaction, client::Context, model::guild::Member};

use super::{PermissionLevel, PermissionResult, WhitelistManager};

/// Validateur de permissions pour les commandes Discord
pub struct PermissionValidator {
    whitelist_manager: WhitelistManager,
}

impl PermissionValidator {
    /// Crée une nouvelle instance du validateur
    pub fn new() -> Self {
        Self {
            whitelist_manager: WhitelistManager::new(),
        }
    }

    /// Valide les permissions d'un utilisateur via une interaction Discord
    pub async fn validate_interaction_permission(
        &self,
        ctx: &Context,
        interaction: &Interaction,
        required_level: PermissionLevel,
    ) -> PermissionResult {
        let (user_id, guild_id) = match interaction {
            Interaction::Command(cmd) => (cmd.user.id, cmd.guild_id),
            _ => return PermissionResult::Error("Type d'interaction non supporté".to_string()),
        };

        // Charger la whitelist
        let permissions = match self.whitelist_manager.load_or_create() {
            Ok(perms) => perms,
            Err(e) => {
                return PermissionResult::Error(format!("Erreur chargement whitelist: {}", e))
            }
        };

        // Vérifier d'abord les permissions utilisateur directes
        if let Some(user_level) = permissions.permissions.get_user_level(&user_id) {
            if user_level >= required_level {
                return PermissionResult::Authorized(user_level);
            }
        }

        // Si pas de permission directe, vérifier les rôles
        if let Some(guild_id) = guild_id {
            match self.get_member_roles(ctx, guild_id, user_id).await {
                Ok(member_roles) => {
                    let mut highest_role_level = None;

                    for role_id in member_roles {
                        if let Some(role_level) = permissions.permissions.get_role_level(&role_id) {
                            match highest_role_level {
                                None => highest_role_level = Some(role_level),
                                Some(current) if role_level > current => {
                                    highest_role_level = Some(role_level)
                                }
                                _ => {}
                            }
                        }
                    }

                    if let Some(level) = highest_role_level {
                        if level >= required_level {
                            return PermissionResult::Authorized(level);
                        }
                    }
                }
                Err(e) => {
                    return PermissionResult::Error(format!("Erreur récupération rôles: {}", e))
                }
            }
        }

        // Si aucune permission trouvée
        PermissionResult::Unauthorized
    }

    /// Récupère les rôles d'un membre
    async fn get_member_roles(
        &self,
        ctx: &Context,
        guild_id: serenity::model::id::GuildId,
        user_id: serenity::model::id::UserId,
    ) -> Result<Vec<serenity::model::id::RoleId>> {
        // D'abord essayer le cache
        if let Some(guild) = ctx.cache.guild(guild_id) {
            if let Some(member) = guild.members.get(&user_id) {
                return Ok(member.roles.clone());
            }
        }

        // Si pas dans le cache, récupérer depuis l'API
        match guild_id.member(&ctx.http, user_id).await {
            Ok(member) => Ok(member.roles),
            Err(e) => Err(anyhow::anyhow!("Impossible de récupérer le membre: {}", e)),
        }
    }

    /// Génère un message d'erreur de permission personnalisé
    pub fn permission_denied_message(
        &self,
        required_level: PermissionLevel,
        user_level: Option<PermissionLevel>,
    ) -> String {
        match user_level {
            Some(level) => format!(
                "❌ **Accès refusé**\n\
                Votre niveau: {}\n\
                Niveau requis: {}\n\n\
                💡 Contactez un administrateur pour obtenir les permissions nécessaires.",
                level, required_level
            ),
            None => format!(
                "❌ **Accès refusé**\n\
                Vous n'êtes pas autorisé à utiliser cette commande.\n\
                Niveau requis: {}\n\n\
                💡 Contactez un administrateur pour être ajouté à la whitelist.",
                required_level
            ),
        }
    }

    /// Accès au gestionnaire de whitelist pour les commandes de gestion
    pub fn whitelist_manager(&self) -> &WhitelistManager {
        &self.whitelist_manager
    }
}

impl Default for PermissionValidator {
    fn default() -> Self {
        Self::new()
    }
}
