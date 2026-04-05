import type { Button } from "../deploy";
import { ButtonInteraction, EmbedBuilder, MessageFlags } from "discord.js";

export default {
  data: 'ows_show_full_story',
  async execute(interaction: ButtonInteraction) {
    const client = interaction.client;
    console.log(client.ows_sentence_history.get(interaction.channelId));
    const story = client.ows_sentence_history.ensure(interaction.channelId, (): string[] => []).join("").slice(-4096);

    const embed = new EmbedBuilder()
      .setTitle('Full story')
      .setDescription(story);

    interaction.reply({ embeds: [ embed ], flags: MessageFlags.Ephemeral})
  }
} as Button
