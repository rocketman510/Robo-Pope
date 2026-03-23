import { SlashCommandBuilder, ChatInputCommandInteraction, MessageFlags, AttachmentBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle, Emoji } from "discord.js";
import type { Command } from "../deploy";
import fs from "fs";
import { getLevelBanner, getLevelBannerSettings, type LevelSettings } from "../level";

export default {
    data: new SlashCommandBuilder()
        .setName('level')
        .setDescription('Get your level'),
    async execute(interaction: ChatInputCommandInteraction) {
      await interaction.deferReply({ flags: MessageFlags.Ephemeral });

      const client = interaction.client
      if (client.is_counting_messages) {fail(interaction, "Come back later, I'm still tallying up the score."); return;}

      const messages = client.messages.get(interaction.guildId!);
      if (!messages) {fail(interaction, "Could not get Server's data");return;}
      const words = messages?.get(interaction.user.id);
      if (!words) {fail(interaction, "Could not get your data");return;}
      const level_setting: LevelSettings = await getLevelBannerSettings(client, interaction.user.id, interaction.guildId!);

      let imagePath = await getLevelBanner(interaction.user, level_setting);
      if (!imagePath) {fail(interaction, "Somthing happend I cant find you data!?"); return;}

      const attachment = new AttachmentBuilder(imagePath);

      const button_share = new ButtonBuilder()
        .setCustomId('level_share')
        .setLabel('Share')
        .setStyle(ButtonStyle.Primary)

      const button_settings = new ButtonBuilder()
        .setCustomId('level_settings')
        .setEmoji('⚙️')
        .setStyle(ButtonStyle.Secondary)

      const action_row = new ActionRowBuilder<ButtonBuilder>()
        .addComponents(button_share)
        .addComponents(button_settings);

      await interaction.editReply({ components: [action_row], files: [attachment] });

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
