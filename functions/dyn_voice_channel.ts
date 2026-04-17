import { ChannelType, Client, type VoiceState } from "discord.js";
import { ensure } from "..";

export async function handle_join(oldState: VoiceState, newState: VoiceState) {
  const dyn_id = JSON.parse(ensure(process.env.DYNAMIC_VOICE_CHANNELS, "No DYNAMIC_VOICE_CHANNELS ENV"));
  const client = newState.client;

  let channels = client.dyn_vc.ensure(newState.guild.id, () => []);

  if (oldState.channelId === newState.channelId) return;

  if (oldState.channelId && channels.includes(oldState.channelId) && oldState.channel?.members.size == 0) {// Removes the channel if all users leave
    await oldState.channel.delete();
    remove_from_array(oldState.guild.id, oldState.channelId, client);
  }

  channels = client.dyn_vc.ensure(oldState.guild.id, () => []);

  if (newState.channelId && dyn_id.includes(newState.channelId)) {// Make's new Channle and moves the user to it
    const new_channel = await newState.guild.channels.create({name: `VC: ${channels.length + 1}`, type: ChannelType.GuildVoice, parent: newState.channel?.parent});
    newState.member?.voice.setChannel(new_channel);

    add_to_array(newState.guild.id, new_channel.id, client)
  }
}

function add_to_array(guild_id:string, channel_id: string, client: Client) {
  let channels = client.dyn_vc.get(guild_id);
  channels!.push(channel_id)
  client.dyn_vc.set(guild_id, channels!)
}

function remove_from_array(guild_id:string, channel_id: string, client: Client) {
  let channels = client.dyn_vc.get(guild_id);
  channels = channels?.filter(e => e != channel_id);
  client.dyn_vc.set(guild_id, channels!)
}
