import { SlashCommandBuilder, ChatInputCommandInteraction, PermissionFlagsBits, MessageFlags, messageLink } from "discord.js";
import type { Command } from "../deploy";
import { sleep } from "bun";

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
      if (!user) {fail(interaction, "Failed to get the user"); return;}
      const guild = interaction.guild;
      if (!guild) {fail(interaction, "Failed to get guild"); return;}
      const member = guild.members.cache.get(user.id);
      if (!member) {fail(interaction, "Failed to get the member"); return;}
      const canBan = !member.permissions.has('Administrator');
      if (!canBan) {fail(interaction, "Failed to get users permissions"); return;}
      const hasBanPerms = guild.members.me.permissions.has('BanMembers');
      if (!hasBanPerms) {fail(interaction, "Dose Not have ban Perms"); return;}


      if (canBan) {
        console.log(canBan);
        console.log("Ban Perms:" + hasBanPerms);
        
        await user.send(`You have been Excommunicated from ${interaction.guild?.name}`)
        
        guild.members.ban(user);
        await interaction.reply({content: `${user?.displayName} has been Excommunicated`});
      }
    },
} as Command;

