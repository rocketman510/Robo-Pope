import { SlashCommandBuilder, ChatInputCommandInteraction, MessageFlags } from "discord.js";
import type { Command } from "../deploy";

export default {
  data: new SlashCommandBuilder()
      .setName('stopspam')
      .setDescription('Stop Spamming'),
  async execute(interaction: ChatInputCommandInteraction) {
      if ('rocketman510 jorbiathan'.includes(interaction.user.username)) {
        interaction.client.shouldStopSpam = true;
        interaction.reply({content: "Stopped Spamming", flags: MessageFlags.Ephemeral})
        console.log(interaction.client.shouldStopSpam);
      } else {
        interaction.reply({content: "you have no perms to run this D:", flags: MessageFlags.Ephemeral});
      }
  },
} as Command;
