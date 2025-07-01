import { Client, GatewayIntentBits } from "discord.js";
import cron from "node-cron";
import "dotenv/config";

// Configurer les couleurs pour les logs
const logStyles = {
  reset: "\x1b[0m",
  bright: "\x1b[1m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  red: "\x1b[31m",
};

// Logger personnalisé
const logger = {
  info: (...args) =>
    console.log(
      `${logStyles.green}🛈${logStyles.reset} ${getTimestamp()}`,
      ...args
    ),
  warn: (...args) =>
    console.warn(
      `${logStyles.yellow}⚠${logStyles.reset} ${getTimestamp()}`,
      ...args
    ),
  error: (...args) =>
    console.error(
      `${logStyles.red}𐄷${logStyles.reset} ${getTimestamp()}`,
      ...args
    ),
};

function getTimestamp() {
  return `[${new Date().toISOString()}]`;
}

// Validation des variables d'environnement
const requiredEnvVars = ["DISCORD_TOKEN", "GUILD_ID", "CHANNEL_ID"];
const missingVars = requiredEnvVars.filter((v) => !process.env[v]);
if (missingVars.length) {
  logger.error(`Variables manquantes dans .env: ${missingVars.join(", ")}`);
  process.exit(1);
}

const client = new Client({
  intents: [GatewayIntentBits.Guilds, GatewayIntentBits.GuildVoiceStates],
});

client.once("ready", () => {
  logger.info(`Bot connecté: ${client.user.tag} (ID: ${client.user.id})`);
  logger.info(
    `Config: Guild=${process.env.GUILD_ID} | Channel=${process.env.CHANNEL_ID}`
  );

  scheduleVoiceCheck();
});

async function handleVoiceChannelCheck() {
  try {
    const guild = await client.guilds
      .fetch(process.env.GUILD_ID)
      .catch((err) => {
        if (err.code === 10004)
          throw new Error(`Serveur non trouvé (ID: ${process.env.GUILD_ID})`);
        throw err;
      });

    const channel = await guild.channels
      .fetch(process.env.CHANNEL_ID)
      .catch((err) => {
        if (err.code === 10003)
          throw new Error(`Salon non trouvé (ID: ${process.env.CHANNEL_ID})`);
        throw err;
      });

    if (!channel?.isVoiceBased()) {
      throw new Error(
        `Le salon ${
          channel?.name || process.env.CHANNEL_ID
        } n'est pas un salon vocal`
      );
    }

    const members = channel.members;
    if (members.size === 0) {
      logger.info(`Aucun membre dans ${channel.name}`);
      return;
    }

    logger.info(`Membres à déconnecter dans ${channel.name}: ${members.size}`);

    for (const member of members.values()) {
      try {
        await member.voice.disconnect("Déconnexion automatique");
        logger.info(`${member.user.tag} déconnecté`);

        await logToChannel(
          guild,
          `🔇 ${member.user.tag} déconnecté de ${channel.name}`
        );
      } catch (err) {
        logger.error(`Échec déconnexion ${member.user.tag}: ${err.message}`);
      }
    }
  } catch (err) {
    handleCheckError(err);
  }
}

function scheduleVoiceCheck() {
  cron.schedule("* * * * *", async () => {
    logger.info("Début de la vérification...");
    await handleVoiceChannelCheck();
    logger.info("Vérification terminée");
  });
}

async function logToChannel(guild, message) {
  if (!process.env.LOG_CHANNEL_ID) return;

  try {
    const channel = await guild.channels.fetch(process.env.LOG_CHANNEL_ID);
    if (channel?.isTextBased()) {
      await channel.send(message);
    }
  } catch (err) {
    logger.error(`Échec d'envoi des logs: ${err.message}`);
  }
}

function handleCheckError(err) {
  const knownErrors = {
    10004: "Le bot n'est pas sur le serveur. Invitation nécessaire.",
    10003: "Salon vocal introuvable. Vérifiez l'ID du channel.",
  };

  const message = knownErrors[err.code] || err.message;
  logger.warn(`ERREUR: ${message}`);

  if (err.code === 10004) {
    logger.warn(`Invitez le bot avec ce lien: ${generateBotInviteLink()}`);
  }
}

function generateBotInviteLink() {
  const permissions = ["ViewChannel", "Connect", "MoveMembers"];
  return `https://discord.com/api/oauth2/authorize?client_id=${client.user?.id}&permissions=${permissions}&scope=bot`;
}

// Gestion propre des arrêts
process.on("SIGINT", () => {
  logger.info("Déconnexion du bot...");
  client.destroy();
  process.exit();
});

client.login(process.env.DISCORD_TOKEN);
