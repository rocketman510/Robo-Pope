import { type Client, type Channel, Collection, Message, User, MessageReaction, MessageFlags } from "discord.js";
import { readFileSync, existsSync } from 'fs';
import fs from 'fs';
import { resolve } from 'path';
import { Db } from "mongodb";

/**
 * This is for tthe users Level banner display settings.
 * _trans = Transparency 0-100
 * _frost = Frosted glass appearance
*/
export interface LevelSettings {
  user_id: string;
  guild_id: string | null;
  frost: boolean;
  primary_color: number;
  primary_color_trans: number;
  secondary_color: number;
  secondary_color_trans: number;
  text_color: number;
  text_color_trans: number;
  shadow_strength: number;
  shadow_color: number;
  has_costome_background: boolean;
  backgrond_url: string;
  is_large: boolean;
}

async function getMessages(before: string, channel: Channel) {
  const messages = await channel.messages.fetch({ limit: 100, before: before });
  return messages
}

export async function getMessageHistory(client: Client, channel: Channel, guild_id: string) {
  try {
    const firstMessage = await channel.messages.fetch({ limit: 1 });

    if (firstMessage.size < 1) {
      return
    }

    let beforeMessage = firstMessage.first().id;


    let number_of_messages = 1;

    while (true) {
      const messages = await getMessages(beforeMessage, channel);
      for (const [key, value] of messages) {
        number_of_messages++;
        addUserMessage(client, value);
        console.log(value.author.displayName + " with " + value.content);
      }

      if (messages.size < 100) {
    return
      }
      beforeMessage = messages.last().id;
    }
  } catch (error) {
    console.error('Fetch failed:', error);
  }
}

export function addUserMessage(client: Client, message: Message) {
  const guild_id = message.guildId!;
  const wordCount = countWords(message.content);
  const userKey = message.author.id;
  let members = client.messages.get(guild_id) ?? client.messages.set(guild_id, new Collection()).get(guild_id)!;
  members.set(userKey, (members.get(userKey) ?? 0) + wordCount);
}

function countWords(str: string): number {
  return str.trim().split(/\s+/).filter(Boolean).length;
}

export function getLevel(xp: number) {
  let level = 0;
  let xp_pool = xp;
  let xp_to_next_lvl = 200;

  while (xp_pool > xp_to_next_lvl) {
    level++;
    xp_to_next_lvl += 10;
    xp_pool -= xp_to_next_lvl;
  }

  return {
    level,
    min_xp: xp_pool,
    max_xp: xp_to_next_lvl,
    total_max_xp: xp + (xp_to_next_lvl - xp_pool),
  }
}

export async function getLevelBanner(user: User, guildId: string, levelsetting: LevelSettings) {
  let level_setting: LevelSettings = {
    user_id: '',
    guild_id: '',
    frost: true,
    primary_color: 0x5865f2,
    primary_color_trans: 0.7,
    secondary_color: 0xe0e3ff,
    secondary_color_trans: 0,
    text_color: 0xe0e3ff,
    text_color_trans: 1,
    shadow_strength: 0.5,
    shadow_color: 0x1a1a1a,
    has_costome_background: true,
    backgrond_url: 'https://media.discordapp.net/attachments/1466974789490184204/1483667216598831124/Screenshot_20260317-222129_Gallery.jpg?ex=69c2ac85&is=69c15b05&hm=22520ba3e652bcf62c718bab3971889c8b4ee7dff71f02a334e46c80149845a8&animated=true',
    is_large: false,
  };
  const htmlPath = level_setting.is_large ? resolve('./assets/level_large.html') : resolve('./assets/level_small.html');
  const cssPath = level_setting.is_large ? resolve('./assets/level_large.css') : resolve('./assets/level_small.css');
  const imagePath = resolve('./cache/level.png')

  const height = level_setting.is_large ? 125 : (level_setting.has_costome_background ? 8 : 0) + 22;
  const width = level_setting.is_large ? 512 : (level_setting.has_costome_background ? 8 : 0) + 256;

  let html = readFileSync(htmlPath, 'utf-8');
  let css = readFileSync(cssPath, 'utf-8');
  
  const level = getLevel(user.client.messages.get(guildId)?.get(user.id)!);
  if (!level) {return false}

  const replaceCSS = {
    PRIMARYCOLOR: hexNumToStr(level_setting.primary_color, level_setting.primary_color_trans),
    SECONDARYCOLOR: hexNumToStr(level_setting.secondary_color, level_setting.secondary_color_trans),
    FROST: level_setting.frost ? "10" : "0",
    TEXTCOLOR: hexNumToStr(level_setting.text_color, level_setting.text_color_trans),
    SHADOWCOLOR: hexNumToStr(level_setting.shadow_color, level_setting.shadow_strength),
    BACKGRONDURL: level_setting.backgrond_url,
  }

  css = css.replace(/\$\{(.*?)\}/g, (_, repName) => {
    return replaceCSS[repName] ?? '';
  });

  const replaceHTML = {
    CSS: css,
    USERNAME: user.displayName,
    USERAVATARURL: user.avatarURL(),
    LEVEL: level.level,
    MAXXP: level.max_xp,
    MINXP: level.min_xp,
    HASCOSTOMEBACKGROUND: level_setting.has_costome_background ? "img":"trans",
  }

  html = html.replace(/\$\{(.*?)\}/g, (_, repName) => {
    return replaceHTML[repName] ?? '';
  });

  const browser = user.client.browser;

  const page = await browser.newPage();

  await page.setViewport({height, width})

  console.log(height, width);
  

  await page.setContent(html);

  await page.evaluate(async () => {
    await document.fonts.ready;
  });

  await waitForFileDeletion(imagePath);

  await page.screenshot({
    path: imagePath,
    omitBackground: true, // VERY IMPORTANT
    fullPage: true,
  });

  //await page.close();

  return imagePath;
}

