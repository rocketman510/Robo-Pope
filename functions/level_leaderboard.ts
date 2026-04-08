import { ContainerBuilder, ContainerComponent, Message, MessageFlags, TextDisplayComponent, type Client, type GuildTextBasedChannel } from "discord.js";
import { getLevel } from "../level";
import { ensure } from "..";

export async function generateLeaderboard(client: Client, guild_id: string, old_message?: Message) {
  const c = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addTextDisplayComponents(t => t.setContent("# Leaderboard"));

  const sorted = [...client.messages.get(guild_id)!.entries()].sort((a, b) => b[1] - a[1]).slice(0, 10);
  const old = old_message
    ? (old_message.components[0] as ContainerComponent).components
        .slice(1)
        .map(x => (x as TextDisplayComponent).data.content.match(/<@(\d+)>/)![1])
    : [];

  for (let i = 0; i < sorted.length; i++) {
    const [user_id, xp] = sorted[i];
    const { level } = getLevel(xp);
    const user = await client.users.fetch(user_id);

    const old_i = old.indexOf(user_id);
    const emoji = !old_message || old_i === -1 || old_i === i
      ? "<:bar:1486853453094453439>"
      : i > old_i
        ? "<:bar_down:1486850346054582272>"
        : "<:bar_up:1486850337146146907>";

    c.addTextDisplayComponents(t =>
      t.setContent(`${emoji}  **${i + 1}. <@${user.id}> - ${level} Lvl. - ${xp} xp**`)
    );
  }

  return [c];
}

export async function update_leaderboard(client: Client, channel_id: string) {
  const channel = await client.channels.fetch(channel_id) as GuildTextBasedChannel;
  if (!channel || !channel.isTextBased()) throw `Not a valid channel at ID: ${channel_id}`;
  const message = await channel.messages.fetch({ limit: 1 })
  if (message.size != 1) {
    const components = await generateLeaderboard(client, channel.guild.id)
    channel.send({ components, flags: MessageFlags.IsComponentsV2, });
  } else if (message.first()?.author.id == ensure(process.env.CLIENT_ID, "No CLIENT_ID environment variable")) {
    const components = await generateLeaderboard(client, channel.guild.id, message.first())
    await message.first()?.edit({ components, flags: MessageFlags.IsComponentsV2});
  }
}
