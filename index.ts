import { Client, Events, GatewayIntentBits, Collection, MessageFlags} from "discord.js";
import type { Command } from "./deploy";
import deploy from "./deploy";
import { error } from "node:console";

declare module "discord.js" {
    export interface Client {
        commands: Collection<string, Command>;
    }
}

const client = new Client({ intents: [
  GatewayIntentBits.Guilds
]});

client.once(Events.ClientReady, readyClient => {
  console.log(`Ready! Logged in as ${readyClient.user.tag}`);

  client.on(Events.InteractionCreate, (interaction) => {
    if (interaction.isChatInputCommand()) {
      const command = client.commands.get(interaction.commandId);
      if (!command) return;
      let script = command.execute;

      try {
        script(interaction);
      } catch (err) {error(err)};
    }; 
  });
});

client.commands = new Collection<string, Command>();

await deploy(client.commands);

client.login(process.env.DISCORD_TOKEN);
