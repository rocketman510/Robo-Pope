import type { Message } from "discord.js";
import { ensure } from "..";

export async function handleOwsMessage(message: Message) {
  if (!message.channelId.includes(JSON.parse(ensure(process.env.ONE_WORD_STORY_IDS, 'No ONE_WORD_STORY_IDS ENV')))) {return}
  const message_before = await message.channel.messages.fetch({ limit: 1, before: message.id});
  if (message_before.first()?.author.id == message.author.id) {
    await message.delete();
  }
  if (!/^\S+$/g.test(message.content)) {
    await message.delete();
  }
}
