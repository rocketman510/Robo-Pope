import { SlashCommandBuilder, ChatInputCommandInteraction, PermissionFlagsBits, MessageFlags } from "discord.js";
import type { Command } from "../deploy";

async function fail(interaction: ChatInputCommandInteraction, message: string) {
  console.error(message);
  await interaction.reply({content: message, flags: MessageFlags.Ephemeral});
}

export default {
  data: new SlashCommandBuilder()
    .setName('vowofsilence')
    .setDescription('Mute someone')
    .addUserOption(
      option => option
      .setName('user')
      .setDescription('The user to Silence')
      .setRequired(true)
    )
    .addStringOption(
      option => option
      .setName('time')
      .setDescription('Set the time by its unit. ? for help')
      .setRequired(true)
    )
    .addStringOption(
      option => option
      .setName('reason')
      .setDescription('The reason the user was silenced')
      .setRequired(false)
    )
    .setDefaultMemberPermissions(PermissionFlagsBits.ModerateMembers),
  async execute(interaction: ChatInputCommandInteraction) {
    const user = interaction.options.getUser('user');
    if (!user) {fail(interaction, "Failed to get the user"); return;}
    const guild = interaction.guild;
    if (!guild) {fail(interaction, "Failed to get guild"); return;}
    const member = guild.members.cache.get(user.id);
    if (!member) {fail(interaction, "Failed to get the member"); return;}
    const time_string = interaction.options.getString('time')
    if (!time_string) fail(interaction, "Invalid Time string")
    const hours = time_string?.match(/(\d+)\s*(?=hours?|h)|(?:hours?|h):\s*(\d+)/gi);

    console.log(hours);
    
  },
} as Command;
