# Changelog

All notable changes to LeKickerFou will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2024-01-15

### 🎉 Added

- **Système de permissions avec whitelist** (User/Moderator/Admin)
- **Historique et sauvegarde** automatique des configurations
- **Import/export** de configurations
- **Commandes slash Discord** complètes
- **Niveaux de verbosité** configurables (-v, -vv, -vvv)
- **Mode avertissement seul** (--warning-only)
- **Gestion des interactions Discord** (boutons, modals)
- **Validation des expressions cron**
- **Rotation automatique des sauvegardes**

### 🔧 Changed

- **Refactorisation complète** de l'architecture
- **Configuration versioned** avec migration automatique
- **Logs améliorés** avec timestamps et emojis
- **Messages d'erreur** plus explicites

### 🐛 Fixed

- **Gestion des erreurs** des déconnexions individuelles
- **Cache Discord** pour éviter les rate limits
- **Validation des IDs** de salons Discord

### 🔒 Security

- **Système de permissions** robuste
- **Validation des entrées** utilisateur

## [1.0.0] - 2024-01-01

### 🎉 Added

- **Déconnexion automatique** selon planning cron
- **Avertissements optionnels** avant déconnexion
- **Configuration par fichier JSON**
- **Logs Discord** pour traçabilité
- **Interface CLI** avec clap
- **Support des variables d'environnement**

### 🔧 Features

- Planning cron 6 parties (seconde à jour_semaine)
- Messages d'avertissement amusants
- Gestion des erreurs Discord
- Configuration par arguments CLI
- Logs console avec niveaux

[1.1.0]: https://github.com/votre-username/lekickerfou/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/votre-username/lekickerfou/releases/tag/v1.0.0

````

**Fichier bonus : `.github/workflows/ci.yml`** (pour CI/CD)

```yaml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: dtolnay/rust-toolchain@stable
    - name: Run tests
      run: cargo test --verbose

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy
    - name: Run Clippy
      run: cargo clippy -- -D warnings

  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt
    - name: Run rustfmt
      run: cargo fmt --all -- --check

  build:
    name: Build
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
    - uses: actions/checkout@v3
    - uses: dtolnay/rust-toolchain@stable
    - name: Build
      run: cargo build --release --verbose
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: lekickerfou-${{ matrix.os }}
        path: target/release/lekickerfou*
````