async function waitForFileDeletion(filePath: string) {
  while (existsSync(filePath)) {
    // wait 100ms before checking again
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
}

/**
 * This function handles leveling up and attributing proper XP.
 * @param {Client} client
 * @param {Message} message
 */
export async function handleLevel(client: Client, message: Message) {
  if (message.author.bot) {return};
  if (client.is_counting_messages) {return};

  addUserMessage(client, message);

  const guild_id = message.guildId;
  const user_id = message.author.id;
  const user_xp = client.messages.get(guild_id!)?.get(user_id);
  let next_level = client.xp.get(guild_id!)?.get(user_id);

  if (!user_xp || !next_level) {return};

  if (user_xp >= next_level) {
    const level = getLevel(user_xp).level;
    if (level % 10 === 0) {
      await message.react('<:Click_to_see_level:1484691165281255457>')
    } else {
      await message.react('<:Click_to_see_level:1484622933061271775>');
    }
    client.xp.get(guild_id!)?.set(user_id, getLevel(user_xp).total_max_xp); // Sets new milestone
  }
}

export async function handleReaction(reaction: MessageReaction, user: User) {
  const client = reaction.client;
  if (user.bot) {return};
  if (client.is_counting_messages) {return};
  if (user.id != reaction.message.author?.id) {return};
  if (reaction.emoji.id != '1484622933061271775') {return};

  const words = client.messages.get(reaction.message.guildId!)!.get(user.id);

  if (!words) {return};

  const level = getLevel(words!).level;

  if (!level) {return};

  reaction.message.reply({ content: `You made it to Level ${level} :partying_face:` });
}

export async function getLevelBannerSettings(client: Client, user_id: string, guild_id: string): Promise<LevelSettings> {
  const default_settings: LevelSettings = {
    user_id,
    guild_id,
    primary_color: 0x5865f2,
    primary_color_trans: 1.0,
    primary_frost: false,
    secondary_color: 0xe0e3ff,
    secondary_color_trans: 1.0,
    secondary_frost: false,
    text_color: 0xe0e3ff,
    text_color_trans: 1.0,
    shadow_color: 0x1a1a1a,
    shadow_strength: 1.0,
    has_costome_background: false,
    backgrond_url: '',
    is_large: false,
  };

  const db = client.db;

  const collection = db.collection<LevelSettings>("level_settings");

  const result = await collection.findOneAndUpdate(
    { user_id, guild_id },
    { $setOnInsert: default_settings },
    { upsert: true, returnDocument: "after" }
  );

  if (!result) {
    throw new Error("Failed to return a valid LevelSettings type from DB.");
  }

  return result as LevelSettings;
}

function hexNumToStr(hex: number, alpha: number): string {
  const rgb = hex.toString(16).padStart(6, "0").toUpperCase();
  const a = Math.round(Math.max(0, Math.min(1, alpha)) * 255)
    .toString(16)
    .padStart(2, "0")
    .toUpperCase();

  return `#${rgb}${a}`;
}
