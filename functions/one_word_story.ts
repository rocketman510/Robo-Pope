import { Collection, type Client, type Message } from "discord.js";
import { ensure } from "..";

export async function handleOwsMessage(message: Message) {
  const client = message.client;
  if (!message.channelId.includes(JSON.parse(ensure(process.env.ONE_WORD_STORY_IDS, 'No ONE_WORD_STORY_IDS ENV')))) {return}
  const message_before = await message.channel.messages.fetch({ limit: 1, before: message.id});
  if (message_before.first()?.author.id == message.author.id) {
    await message.delete();
  }
  if (!/^\S+$/g.test(message.content)) {
    await message.delete();
  }
  if (/[.?!]/g.test(message.content)) {
    const sentence = await get_sentence_till_last_punctuation(client, message.channelId);
    if (sentence == null) {
      return
    }
    client.ows_sentence_history
      .ensure(message.channelId, (): string[] => [])
      .unshift(sentence);
}
}

async function get_sentence_till_last_punctuation(client: Client, channel_id: string): Promise<string | null> {
  const channel = await client.channels.fetch(channel_id);
  if (!channel || !channel.isTextBased()) return null;
  let before_message = '';
  let sentence = ''

  while (true) {
    if (before_message == '') {
      var messages = await channel.messages.fetch({ limit: 100 })
    } else {
      var messages = await channel.messages.fetch({ limit: 100, before: before_message})
    }
    if (messages.size === 0) break;
    before_message = messages.last()!.id;
    for (const [_, message] of messages) {
      if (message.author.bot) continue;
      if (!/[.!?]/g.test(message.content)) {
        sentence = ` ${message.content}${sentence}`
      } else {
        return sentence;
      }
    }
  }
  return null;
}
