import { Collection, SlashCommandBuilder, ChatInputCommandInteraction, Client, type ButtonInteraction, ModalSubmitInteraction, UserSelectMenuInteraction, ChannelSelectMenuInteraction, MentionableSelectMenuInteraction, RoleSelectMenuInteraction, StringSelectMenuInteraction, MessageFlags, type GuildTextBasedChannel, PermissionsBitField } from "discord.js";
import { REST, Routes } from 'discord.js';
import { error, log } from "node:console";
import { readdir } from "node:fs/promises";
import { getMessageHistory, getLevelBanner, getLevel } from "./level";
import puppeteer from 'puppeteer';
import { MongoClient, Db } from "mongodb";
import type { Browser } from "puppeteer";
import fs from "fs";
import path from "node:path";
import { ensure } from ".";
import { generateLeaderbord, update_leaderbord } from "./functions/level_leaderboard";

export interface Command {
    data: SlashCommandBuilder;
    execute: (interaction: ChatInputCommandInteraction) => Promise<void>;
}

export interface Button {
  data: string;
  execute: (Interaction: ButtonInteraction) => Promise<void>;
}

export interface Modal {
  data: string;
  execute: (Interaction: ModalSubmitInteraction) => Promise<void>;
}

export interface SelectionMenu {
  data: string;
  execute: (Interaction: UserSelectMenuInteraction | ChannelSelectMenuInteraction | StringSelectMenuInteraction | RoleSelectMenuInteraction | MentionableSelectMenuInteraction) => Promise<void>;
}

export default async function(client: Client) {
  client.commands = new Collection<string, Command>();
  client.buttons = new Collection<string, Button>();
  client.modals = new Collection<string, Modal>();
  client.selection_menus = new Collection<string, SelectionMenu>();
  client.messages = new Collection<string, Collection<string, number>>();
  client.xp = new Collection<string, Collection<string, number>>();

  client.shouldStopSpam = false;
  client.is_counting_messages = true;

  console.log("Clearing Cache");
  clear_cache('./cache/');
  console.log("Loading Commands...");
  await deply_commands(client.commands);
  console.log("Loading Buttons...");
  await deply_buttons(client.buttons);
  console.log("Loading Modals...");
  await deply_modals(client.modals);
  console.log('Loading Selection Menus...');
  await deply_selection_menus(client.selection_menus)
  console.log("Loading Browser...");
  client.browser = await puppeteer.launch({headless: ensure(process.env.DEV_MODE, "No DEV_MODE ENV") == 'false', executablePath: process.env.PUPPETEEREXECUTABLEPATH});
  console.log("Loading Member Count...");
  await deply_member_count(client)
  console.log("Fetching Messages...");
  await get_user_messages_for_all(client);
  console.log("Clearing leaderbord Channel");
  await clear_leaderbord(client);
  console.log("Calculating Level Data..");
  deply_xp(client);
  console.log("Connecteing to DB");
  client.db = await deply_db();
}

async function deply_commands(client_commands: Collection<string,Command>) {
  const path = './commands/'
  const files = await readdir(path);

  let commands = new Map<string, Command>();
  let appCommands = [];

  for (const index in files) {
    const file = path + files[index];

    const module = await import(file);
    const command = module.default as Command;

    appCommands.push(command.data.toJSON());
    commands.set(command.data.name, command);
  }

  const rest = new REST().setToken(process.env.DISCORD_TOKEN!);

  const result = await rest.put(Routes.applicationCommands(process.env.CLIENT_ID!), { body: appCommands }) as any[];

  for (const index in result!) {
    const element = result[index];

    const script = commands.get(element.name);

    if (!script) {
      error(`Failed to find Command Object for Command: ${element.name}\nThis can lead to inconsistent behavior and commands fail to execute`);
      return;
    }

    client_commands.set(element.id, script);
  } 
}

