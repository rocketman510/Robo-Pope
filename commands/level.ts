import { SlashCommandBuilder, ChatInputCommandInteraction, MessageFlags } from "discord.js";
import type { Command } from "../deploy";
import { getLevel } from "../level";

export default {
    data: new SlashCommandBuilder()
        .setName('level')
        .setDescription('Get your level'),
    async execute(interaction: ChatInputCommandInteraction) {
      const client = interaction.client
      if (client.is_counting_messages) {fail(interaction, "Come back later, I'm still tallying up the score."); return;}

      const messages = client.messages.get(interaction.guildId!);
      if (!messages) {fail(interaction, "Could not get Server's data");return;}
      const words = messages?.get(interaction.user.id);
      if (!words) {fail(interaction, "Could not get your data");return;}

      const level = getLevel(words);
      if (interaction.user.id == '715043871503024218') {
        interaction.reply({ content: `You have spocken ${words} words in this server. level is ${level.level} minxp: ${level.min_xp} and maxxp is ${level.max_xp}`, flags: MessageFlags.Ephemeral})
      } else {
        interaction.reply({ content: `You have spocken ${words} words in this server.`, flags: MessageFlags.Ephemeral})
      }
    },
} as Command;

function fail(interaction: ChatInputCommandInteraction, message: string) {
  interaction.reply({ content: message, flags: MessageFlags.Ephemeral });
}
