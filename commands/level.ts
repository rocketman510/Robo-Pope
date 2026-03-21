import { SlashCommandBuilder, ChatInputCommandInteraction, MessageFlags, AttachmentBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle, Emoji } from "discord.js";
import type { Command } from "../deploy";
import fs from "fs";
import { getLevel, getLevelBanner } from "../level";

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

      let imagePath = await getLevelBanner(interaction.user, interaction.guildId!);
      if (!imagePath) {fail(interaction, "Somthing happend I cant find you data!?"); return;}

      const attachment = new AttachmentBuilder(imagePath);

      const button_share = new ButtonBuilder()
        .setCustomId('level_share')
        .setLabel('Share')
        .setStyle(ButtonStyle.Primary)

      const button_settings = new ButtonBuilder()//TODO
        .setCustomId('level_settings')
        .setEmoji('⚙️')
        .setStyle(ButtonStyle.Secondary)
        .setDisabled(true);

      const action_row = new ActionRowBuilder()
        .addComponents(button_share)
        .addComponents(button_settings);

      await interaction.reply({ components: [action_row], files: [attachment], flags: MessageFlags.Ephemeral});

      if (fs.existsSync(imagePath)) {
        try {
          fs.unlinkSync(imagePath);
        } catch (err) {}
      }
    },
} as Command;

function fail(interaction: ChatInputCommandInteraction, message: string) {
  interaction.reply({ content: message, flags: MessageFlags.Ephemeral });
}
