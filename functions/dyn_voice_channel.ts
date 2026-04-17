import { ChannelType, type VoiceState } from "discord.js";
import { ensure } from "..";

export async function handle_join(oldState: VoiceState, newState: VoiceState) {
  const dyn_id = JSON.parse(ensure(process.env.DYNAMIC_VOICE_CHANNELS, "No DYNAMIC_VOICE_CHANNELS ENV"));
  const client = newState.client;

  let channels = client.dyn_vc.ensure(newState.guild.id, () => []);

  if (oldState.channelId && channels.includes(oldState.channelId)) {
    if (oldState.channel?.members.size == 0) {
      await oldState.channel.delete()
      channels = client.dyn_vc.get(oldState.guild.id)!;
      console.log(channels, oldState.channelId);
      channels = channels.filter(e => e != oldState.channelId);
      console.log(channels);
      client.dyn_vc.set(oldState.guild.id, channels)
    }
  }

  channels = client.dyn_vc.get(oldState.guild.id)!;

  if (newState.channelId && dyn_id.includes(newState.channelId)) {
    const new_channel = await newState.guild.channels.create({name: `VC: ${channels.length + 1}`, type: ChannelType.GuildVoice, parent: newState.channel?.parent});
    newState.member?.voice.setChannel(new_channel);

    channels = client.dyn_vc.get(oldState.guild.id)!;
    channels.push(new_channel.id);
    client.dyn_vc.set(newState.guild.id, channels)
  }
}
