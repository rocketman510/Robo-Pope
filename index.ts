import { Client, Events, GatewayIntentBits, Collection, MessageFlags, ContainerBuilder, flatten, TextDisplayBuilder, ButtonBuilder, ButtonStyle, ThumbnailBuilder, SeparatorSpacingSize, ButtonInteraction } from "discord.js";
import type { Command, Button, Modal, SelectionMenu } from "./deploy";
import deploy, { deply_member_count } from "./deploy";
import { error, trace } from "node:console";
import { Browser } from 'puppeteer';
import { handleLevel, handleReaction } from "./level";
import type { Db } from "mongodb"
import { handleOwsMessage } from "./functions/one_word_story";
import { handle_join } from "./functions/dyn_voice_channel";
import { handel_bible_mention, handel_reaction_bible } from "./functions/mentions_bible";
import fs from "fs";
import { get_welcome_banner } from "./functions/welcome_banner";
import { handle_message, type ChatLogEntry } from "./functions/ai";
import { Button as UiButton, Thumbnail, Page, Section, TextDisplay, ActionRow, Window, Separator, MediaGallery, ProgressBar, ProgressBarSize  } from "./functions/ui_framework/ui.ts"

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
        highlight_color: Collection<string, number>;
        ai_message_buffer: Collection<string, ChatLogEntry[]>;
        ai_is_thinking: boolean;
        ai_memories: Collection<string, string>;
        pages: Collection<string, Page>;
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
      await handel_bible_mention(message);

      // await handle_message(message);

      if (message.content == '?test') {
        let test = new Page("test", client, true, true)
          .addStaticElement(new Section("Test", new UiButton(async (p,i,d) => {if (i.isRepliable()) i.reply(await p.render(0, i))}, { label: async (p, d, i) => i?.user?.username ?? "No Username", style: ButtonStyle.Secondary }, null)))
          .addStaticElement(new Section("test", new Thumbnail({ url: async () => "https://images-ext-1.discordapp.net/external/UvhGUq0bLrS6rpDzZT6rt7GQyrBNQKjhhzUu_QNPeNs/%3Fformat%3Dwebp/https/images-ext-1.discordapp.net/external/-0-1pGDZj_iwVP8oHfOV2bhFdSl4GdM6Xmq4KiPWbno/https/static.wixstatic.com/media/16a265_38600247e9554deabc93de93300c667c~mv2.jpg/v1/fill/w_560%252Ch_459%252Cal_c%252Clg_1%252Cq_80/16a265_38600247e9554deabc93de93300c667c~mv2.jpg?format=webp"})))
          .addStaticElement(new Window())
          .addStaticElement(new MediaGallery([{url: "https://images-ext-1.discordapp.net/external/I0eDciGakh0_4XRqrmPsY0FsQsknas1CAz_L5kV4aTU/https/media.tenor.com/8pfnbPJLJ3gAAAPo/%25D9%2581%25D8%25B1%25D8%25A7%25D9%258A%25D8%25B2%25D9%258A-%25D9%2581%25D8%25B1%25D8%25A7%25D8%25B3.mp4"}]))
          .addStaticElement(new ProgressBar({ value: async () => 10, max: 15, width: 11, size: ProgressBarSize.Small}))
          .addDynamicElement(new TextDisplay("Test1"))
          .addDynamicElement(new TextDisplay("Test2"))
          .addDynamicElement(new TextDisplay("Test3"))
          .addDynamicElement(new TextDisplay("Test4"))
          .addDynamicElement(new TextDisplay("Test5"))
          .addDynamicElement(new TextDisplay("Test6"))
          .addDynamicElement(new TextDisplay("Test7"))
          .addDynamicElement(new TextDisplay("Test8"))
          .addDynamicElement(new Separator(async () => true, async () => SeparatorSpacingSize.Large))
          .addDynamicElement(new TextDisplay("Test9"))
          .addDynamicElement(new TextDisplay("Test10"))
          .addDynamicElement(new TextDisplay("Test11"))
          .addDynamicElement(new TextDisplay("Test12"))
          .addDynamicElement(new TextDisplay("Test13"))
          .addDynamicElement(new TextDisplay("Test14"))
          .addDynamicElement(new TextDisplay("Test15"))
          .addDynamicElement(new TextDisplay("Test16"))

        const next_button = new UiButton(
          async (p, i, d) => {
            await (i as ButtonInteraction).update(await p.render(d, i));
          },
          {style: ButtonStyle.Secondary, label: "next", disabled: async (p, d, i) => (d ?? 0) >= p.dynamicElements.length - p.dynamicStartMax},
          async (p, d) => {
            return p.next(d)
          }
        )
        const previous_button = new UiButton(
          async (p, i, d) => {
            await (i as ButtonInteraction).update(await p.render(d, i));
          },
          {style: ButtonStyle.Secondary, label: "previous", disabled: async (p, d, i) => d == 0},
          async (p, d) => {
            return p.previous(d)
          }
        )

        test.addStaticElement(new ActionRow([previous_button, next_button]));

        message.reply(await test.render(0))
      }
    });

    client.on(Events.MessageReactionAdd, async (reaction, user) => {
      await handleReaction(reaction, user);
      await handel_reaction_bible(reaction, user);
    });

    client.on(Events.GuildMemberAdd, async (member) => {
      const welcome_banner = await get_welcome_banner(member.user);
      const channel = await client.channels.fetch(ensure(process.env.WELCOME_CHANNEL));
      if (!channel) return;
      if (!channel.isSendable()) return;

      await channel.send({files: [welcome_banner]});
      await deply_member_count(client);
      fs.unlinkSync(welcome_banner);
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
        if (/^\w.-+[\w-+/=]*$/.test(interaction.customId)) {
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
