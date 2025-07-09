# 🤖 LeKickerFou

Bot Discord pour déconnecter automatiquement les utilisateurs des salons vocaux selon un planning configurable.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Discord](https://img.shields.io/badge/discord-bot-blue.svg)](https://discord.com/developers/applications)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🎯 Fonctionnalités

- **Déconnexion automatique** selon un planning cron configurable
- **Avertissements optionnels** avant déconnexion avec messages amusants
- **Mode "avertissement seul"** pour rappels sans déconnexion
- **Système de logs Discord** avec niveaux de verbosité
- **Gestion des permissions** via whitelist (User/Moderator/Admin)
- **Historique et sauvegarde** automatique des configurations
- **Import/export** de configurations
- **Commandes slash Discord** pour la gestion en temps réel
- **Interface CLI complète** pour la configuration

## 📋 Prérequis

- **Rust** 1.70 ou supérieur
- **Discord Bot Token** (voir [Configuration Discord](#-configuration-discord))
- **Permissions Discord** appropriées pour le bot

### Permissions Discord requises

- ✅ **Déplacer des membres** (pour déconnecter)
- ✅ **Envoyer des messages** (pour les logs et avertissements)
- ✅ **Utiliser des commandes slash**
- ✅ **Voir les salons vocaux**
- ✅ **Lire l'historique des messages**

## 🚀 Installation

### 1. Compilation depuis les sources

```bash
# Cloner le repository
git clone https://github.com/votre-username/lekickerfou.git
cd lekickerfou

# Compiler
cargo build --release

# L'exécutable sera dans target/release/
```

### 2. Installation rapide avec Cargo

```bash
# Installer directement depuis Git
cargo install --git https://github.com/votre-username/lekickerfou.git

# Ou depuis crates.io (quand disponible)
cargo install lekickerfou
```

## 🔧 Configuration Discord

### 1. Créer un bot Discord

1. Allez sur [Discord Developer Portal](https://discord.com/developers/applications)
2. Créez une "New Application"
3. Dans l'onglet "Bot", créez un bot
4. Copiez le **Token** (gardez-le secret !)
5. Activez les **Privileged Gateway Intents** si nécessaire

### 2. Inviter le bot sur votre serveur

Utilisez cette URL (remplacez `YOUR_CLIENT_ID` par l'ID de votre application) :

```
https://discord.com/api/oauth2/authorize?client_id=YOUR_CLIENT_ID&permissions=16785408&scope=bot%20applications.commands
```

### 3. Configurer le token

```bash
# Variable d'environnement (recommandé)
export DISCORD_TOKEN="votre_token_ici"

# Ou dans un fichier .env
echo "DISCORD_TOKEN=votre_token_ici" > .env
```

## ⚡ Démarrage rapide

### Configuration initiale

```bash
# Configuration basique (salon vocal obligatoire)
./lekickerfou --channel 123456789012345678

# Configuration complète avec logs et avertissements
./lekickerfou \
  --channel 123456789012345678 \
  --log-channel 987654321098765432 \
  --warning-channel 555666777888999000 \
  --schedule "0 0 22 * * *"  # Tous les jours à 22h
```

### Démarrage du bot

```bash
# Une fois configuré, démarrer simplement
./lekickerfou

# Avec logs verbeux
./lekickerfou -vv
```

## 🎮 Utilisation

### Commandes ligne de commande

```bash
# Configuration
./lekickerfou --channel SALON_VOCAL_ID                    # Salon à surveiller
./lekickerfou --log-channel SALON_LOG_ID                  # Salon pour les logs (optionnel)
./lekickerfou --warning-channel SALON_AVERTISSEMENT_ID    # Salon pour avertissements (optionnel)
./lekickerfou --warning-delay 120                         # Délai avant déconnexion (secondes)
./lekickerfou --warning-only                              # Mode avertissement seul
./lekickerfou --schedule "*/30 * * * * *"                # Expression cron

# Verbosité
./lekickerfou -v          # Changements critiques
./lekickerfou -vv         # Tous les changements
./lekickerfou -vvv        # Toutes les interactions

# Gestion des configurations
./lekickerfou --export production-config.json             # Exporter
./lekickerfou --import production-config.json             # Importer
./lekickerfou --list-backups                              # Lister les sauvegardes
./lekickerfou --restore "2024-01-15_14-30-25.json"      # Restaurer une sauvegarde
```

### Commandes slash Discord

Une fois le bot démarré, utilisez ces commandes dans Discord :

- `/status` - Affiche le statut et la configuration actuelle
- `/kick` - Déconnecte manuellement tous les utilisateurs (Moderator+)
- `/config` - Affiche/modifie la configuration (Moderator+)
- `/permissions` - Gère les permissions utilisateur (Admin)
- `/history` - Affiche l'historique des configurations

## 🔐 Système de permissions

LeKickerFou utilise une whitelist avec 3 niveaux de permissions :

### Niveaux disponibles

| Niveau        | Emoji | Description                      | Accès                               |
| ------------- | ----- | -------------------------------- | ----------------------------------- |
| **User**      | 👤    | Consultation uniquement          | `/status`, `/history`               |
| **Moderator** | 🛡️    | Configuration (sauf permissions) | + `/kick`, `/config`                |
| **Admin**     | 👑    | Accès complet                    | + `/permissions`, gestion whitelist |

### Configuration de la whitelist

Éditez le fichier `whitelist.json` :

```json
{
  "version": "1.1.0",
  "permissions": {
    "users": {
      "123456789012345678": "Admin",
      "987654321098765432": "Moderator"
    },
    "roles": {
      "555666777888999000": "Moderator",
      "111222333444555666": "User"
    }
  },
  "metadata": {
    "last_modified": "2024-01-15T14:30:25Z",
    "total_users": 2,
    "total_roles": 2,
    "modified_by": "AdminUser#1234"
  }
}
```

## 📅 Planning cron

Le bot utilise des expressions cron à 6 parties : `seconde minute heure jour mois jour_semaine`

### Exemples courants

```bash
# Toutes les minutes
"0 * * * * *"

# Toutes les 30 secondes
"*/30 * * * * *"

# Tous les jours à 22h
"0 0 22 * * *"

# Du lundi au vendredi à 18h
"0 0 18 * * 1-5"

# Toutes les heures en semaine (9h-17h)
"0 0 9-17 * * 1-5"

# Premier jour de chaque mois à minuit
"0 0 0 1 * *"
```

### Générateurs cron en ligne

- [Crontab.guru](https://crontab.guru/) (convertissez vers 6 parties)
- [Cron Expression Generator](https://www.freeformatter.com/cron-expression-generator-quartz.html)

## 📁 Structure des fichiers

```
./
├── lekickerfou                 # Exécutable principal
├── bot_config.json            # Configuration principale
├── whitelist.json             # Permissions utilisateurs
├── configs/
│   └── backups/               # Sauvegardes automatiques
│       ├── 2024-01-15_14-30-25.json
│       └── 2024-01-15_15-45-10.json
└── .env                       # Variables d'environnement (optionnel)
```

## 🔄 Sauvegarde et restauration

### Sauvegarde automatique

Le bot crée automatiquement des sauvegardes :

- ✅ À chaque modification de configuration
- ✅ Maximum 10 sauvegardes conservées
- ✅ Rotation automatique (plus anciennes supprimées)

### Gestion manuelle

```bash
# Lister les sauvegardes
./lekickerfou --list-backups

# Exporter la configuration actuelle
./lekickerfou --export backup-$(date +%Y%m%d).json

# Restaurer une sauvegarde
./lekickerfou --restore "2024-01-15_14-30-25.json"

# Importer une configuration externe
./lekickerfou --import external-config.json
```

## 🚨 Dépannage

### Le bot ne démarre pas

```bash
# Vérifier le token
echo $DISCORD_TOKEN

# Vérifier la configuration
cat bot_config.json

# Lancer avec logs détaillés
./lekickerfou -vvv
```

### Le bot ne déconnecte pas

1. **Vérifiez les permissions Discord** du bot
2. **Vérifiez l'ID du salon vocal** dans la configuration
3. **Testez une déconnexion manuelle** : `/kick`
4. **Vérifiez les logs** avec `-vv`

### Expression cron invalide

```bash
# Tester la validation
./lekickerfou --schedule "VOTRE_EXPRESSION" --channel 123 --config-file /tmp/test.json

# Expressions valides
./lekickerfou --schedule "0 * * * * *"     # ✅ Toutes les minutes
./lekickerfou --schedule "0 0 22 * * *"    # ✅ Chaque jour à 22h
./lekickerfou --schedule "*/30 * * * *"    # ❌ Seulement 5 parties
```

### Permissions refusées

1. **Vérifiez la whitelist** : `cat whitelist.json`
2. **Ajoutez votre utilisateur** :
   ```json
   {
     "users": {
       "VOTRE_USER_ID": "Admin"
     }
   }
   ```
3. **Redémarrez le bot** après modification

## 🎛️ Configuration avancée

### Variables d'environnement

```bash
# Token Discord (obligatoire)
export DISCORD_TOKEN="votre_token"

# Fichier de configuration personnalisé
export BOT_CONFIG_FILE="custom-config.json"

# Niveau de log par défaut
export BOT_LOG_LEVEL="info"
```

### Configuration multi-serveurs

```bash
# Serveur 1
./lekickerfou --config-file server1-config.json --channel 111

# Serveur 2
./lekickerfou --config-file server2-config.json --channel 222
```

### Déploiement avec systemd

Créez `/etc/systemd/system/lekickerfou.service` :

```ini
[Unit]
Description=LeKickerFou Discord Bot
After=network.target

[Service]
Type=simple
User=discord-bot
WorkingDirectory=/opt/lekickerfou
ExecStart=/opt/lekickerfou/lekickerfou
Environment=DISCORD_TOKEN=votre_token_ici
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Puis :

```bash
sudo systemctl daemon-reload
sudo systemctl enable lekickerfou
sudo systemctl start lekickerfou
sudo systemctl status lekickerfou
```

## 📊 Monitoring et logs

### Niveaux de verbosité

```bash
# Niveau 0 (défaut) - Kicks effectifs uniquement
./lekickerfou

# Niveau 1 (-v) - + Changements critiques
./lekickerfou -v

# Niveau 2 (-vv) - + Tous les changements
./lekickerfou -vv

# Niveau 3 (-vvv) - + Toutes les interactions
./lekickerfou -vvv
```

### Logs Discord

Si un salon de log est configuré, le bot y enverra :

- ✅ Déconnexions effectuées
- ✅ Changements de configuration
- ✅ Erreurs importantes
- ✅ Statistiques périodiques

## 🤝 Contribution

### Développement

```bash
# Cloner et développer
git clone https://github.com/votre-username/lekickerfou.git
cd lekickerfou

# Tests
cargo test

# Linter
cargo clippy

# Format
cargo fmt

# Documentation
cargo doc --open
```

### Signaler un bug

1. Utilisez les [Issues GitHub](https://github.com/votre-username/lekickerfou/issues)
2. Incluez :
   - Version du bot (`./lekickerfou --version`)
   - Configuration (masquez les tokens !)
   - Logs avec `-vvv`
   - Description du comportement attendu/observé

### Proposer une fonctionnalité

1. Créez une [Issue GitHub](https://github.com/votre-username/lekickerfou/issues) avec le label `feature`
2. Décrivez le cas d'usage
3. Proposez une implémentation si possible

## 📄 License

Ce projet est sous licence MIT. Voir le fichier [LICENSE](LICENSE) pour plus de détails.

## 🙏 Remerciements

- [Serenity](https://github.com/serenity-rs/serenity) - Bibliothèque Discord pour Rust
- [Tokio Cron Scheduler](https://github.com/mvniekerk/tokio-cron-scheduler) - Scheduler cron async
- [Clap](https://github.com/clap-rs/clap) - Parser d'arguments CLI
- La communauté Rust Discord

## 📞 Support

- **Documentation** : Ce README et `./lekickerfou --help`
- **Issues GitHub** : [Créer une issue](https://github.com/votre-username/lekickerfou/issues/new)
- **Discord** : [Serveur de support](https://discord.gg/votre-invite) (si applicable)

---

<div align="center">

**[⬆️ Retour en haut](#-lekickerfou)**

Made with ❤️ and 🦀 Rust

</div>
