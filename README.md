# 🤖 LeKickerFou

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?style=for-the-badge&logo=discord&logoColor=white)](https://discord.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/github/actions/workflow/status/bourbask/LeKickerFou/ci.yml?style=for-the-badge)](https://github.com/bourbask/LeKickerFou/actions)
[![Release](https://img.shields.io/github/v/release/bourbask/LeKickerFou?style=for-the-badge)](https://github.com/bourbask/LeKickerFou/releases)

> 🔇 Bot Discord intelligent pour automatiser la déconnexion d'utilisateurs des salons vocaux selon un planning configurable.

## 🚀 Migration vers Rust

**Ce projet a migré de JavaScript vers Rust pour de meilleures performances et une meilleure fiabilité.**

- 📁 **Version actuelle (Rust)** : Branche `main`
- 📁 **Version legacy (JavaScript)** : Branche [`legacy/javascript`](https://github.com/bourbask/LeKickerFou/tree/legacy/javascript)

## 📋 Table des matières

- [✨ Fonctionnalités](#-fonctionnalités)
- [🚀 Installation](#-installation)
- [⚙️ Configuration](#️-configuration)
- [📖 Usage](#-usage)
- [🔧 Développement](#-développement)
- [📊 Monitoring](#-monitoring)
- [🤝 Contribution](#-contribution)
- [📄 License](#-license)

## ✨ Fonctionnalités

### 🎯 Fonctionnalités principales

- **Déconnexion automatique** des utilisateurs des salons vocaux
- **Planning personnalisable** avec expressions cron
- **Configuration flexible** via CLI, fichiers JSON ou variables d'environnement
- **Logging Discord** optionnel avec notifications dans un salon dédié
- **Import/Export** de configurations pour faciliter le déploiement
- **Gestion d'erreurs robuste** sans crash possible
- **Interface CLI intuitive** avec aide complète

### 🛡️ Sécurité & Fiabilité

- ✅ Gestion exhaustive des erreurs
- ✅ Validation des configurations
- ✅ Logs détaillés pour le debugging
- ✅ Aucun crash possible lors d'erreurs Discord API
- ✅ Rollback automatique en cas de configuration invalide

### 🔄 Cas d'usage typiques

- **Fermeture automatique** de salons de réunion après les heures de bureau
- **Nettoyage périodique** de salons d'attente
- **Gestion de salons temporaires** pour événements
- **Application de règles horaires** sur des serveurs communautaires

## 🚀 Installation

### 📦 Téléchargement des binaires

Téléchargez la dernière version depuis les [releases GitHub](https://github.com/bourbask/LeKickerFou/releases) :

```bash
# Linux/macOS
curl -L https://github.com/bourbask/LeKickerFou/releases/latest/download/LeKickerFou-linux -o LeKickerFou
chmod +x LeKickerFou

# Windows
# Téléchargez LeKickerFou-windows.exe depuis les releases
```

### 🔨 Compilation depuis les sources

```bash
# Cloner le repository
git clone https://github.com/bourbask/LeKickerFou.git
cd LeKickerFou

# Compiler en mode release
cargo build --release

# Le binaire sera disponible dans target/release/
```

### 🐳 Docker (optionnel)

```bash
# Construire l'image
docker build -t LeKickerFou .

# Lancer avec variables d'environnement
docker run -e DISCORD_TOKEN=votre_token LeKickerFou --channel 123456789
```

## ⚙️ Configuration

### 🔑 Token Discord (Obligatoire)

Le bot nécessite un token Discord. Plusieurs méthodes de configuration :

#### Méthode 1 : Fichier .env (Recommandé)

```bash
# Créer un fichier .env
echo "DISCORD_TOKEN=votre_token_discord" > .env
```

#### Méthode 2 : Variable d'environnement

```bash
export DISCORD_TOKEN=votre_token_discord
```

#### Méthode 3 : Inline

```bash
DISCORD_TOKEN=votre_token_discord ./LeKickerFou --channel 123456789
```

### 🤖 Création du bot Discord

1. Allez sur [Discord Developer Portal](https://discord.com/developers/applications)
2. Créez une nouvelle application
3. Dans "Bot", créez un bot et copiez le token
4. Dans "OAuth2" > "URL Generator" :
   - Scope : `bot`
   - Permissions : `Move Members` + `View Channels`
5. Invitez le bot sur votre serveur avec l'URL générée

### 📋 Permissions requises

Le bot nécessite les permissions Discord suivantes :

- `View Channels` - Pour voir les salons
- `Move Members` - Pour déconnecter les utilisateurs
- `Send Messages` - Pour les logs (optionnel)

## 📖 Usage

### 🎯 Commandes principales

#### Configuration initiale

```bash
# Configuration basique
./LeKickerFou --channel 123456789

# Configuration complète
./LeKickerFou --channel 123456789 --log-channel 987654321 --schedule "0 0 22 * * *"
```

#### Gestion des configurations

```bash
# Export pour sauvegarde/partage
./LeKickerFou --export production.json

# Import d'une configuration
./LeKickerFou --import production.json

# Utilisation d'un fichier config personnalisé
./LeKickerFou --config-file my-config.json --channel 123456789
```

#### Lancement normal

```bash
# Avec configuration existante
./LeKickerFou

# Aide complète
./LeKickerFou --help
```

### ⏰ Expressions Cron

Le bot utilise la syntaxe cron standard avec support des secondes :

```
Format : "seconde minute heure jour mois jour_semaine"

Exemples :
* * * * * *          -> Chaque seconde
0 * * * * *          -> Chaque minute (défaut)
*/30 * * * * *       -> Toutes les 30 secondes
0 0 * * * *          -> Chaque heure
0 0 22 * * *         -> Tous les jours à 22h
0 0 18 * * 1-5       -> 18h en semaine uniquement
0 */15 9-17 * * 1-5  -> Toutes les 15 min, 9h-17h, en semaine
```

### 📁 Structure des fichiers

```
├── LeKickerFou           # Binaire principal
├── .env                     # Token Discord (optionnel)
├── bot_config.json          # Configuration automatique
├── production.json          # Configuration exportée
└── logs/                    # Logs (si configurés)
```

### 🔍 Exemple de configuration JSON

```json
{
  "voice_channel_id": "123456789012345678",
  "log_channel_id": "987654321098765432",
  "cron_schedule": "0 0 22 * * *"
}
```

## 📊 Monitoring

### 📋 Logs console

Le bot affiche des logs colorés en temps réel :

```
ℹ️  [2024-01-15 14:30:00 UTC] Bot connecté sous MonBot (ID: 123456789)
ℹ️  [2024-01-15 14:30:00 UTC] Configuration: Canal vocal 123456789, Canal de log 987654321
ℹ️  [2024-01-15 14:30:01 UTC] Surveillance des salons vocaux démarrée (planning: 0 * * * * *)
ℹ️  [2024-01-15 14:31:00 UTC] 3 membre(s) détecté(s) dans le salon 'Réunion'
ℹ️  [2024-01-15 14:31:01 UTC] ✅ User#1234 déconnecté avec succès
ℹ️  [2024-01-15 14:31:01 UTC] ✅ User#5678 déconnecté avec succès
ℹ️  [2024-01-15 14:31:02 UTC] ✅ User#9012 déconnecté avec succès
ℹ️  [2024-01-15 14:31:02 UTC] 3 utilisateur(s) déconnecté(s)
```

### 🔔 Logs Discord

Si configuré avec `--log-channel`, le bot envoie des notifications :

```
🔇 User#1234 déconnecté du salon 'Réunion'
🔇 User#5678 déconnecté du salon 'Réunion'
🔇 User#9012 déconnecté du salon 'Réunion'
```

### ❌ Gestion d'erreurs

Toutes les erreurs sont capturées et loggées sans interrompre le bot :

```
❌ [2024-01-15 14:31:00 UTC] Impossible de déconnecter User#1234: Missing Permissions
❌ [2024-01-15 14:31:01 UTC] Impossible d'envoyer le log Discord: Unknown Channel
```

## 🔧 Développement

### 🛠️ Environnement de développement

```bash
# Prérequis : Rust 1.70+
rustup update

# Cloner et setup
git clone https://github.com/bourbask/LeKickerFou.git
cd LeKickerFou

# Installation des dépendances
cargo check

# Tests
cargo test

# Lancement en mode debug
cargo run -- --channel 123456789
```

### 📦 Dépendances principales

- **serenity** `0.12` - SDK Discord
- **tokio** `1.0` - Runtime async
- **tokio-cron-scheduler** `0.14` - Planificateur de tâches
- **clap** `4.0` - Interface CLI
- **anyhow** `1.0` - Gestion d'erreurs
- **serde** `1.0` - Sérialisation JSON

### 🏗️ Architecture

```
src/
├── main.rs                  # Point d'entrée et CLI
├── bot/
│   ├── mod.rs              # Structure principale du bot
│   ├── events.rs           # Gestionnaires d'événements
│   └── voice_manager.rs    # Logique de déconnexion
├── config/
│   ├── mod.rs              # Configuration et validation
│   └── cron.rs             # Gestion des expressions cron
└── utils/
    ├── logging.rs          # Système de logs
    └── errors.rs           # Erreurs personnalisées
```

### 🧪 Tests

```bash
# Tests unitaires
cargo test

# Tests d'intégration
cargo test --test integration

# Coverage
cargo tarpaulin --out Html
```

### 📋 Standards de code

- **Formatage** : `cargo fmt`
- **Linting** : `cargo clippy`
- **Documentation** : `cargo doc --open`
- **Sécurité** : `cargo audit`

## 🤝 Contribution

### 🐛 Signaler un bug

1. Vérifiez que le bug n'est pas déjà signalé
2. Ouvrez une [issue](https://github.com/bourbask/LeKickerFou/issues) avec :
   - Description détaillée
   - Étapes pour reproduire
   - Logs d'erreur
   - Environnement (OS, version Rust, etc.)

### 💡 Proposer une fonctionnalité

1. Ouvrez une [discussion](https://github.com/bourbask/LeKickerFou/discussions)
2. Décrivez le cas d'usage
3. Attendez les retours avant de développer

### 🔧 Contribuer au code

1. Fork le projet
2. Créez une branche : `git checkout -b feature/ma-fonctionnalite`
3. Committez : `git commit -m "feat: ajouter ma fonctionnalité"`
4. Poussez : `git push origin feature/ma-fonctionnalite`
5. Ouvrez une Pull Request

### 📝 Convention de commits

```
feat: nouvelle fonctionnalité
fix: correction de bug
docs: documentation
style: formatage
refactor: refactoring
test: tests
chore: maintenance
```

## 🚀 Roadmap

### Version 1.1.0

- [ ] Interface web de configuration
- [ ] Métriques Prometheus
- [ ] Support multi-serveurs
- [ ] Notifications personnalisables

### Version 1.2.0

- [ ] Conditions avancées (nombre d'utilisateurs, rôles)
- [ ] Intégration webhook
- [ ] API REST pour configuration
- [ ] Dashboard de monitoring

### Version 2.0.0

- [ ] Support des slash commands Discord
- [ ] Base de données pour historique
- [ ] Clustering pour haute disponibilité
- [ ] Plugin système

## 📊 Statistiques

![GitHub stars](https://img.shields.io/github/stars/bourbask/LeKickerFou?style=social)
![GitHub forks](https://img.shields.io/github/forks/bourbask/LeKickerFou?style=social)
![GitHub issues](https://img.shields.io/github/issues/bourbask/LeKickerFou)
![GitHub pull requests](https://img.shields.io/github/issues-pr/bourbask/LeKickerFou)

## 👥 Communauté

- 💬 [Discussions GitHub](https://github.com/bourbask/LeKickerFou/discussions)
- 🐛 [Issues](https://github.com/bourbask/LeKickerFou/issues)

## 📄 License

Ce projet est sous licence MIT. Voir le fichier [LICENSE](LICENSE) pour plus de détails.

---

<div align="center">

**⭐ Si ce projet vous a été utile, n'hésitez pas à lui donner une étoile ! ⭐**

Fait avec ❤️ en Rust

</div>
