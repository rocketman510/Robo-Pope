import { SlashCommandBuilder, ChatInputCommandInteraction, MessageFlags, AttachmentBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle } from "discord.js";
import type { Command } from "../deploy";
import fs from "fs";
import { getLevelBanner, getLevelBannerSettings, type LevelSettings } from "../level";

export default {
    data: new SlashCommandBuilder()
        .setName('level')
        .setDescription('Get your level')
        .addUserOption(
          option => option
          .setName('user')
          .setDescription('The User to get the level of')
          .setRequired(false)
        ),
    async execute(interaction: ChatInputCommandInteraction) {
      await interaction.deferReply({ flags: MessageFlags.Ephemeral });

      const selected_user = interaction.options.getUser('user') || interaction.user
      const is_users = interaction.user.id == selected_user.id;

      const client = interaction.client
      if (client.is_counting_messages) {fail(interaction, "Come back later, I'm still tallying up the score."); return;}

      const messages = client.messages.get(interaction.guildId!);
      if (!messages) return fail(interaction, "Could not get Server's data");
      const words = messages?.get(selected_user.id);
      if (!words) return fail(interaction, "Could not get User's data");
      const level_setting: LevelSettings = await getLevelBannerSettings(client, selected_user.id, interaction.guildId!);

      let imagePath = await getLevelBanner(selected_user, level_setting);
      if (!imagePath) {fail(interaction, "Somthing happened I can't find your data!?"); return;}

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

      if (is_users) {
        await interaction.editReply({ components: [action_row], files: [attachment] });
      } else {
        await interaction.editReply({ files: [attachment] });
      }

      if (fs.existsSync(imagePath)) {
        fs.unlinkSync(imagePath);
      }
    },
} as Command;

function fail(interaction: ChatInputCommandInteraction, message: string) {
  interaction.editReply({ content: message });
}
