import { type Client, type Channel, Collection, Message, User, MessageReaction, MessageFlags } from "discord.js";
import fs from 'fs';
import { resolve } from 'path';
import { ensure } from ".";
import { error } from "console";
import level_settings from "./button/level_settings";

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
  if (!channel.isTextBased()) {return null}
  const messages = await channel.messages.fetch({ limit: 100, before: before });
  return messages
}

export async function getMessageHistory(client: Client, channel: Channel, guild_id: string) {
  try {
    if (!channel.isTextBased()) {throw "Is not a text channel"}
    const firstMessage = await channel.messages.fetch({ limit: 1 });

    let beforeMessage = ensure(firstMessage.first()?.id, "Could not feach the fisrt message in a channel. This is becasue there are none");

    let number_of_messages = 1;

    while (true) {
      const messages = ensure(await getMessages(beforeMessage, channel), "Is not a text channel");
      for (const [_, value] of messages) {
        number_of_messages++;
        addUserMessage(client, value);
        console.log(value.author.displayName + " with " + value.content);
      }

      if (messages.size < 100) {
        throw "No more messages";
      }
      beforeMessage = ensure(messages.last()?.id);
    }
  } catch (_) {}
}

export function addUserMessage(client: Client, message: Message) {
  const guild_id = ensure(message.guildId, "Message must be form a guild");
  const wordCount = countWords(message.content);
  const userKey = message.author.id;
  let members = ensure(client.messages.get(guild_id) ?? client.messages.set(guild_id, new Collection()).get(guild_id), "Failed to Set and Get a obj to the client.messages Collection");
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

export async function getLevelBanner(user: User, level_setting: LevelSettings) {
  const htmlPath = level_setting.is_large ? resolve('./assets/level_large.html') : resolve('./assets/level_small.html');
  const cssPath = level_setting.is_large ? resolve('./assets/level_large.css') : resolve('./assets/level_small.css');
  const imagePath = resolve('./cache/level.png')

  const height = level_setting.is_large ? 125 : (level_setting.has_costome_background ? 8 : 0) + 22;
  const width = level_setting.is_large ? 512 : (level_setting.has_costome_background ? 8 : 0) + 256;

  let html = fs.readFileSync(htmlPath, 'utf-8');
  let css = fs.readFileSync(cssPath, 'utf-8');

  const words = getUsersWords(user.client, user.id, level_setting.guild_id!);
  const level = getLevel(words);

  const parsedUrl = new URL(level_setting.backgrond_url).pathname.split("/").pop();

  let backgrond_url = ''

  if (fs.existsSync(process.env.CACHE_PATH! + parsedUrl!)) {
    backgrond_url = 'file://' + process.env.CACHE_PATH! + parsedUrl!;
    /*const data = fs.readFileSync(process.env.CACHE_PATH! + parsedUrl!);
    const base64 = data.toString("base64");
    const dataUrl = `data:image/png;base64,${base64}`;
    backgrond_url = dataUrl;
    console.log(backgrond_url);*/
  } else {
    backgrond_url = level_setting.backgrond_url;
    const res = await fetch(level_setting.backgrond_url);
    if (!res.ok) throw new Error(`Failed: ${res.statusText}`);

    const buffer = Buffer.from(await res.arrayBuffer());
    fs.writeFileSync(process.env.CACHE_PATH! + parsedUrl!, buffer);
  }

  const replaceCSS = {
    PRIMARYCOLOR: hexNumToStr(level_setting.primary_color, level_setting.primary_color_trans),
    SECONDARYCOLOR: hexNumToStr(level_setting.secondary_color, level_setting.secondary_color_trans),
    FROST: level_setting.frost ? "10" : "0",
    TEXTCOLOR: hexNumToStr(level_setting.text_color, level_setting.text_color_trans),
    SHADOWCOLOR: hexNumToStr(level_setting.shadow_color, level_setting.shadow_strength),
    BACKGRONDURL: backgrond_url,
  }

  css = css.replace(/\$\{(.*?)\}/g, (_, repName) => {
    const value = replaceCSS[repName as keyof typeof replaceCSS];
    return value?.toString() ?? '';
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
    const value = replaceHTML[repName as keyof typeof replaceHTML];
    return value?.toString() ?? '';
  });

  fs.writeFileSync(process.env.CACHE_PATH! + 'html.html', html);

  const browser = user.client.browser;

  const page = await browser.newPage();

  await page.setViewport({height, width})

  await page.goto('file://' + process.env.CACHE_PATH! + 'html.html')

  await page.evaluate(async () => {
    await document.fonts.ready;
  });

  await waitForFileDeletion(imagePath);

  await page.screenshot({
    path: imagePath,
    omitBackground: true,
    fullPage: true,
  });

  await page.close();

  return imagePath;
}

async function waitForFileDeletion(filePath: string) {
  while (fs.existsSync(filePath)) {
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
  try {
    const client = reaction.client;
    if (user.bot) {return};
    if (client.is_counting_messages) {return};
    if (user.id != reaction.message.author?.id) {return};
    if (reaction.emoji.id != '1484622933061271775') {return};
    if (!reaction.message.guildId) {return}

    const words = getUsersWords(client, user.id, reaction.message.guildId);

    const level = getLevel(words).level;

    reaction.message.reply({ content: `You made it to Level ${level} :partying_face:` });
  } catch (err) {error('handleReaction in ./level.ts function failed error: ' + err)}
}

export async function getLevelBannerSettings(client: Client, user_id: string, guild_id: string): Promise<LevelSettings> {
  const default_settings: LevelSettings = {
    user_id,
    guild_id,
    frost: false,
    primary_color: 0x5865f2,
    primary_color_trans: 1.0,
    secondary_color: 0xe0e3ff,
    secondary_color_trans: 1.0,
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

export async function setLevelBannerSettings(client: Client, level_settings: LevelSettings) {
    const db = client.db;
    const collection = db.collection<LevelSettings>("level_settings");

    const filter = {
        user_id: level_settings.user_id,
        guild_id: level_settings.guild_id,
    };

    const { _id, ...safeLevelSettings } = level_settings as any;

    return await collection.updateOne(
        filter,
        { $set: safeLevelSettings },
        { upsert: true }
    );
}

export function hexNumToStr(hex: number, alpha: number): string {
  const rgb = hex.toString(16).padStart(6, "0").toUpperCase();
  const a = Math.round(Math.max(0, Math.min(1, alpha)) * 255)
    .toString(16)
    .padStart(2, "0")
    .toUpperCase();

  return `#${rgb}${a}`;
}

function getUsersWords(client: Client, user_id: string | undefined | null, guild_id: string | undefined | null) {
  const fixed_user = ensure(user_id, "No valid User Id was provided for the function getUserWords");
  const fixed_guild = ensure(guild_id, "No valid Guild Id was provided for the functoion getUsersWords");
  return client.messages.ensure(fixed_guild, () => new Collection<string, number>()).ensure(fixed_user, () => 0);
}
