import { Client, Events, GatewayIntentBits, Collection, MessageFlags} from "discord.js";
import type { Command, Button } from "./deploy";
import deploy from "./deploy";
import { error } from "node:console";

declare module "discord.js" {
    export interface Client {
        commands: Collection<string, Command>;
        buttons: Collection<string, Button>;
    }
}

const client = new Client({ intents: [
  GatewayIntentBits.Guilds,
  GatewayIntentBits.DirectMessages,
  GatewayIntentBits.GuildMembers
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
    } else if (interaction.isButton()) {
      const button = client.buttons.get(interaction.customId)
      if (!button) return;
      try {
        button.execute(interaction)
      } catch (err) {error(err)}
    }; 
  });
});

client.commands = new Collection<string, Command>();
client.buttons = new Collection<string, Button>();
client.shouldStopSpam = false;

await deploy(client);

client.login(process.env.DISCORD_TOKEN);
