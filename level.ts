import { type Client, type Channel, Collection, Message } from "discord.js";

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
  console.log(client.messages);
}

function countWords(str: string): number {
  return str.trim().split(/\s+/).filter(Boolean).length;
}

export function getLevel(xp: number) {
  const a = 200;
  const b = 10;
  const c = 1.05;

  // inline total XP function
  const totalXP = (level: number) => a * level + b * (Math.pow(c, level) - 1);

  // inline XP needed for next level
  const xpForLevel = (level: number) => totalXP(level + 1) - totalXP(level);

  // binary search to find the level
  let low = 0;
  let high = 1000; // adjust if needed for max possible level

  while (low < high) {
    const mid = Math.floor((low + high + 1) / 2);
    if (totalXP(mid) <= xp) {
      low = mid;
    } else {
      high = mid - 1;
    }
  }

  const level = low;
  const min_xp = xp - totalXP(level);
  const max_xp = xpForLevel(level);

  return {
    level,
    min_xp: Math.floor(min_xp),
    max_xp: Math.floor(max_xp),
  };
}
