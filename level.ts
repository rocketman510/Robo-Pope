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

export function getLevel(xp: number): { level: number; min_xp: number; max_xp: number } {
  let level = 0;
  let xp_pool = xp;
  let max_xp = 50;

  while (xp_pool >= max_xp) {
    xp_pool -= max_xp;
    level++;

    max_xp = Math.floor(max_xp * 1.5);
  }

  return {
    level,
    min_xp: xp_pool,
    max_xp
  };
}
