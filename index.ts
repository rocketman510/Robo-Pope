import { Client, Events, GatewayIntentBits, Collection, MessageFlags, ContainerBuilder} from "discord.js";
import type { Command, Button, Modal, SelectionMenu } from "./deploy";
import deploy, { deply_member_count } from "./deploy";
import { error, log } from "node:console";
import { Browser } from 'puppeteer';
import { handleLevel, handleReaction } from "./level";
import type { Db } from "mongodb"
import { handleOwsMessage } from "./functions/one_word_story";
import { handle_join } from "./functions/dyn_voice_channel";

declare module "discord.js" {
    export interface Client {
        commands: Collection<string, Command>;
        buttons: Collection<string, Button>;
        modals: Collection<string, Modal>;
        selection_menus: Collection<string, SelectionMenu>;
        messages: Collection<string, Collection<string, number>>;
        xp: Collection<string, Collection<string, number>>;
        shouldStopSpam: boolean;
        is_counting_messages: boolean;
        browser: Browser;
        db: Db;
        ows_last_bot_message: Collection<string, string>;
        ows_sentence_history: Collection<string, string[]>;
        dyn_vc: Collection<string, string[]>;
        interaction_queue: Collection<string, number>;
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
  GatewayIntentBits.GuildMessageReactions,
  GatewayIntentBits.GuildPresences,
  GatewayIntentBits.GuildVoiceStates
]});

client.once(Events.ClientReady, async readyClient => {
  try {
    await deploy(client);
    console.log(`Ready! Logged in as ${readyClient.user.tag}`);

    client.on(Events.MessageCreate, async (message) => {
      await handleOwsMessage(message);
      await handleLevel(client, message);
      if (message.content == '?test') {
        for (let i = 0; i < 50; i++) {
          await message.channel.send(i.toString())
        }
      }
    });

    client.on(Events.MessageReactionAdd, async (reaction, user) => {
      await handleReaction(reaction, user);
    });

    client.on(Events.GuildMemberAdd, async () => {
      await deply_member_count(client);
    });

    client.on(Events.GuildMemberRemove, async () => {
      await deply_member_count(client);
    });

    client.on(Events.VoiceStateUpdate, async (oldState, newState) => {
      await handle_join(oldState, newState)
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
        let button: any = {};
        if (/^\w.-+[\w-+]*$/.test(interaction.customId)) {
          button = client.buttons.get(interaction.customId.slice(0,2))
        } else {
          button = client.buttons.get(interaction.customId)
        }
        if (!button) return;
        try {
          button.execute(interaction)
        } catch (err) {error(err)}
      } else if (interaction.isModalSubmit()) {
        const modal = client.modals.get(interaction.customId)
        if (!modal) return;
        try {
          modal.execute(interaction)
        } catch (err) {error(err)}
      } else if (interaction.isAnySelectMenu()) {
        const selection_menu = client.selection_menus.get(interaction.customId)
        if (!selection_menu) return;
        try {
          selection_menu.execute(interaction)
        } catch (err) {error(err)}
      }; 
    });
  } catch (error) {
    await sendErr(client, error as string)
  }
});

async function sendErr(client: Client, error: string) {
  console.log(process.env.ERROR_CHANNEL_ID);
  const channel = await client.channels.fetch(ensure(process.env.ERROR_CHANNEL_ID, "No ERROR_CHANNEL_ID ENV"))
  const container = new ContainerBuilder()
    .setAccentColor(0xff0000)
    .addTextDisplayComponents((td) => td.setContent(`Bot Had an Error:\n\`\`\`${error}\`\`\``));

  await channel.send({ components: [ container ], flags: [MessageFlags.IsComponentsV2] });
}


client.login(process.env.DISCORD_TOKEN);
