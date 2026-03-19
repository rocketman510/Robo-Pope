import { Collection, SlashCommandBuilder, ChatInputCommandInteraction, Client, type Interaction, type ButtonInteraction } from "discord.js";
import { REST, Routes } from 'discord.js';
import { error, log } from "node:console";
import { readdir } from "node:fs/promises";
import { getMessageHistory, getLevelBanner } from "./level";
import puppeteer from 'puppeteer';
import type { Browser } from "puppeteer";
import fs from "fs";

export interface Command {
    data: SlashCommandBuilder;
    execute: (interaction: ChatInputCommandInteraction) => Promise<void>;
}

export interface Button {
  data: string;
  execute: (Interaction: ButtonInteraction) => Promise<void>;
}

export default async function(client: Client) {
  client.commands = new Collection<string, Command>();
  client.buttons = new Collection<string, Button>();
  client.messages = new Collection<string, Collection<string, number>>()

  client.shouldStopSpam = false;
  client.is_counting_messages = true;
  console.log("Clearing Cache");
  if (fs.existsSync('./cache/level.png')) {
    fs.unlinkSync('./cache/level.png');
  }
  console.log("Louding Commands...");
  await deply_commands(client.commands);
  console.log("Louding Buttons...");
  await deply_buttons(client.buttons);
  console.log("Louding Browser...");
  client.browser = await puppeteer.launch({headless: true, executablePath: process.env.PUPPETEEREXECUTABLEPATH});
  console.log(client.browser);
  console.log("Fetching Messages...");
  await get_user_messages_for_all(client);
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
