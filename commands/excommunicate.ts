import { SlashCommandBuilder, ChatInputCommandInteraction, PermissionFlagsBits, MessageFlags } from "discord.js";
import type { Command } from "../deploy";

async function fail(interaction: ChatInputCommandInteraction, message: string) {
  console.error(message);
  await interaction.reply({content: message, flags: MessageFlags.Ephemeral});
}

export default {
  data: new SlashCommandBuilder()
      .setName('excommunicate')
      .setDescription('Excommunicate a user')
      .addUserOption(
        option => option
        .setName('user')
        .setDescription('The user to Excommunicate')
        .setRequired(true)
      )
      .setDefaultMemberPermissions(PermissionFlagsBits.BanMembers),
  async execute(interaction: ChatInputCommandInteraction) {
    const user = interaction.options.getUser('user');
    if (!user) return fail(interaction, "Failed to get the user");
    const guild = interaction.guild;
    if (!guild) return fail(interaction, "Failed to get guild");
    const member = guild.members.cache.get(user.id);
    if (!member) return fail(interaction, "Failed to get the member");
    const canBan = !member.permissions.has('Administrator');
    if (!canBan) return fail(interaction, "Failed to get users permissions");
    const hasBanPerms = guild.members.me?.permissions.has('BanMembers');
    if (!hasBanPerms) return fail(interaction, "Does not have ban perms");

    console.log(canBan);
    console.log("Ban Perms:" + hasBanPerms);

    await user.send(`You have been Excommunicated from ${interaction.guild?.name}`)

    guild.members.ban(user);
    await interaction.reply({content: `${user?.displayName} has been Excommunicated`});
  },
} as Command;
