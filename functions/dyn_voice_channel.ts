import { ChannelType, Client, Guild, GuildMember, OverwriteResolvable, PermissionFlagsBits, User, VoiceChannel, type VoiceState } from "discord.js";
import { ensure } from "..";
import { get_result, type VcSettings } from "../commands/vc";



export class DynamicVC {
  public settings: VcSettings;
  public channel!: VoiceChannel;
  public made: Date;
  public async slef_delete() {
    const diff = 60000 - (new Date().getTime() - this.made.getTime());
    if (this.channel.members.size !== 0) return;
    if (diff < 0) {this.remove(); return;}
    await new Promise(r => setTimeout(r, diff));
    if (this.channel.members.size !== 0) return;
    this.remove();
  }
  public async construct(): Promise<VoiceChannel | null> {
    const client = this.guild.client;
    const candidates = JSON.parse(process.env.DYNAMIC_VOICE_CHANNELS ?? "[]").filter(async (id: string)=> (await this.guild.channels.fetch()).has(id))
    if (!candidates?.length) return null;
    const parent_vc = await client.channels.fetch(candidates[0])
    if (!parent_vc?.isVoiceBased()) return null;

    const maker_perms = {
      id: this.owner.user.id,
      allow: [
        PermissionFlagsBits.ViewChannel,
        PermissionFlagsBits.Connect,
        PermissionFlagsBits.Speak,
        PermissionFlagsBits.MuteMembers,
        PermissionFlagsBits.DeafenMembers,
        PermissionFlagsBits.MoveMembers,
        PermissionFlagsBits.ManageChannels,
        PermissionFlagsBits.ManageRoles
      ],
    }

    let permissions: OverwriteResolvable[] = [maker_perms];

    for (const e of [...this.settings.permitted.users_id, ...this.settings.permitted.roles_id]) {
      permissions.push({
        id: e,
        allow: [
          PermissionFlagsBits.Connect
        ],
      })
    }

    if (this.settings.private) {
      permissions.push({
        id: this.guild.roles.everyone.id,
        deny: [
          PermissionFlagsBits.Connect,
          // PermissionFlagsBits.ViewChannel
        ]
      })
    }

    const new_channel = await this.guild.channels.create({name: `${this.owner.displayName}'s VC`, type: ChannelType.GuildVoice, parent: parent_vc.parent, permissionOverwrites: permissions, userLimit: this.settings.limit});
    this.channel = new_channel;
    this.guild.client.dyn_vc.set(this.channel.id, this);
    this.slef_delete();
    return new_channel;
  }
  public async remove() {
    try {
      await this.channel.delete();
      this.guild.client.dyn_vc.delete(this.channel.id);
    } catch (errr) {}
  }
  constructor(private guild: Guild, public owner: GuildMember, settings?: VcSettings) {
    const default_document = {
      _id: this.owner.user.id,
      private: false,
      limit: 0,
      permitted: {
        users_id: [],
        roles_id: [],
      }
    };
    this.settings = settings ?? default_document;
    this.made = new Date()
  }
}

export async function handle_join(oldState: VoiceState, newState: VoiceState) {
  const client = newState.client;
  if (JSON.parse(process.env.DYNAMIC_VOICE_CHANNELS ?? "[]").includes(newState.channel?.id) && !!newState.member) {// Make new VC on join to make channle
    const vc = new DynamicVC(newState.guild, newState.member)
    await vc.construct();
    await vc.owner.voice.setChannel(vc.channel);
  }
  if (client.dyn_vc.has(oldState.channel?.id ?? "") && oldState.channel?.members.size === 0) {//Removes when empty
    client.dyn_vc.get(oldState.channel.id)?.remove()
  }
}

export async function make_vc(guild: Guild, make_vc_id: string, user_id: string): Promise<VoiceChannel | null> {
  const member = await guild.members.fetch(user_id);
  if (!member) return null;
  const vc = new DynamicVC(guild, member, await get_result(guild.client, user_id))
  return await vc.construct();
}
