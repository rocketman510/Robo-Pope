import { type Client, type Channel, Collection, Message, User, MessageReaction } from "discord.js";
import { readFileSync, existsSync } from 'fs';
import fs from 'fs';
import { resolve } from 'path';

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
  const a = 200;
  const b = 10;
  const c = 1.05;

  // 1. Math.round handles floating point errors from the exponent
  const totalxp = (level: number): number => 
    Math.round(a * level + b * (Math.pow(c, level) - 1));

  let low = 0;
  let high = 1000; 

  // 2. The <= check ensures that hitting the EXACT XP requirement 
  // pushes the user into the new level instead of leaving them at 100%.
  while (low <= high) {
    const mid = Math.floor((low + high) / 2);
    if (totalxp(mid) <= xp) {
      low = mid + 1;
    } else {
      high = mid - 1;
    }
  }

  const level = high;
  const currentLevelTotal = totalxp(level);
  const nextLevelTotal = totalxp(level + 1);

  return {
    level,
    min_xp: Math.floor(xp - currentLevelTotal),
    max_xp: Math.floor(nextLevelTotal - currentLevelTotal),
    total_max_xp: nextLevelTotal,
  };
}

export async function getLevelBanner(user: User, guildId: string) {
  const htmlPath = resolve('./assets/level_small.html');
  const cssPath = resolve('./assets/level_small.css');
  const imagePath = resolve('./cache/level.png')

  let html = readFileSync(htmlPath, 'utf-8');
  const css = readFileSync(cssPath, 'utf-8');
  
  const level = getLevel(user.client.messages.get(guildId)?.get(user.id)!);
  if (!level) {return false}

  const replaceWith = {
    CSS: css,
    USERAVATARURL: user.avatarURL(),
    LEVEL: level.level,
    MAXXP: level.max_xp,
    MINXP: level.min_xp,
  }

  html = html.replace(/\$\{(.*?)\}/g, (_, repName) => {
    return replaceWith[repName] ?? '';
  });

  const browser = user.client.browser;

  const page = await browser.newPage();

  await page.setViewport({ width: 256, height: 22 });

  await page.setContent(html);

  await page.evaluate(async () => {
    await document.fonts.ready;
  });

  await waitForFileDeletion(imagePath);

  await page.screenshot({
    path: imagePath,
    omitBackground: true, // VERY IMPORTANT
  });

  await page.close();

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

  console.log(user_xp + " " + next_level);
  

  if (user_xp >= next_level) {
    await message.react('<:Click_to_see_level:1484622933061271775>');
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

  reaction.message.reply({ content: `You made it to Level ${level} :partying_face:`});
}
