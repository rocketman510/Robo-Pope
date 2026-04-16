import { ChannelType, type VoiceState } from "discord.js";
import { ensure } from "..";

export async function handle_join(oldState: VoiceState, newState: VoiceState) {
  const dyn_id = JSON.parse(ensure(process.env.DYNAMIC_VOICE_CHANNELS, "No DYNAMIC_VOICE_CHANNELS ENV"));
  if (dyn_id.includes(newState.channelId)) {
    const new_channle = await newState.guild.channels.create({name: `VC:`, type: ChannelType.GuildVoice})
    newState.member?.voice.setChannel(new_channle)
  }
}
