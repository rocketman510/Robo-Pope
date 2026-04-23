import { SlashCommandBuilder, ChatInputCommandInteraction, PermissionFlagsBits, MessageFlags } from "discord.js";
import type { Command } from "../deploy";

async function fail(interaction: ChatInputCommandInteraction, message: string) {
  console.error(message);
  await interaction.reply({content: message, flags: MessageFlags.Ephemeral});
}

export default {
  data: new SlashCommandBuilder()
      .setName('admonish')
      .setDescription('Kick a user')
      .addUserOption(
        option => option
        .setName('user')
        .setDescription('The user to Kick')
        .setRequired(true)
      )
      .addStringOption(
        option => option
        .setName('reason')
        .setDescription('The reason the user was kicked')
        .setRequired(false)
      )
      .setDefaultMemberPermissions(PermissionFlagsBits.KickMembers),
  async execute(interaction: ChatInputCommandInteraction) {
    const user = interaction.options.getUser('user');
    if (!user) return fail(interaction, "Failed to get the user");
    const guild = interaction.guild;
    if (!guild) return fail(interaction, "Failed to get guild");
    const member = guild.members.cache.get(user.id);
    if (!member) return fail(interaction, "Failed to get the member");
    const canKick = !member.permissions.has('Administrator');
    if (!canKick) return fail(interaction, "You can not Kick this User");
    const hasKickPerms = guild.members.me?.permissions.has(PermissionFlagsBits.KickMembers);
    if (!hasKickPerms) return fail(interaction, "Does not have kick perms");

    await user.send(`You have been Admonish from ${interaction.guild?.name} for the reason: ${interaction.options.getString('reason') ?? 'no reason'}. You are free to join back after substantial consideration.`)

    await guild.members.kick(user, interaction.options.getString('reason') ?? undefined);
    await interaction.reply({content: `${user?.displayName} has been Admonish`});
  },
} as Command;
