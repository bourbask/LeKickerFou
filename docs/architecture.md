# 🧠 Architecture du bot Discord — LeKickerFou

Ce document présente l'architecture fonctionnelle du bot LeKickerFou, afin de visualiser les interactions entre les composants, leurs rôles et les points d'intelligence du système.

---

## 📦 Vue modulaire de l'architecture

```mermaid
graph TD
    main[main.rs]
    config[config::ConfigManager]
    serenity[serenity::Client]
    handler[bot::BotHandler]
    scheduler[Tokio Cron Job]
    voiceMgr[VoiceChannelManager]
    warnMgr[WarningManager]
    utils[utils.rs **log + token**]

    main --> config
    main --> utils
    main --> serenity
    serenity --> handler
    handler --> scheduler
    scheduler --> voiceMgr
    voiceMgr --> warnMgr
    voiceMgr --> utils
    warnMgr --> utils
```

---

## 📋 Flux global de démarrage

```mermaid
sequenceDiagram
  participant CLI as Lancement CLI (main.rs)
  participant Config as ConfigManager
  participant Discord as Discord / Serenity
  participant Bot as BotHandler
  participant Cron as CronScheduler
  participant Voice as VoiceManager
  participant Warn as WarningManager

  CLI->>Config: Args + .env → load_or_create_configuration
  CLI->>Discord: Crée Client Serenity
  Discord->>Bot: EventHandler.ready()
  Bot->>Cron: Planifie tâche via cron
  Cron->>Voice: check_and_disconnect_users()
  Voice->>Warn: send_warning()
  Voice->>Discord: Déconne utilisateurs
```

---

### ⏱️ Tâche planifiée : déroulement complet (corrigé)

```mermaid
flowchart TD
    cronjob(Cron planifié) --> check[🔍 Vérifie présence dans canal vocal]
    check --> test_warn{Avertissements activés ?}

    test_warn -- "Non" --> kick_all[🔇 Déconnecter tous les membres]
    test_warn -- "Oui" --> send_warn[📢 Envoyer message d'avertissement]

    send_warn --> wait[⏳ Attendre délai configuré]
    wait --> test_mode{Mode ''warning only'' ?}

    test_mode -- "Oui" --> end1[✅ Fin tâche **rappel seulement**]
    test_mode -- "Non" --> recheck[🔁 Réevaluer les membres restants]

    recheck --> test_empty{Encore présents ?}
    test_empty -- "Non" --> end2[✅ Rien à faire]
    test_empty -- "Oui" --> kick_remaining[🔇 Déconnexion ciblée]

    kick_all --> end3[✅ Fin tâche **directe**]
    kick_remaining --> end4[✅ Fin tâche **post-avertissement**]
```

---

## 📁 Structure des fichiers

```mermaid
classDiagram
class main_rs {
    +main()
}
class BotHandler {
    +ready()
    +start_voice_monitoring()
}
class ConfigManager {
    +load_or_create_configuration()
    +import_configuration()
    +export_configuration()
}
class VoiceChannelManager {
    +check_and_disconnect_users()
}
class WarningManager {
    +send_warning()
    +wait_warning_delay()
}

main_rs --> ConfigManager
main_rs --> BotHandler
BotHandler --> VoiceChannelManager
VoiceChannelManager --> WarningManager
VoiceChannelManager --> DiscordAPI
WarningManager --> DiscordAPI
```

---
