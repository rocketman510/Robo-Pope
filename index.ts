import { Client, Events, GatewayIntentBits, Collection, MessageFlags} from "discord.js";
import type { Command, Button } from "./deploy";
import deploy from "./deploy";
import { error, log } from "node:console";
import { Browser } from 'puppeteer';
import { handleLevel, handleReaction } from "./level";
import type { Db } from "mongodb"

declare module "discord.js" {
    export interface Client {
        commands: Collection<string, Command>;
        buttons: Collection<string, Button>;
        messages: Collection<string, Collection<string, number>>;
        xp: Collection<string, Collection<string, number>>;
        shouldStopSpam: boolean;
        is_counting_messages: boolean;
        browser: Browser;
        db: Db;
    }
}

export function ensure<T>(value: T | null | undefined, error?: string): T {
    if (value == null || value == undefined) throw new Error(error ?? "Unexpected null!");
    return value;
}

const client = new Client({ intents: [
  GatewayIntentBits.Guilds,
  GatewayIntentBits.GuildMessages,
  GatewayIntentBits.DirectMessages,
  GatewayIntentBits.GuildMembers,
  GatewayIntentBits.MessageContent,
  GatewayIntentBits.GuildMessageReactions
]});

client.once(Events.ClientReady, async readyClient => {
  await deploy(client);
  console.log(`Ready! Logged in as ${readyClient.user.tag}`);

  client.on(Events.MessageCreate, async (message) => {
    await handleLevel(client, message);
  });

  client.on(Events.MessageReactionAdd, async (reaction, user) => {
    await handleReaction(reaction, user);
  })

  client.on(Events.InteractionCreate, (interaction) => {
    if (interaction.isChatInputCommand()) {
      const command = client.commands.get(interaction.commandId);
      if (!command) return;
      let script = command.execute;
      try {
        script(interaction);
      } catch (err) {error(err)};
    } else if (interaction.isButton()) {
      const button = client.buttons.get(interaction.customId)
      if (!button) return;
      try {
        button.execute(interaction)
      } catch (err) {error(err)}
    }; 
  });
});


client.login(process.env.DISCORD_TOKEN);