async function deply_buttons(buttons: Collection<string, Button>) {
  const path = './button/'
  const files = await readdir(path);

  for (const index in files) {
    const file = path + files[index];

    const module = await import(file);
    const button = module.default as Button;

   buttons.set(button.data, button); 
  }
}

async function deply_modals(modals: Collection<string, Modal>) {
  const path = './modals/'
  const files = await readdir(path);

  for (const index in files) {
    const file = path + files[index];

    const module = await import(file);
    const modal = module.default as Modal;
    modals.set(modal.data, modal); 
  }
}

async function deply_selection_menus(selection_menus: Collection<string, SelectionMenu>) {
  const path = './selection_menus/'
  const files = await readdir(path);

  for (const index in files) {
    const file = path + files[index];

    const module = await import(file);
    const selection_menu = module.default as SelectionMenu;
    selection_menus.set(selection_menu.data, selection_menu);
  }
}

async function get_user_messages_for_all(client: Client) {
  const guilds = client.guilds.cache;

  for (const [guild_id, guild] of guilds) {
    const channels = guild.channels.cache;
    for (const [channel_id, channel] of channels) {
      await getMessageHistory(client, channel, guild_id);
    }
  }
  console.log(client.messages);
  console.log("DONE");
  client.is_counting_messages = false;
}

/**
 * Sets the client.xp witch is a Map of Maps that hold the users next level milestone per guild.
 * @param {Client} client - Takes the client obj where .xp and .messages are.
 * */
function deply_xp(client: Client) {
  const words = client.messages;
  let xp = client.xp;

  for (const [guild_id, guild] of words) {
    let guild_xp = xp.get(guild_id) ?? xp.set(guild_id, new Collection()).get(guild_id);
    for (const [user_id, words_said] of guild) {
      guild_xp!.set(user_id, getLevel(words_said).total_max_xp);
    }
  }
}

export async function deply_db() {
  const uri = `mongodb://admin:${process.env.DB_PASSWORD}@${process.env.DB_DOMAIN}`;
  const client = new MongoClient(uri);
  await client.connect();
  console.log("Connected to MongoDB!");
  return client.db("Robo-Pope-DB");
}

function clear_cache(cacheDir: string) {
  if (fs.existsSync(cacheDir)) {
    fs.readdirSync(cacheDir).forEach(item => {
      const itemPath = path.join(cacheDir, item);
      if (fs.lstatSync(itemPath).isFile()) {
        fs.unlinkSync(itemPath); // delete file
        console.log(`Deleted file: ${item}`);
      } else if (fs.lstatSync(itemPath).isDirectory()) {
        fs.rmSync(itemPath, { recursive: true, force: true }); // delete folder
        console.log(`Deleted folder: ${item}`);
      }
    });
  }
}

async function clear_leaderbord(client: Client) {
  for (const channel_id of JSON.parse(ensure(process.env.LEADERBOARD_CHANNEL_ID, "No LEADERBOARD_CHANNEL_ID Environment variable."))) {
    await update_leaderbord(client, channel_id);
  }
}

export async function deply_member_count(client: Client) {
  const channes = JSON.parse(ensure(process.env.MEMBER_COUNT,"No MEMBER_COUNT ENV"));
  for (const channel_id of channes) {
    const channel = await client.channels.fetch(channel_id);
    if (!channel) {console.error(`${channel_id} is a not accessible by the bot`); continue;};
    if (!channel.isVoiceBased()) {continue};
    const everyone = channel.guild.roles.everyone;
    if (!channel.permissionsFor(everyone).has(PermissionsBitField.Flags.ViewChannel)) {
      await channel.permissionOverwrites.create(everyone, { ViewChannel: true } );
    }
    if (channel.permissionsFor(everyone).has(PermissionsBitField.Flags.Connect)) {
      await channel.permissionOverwrites.create(everyone, { Connect: false } );
    }
    const members = channel.guild.memberCount;
    await channel.setName(`● ${members}`);
  }
}
