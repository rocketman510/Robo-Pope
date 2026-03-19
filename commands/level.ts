import { SlashCommandBuilder, ChatInputCommandInteraction, MessageFlags, AttachmentBuilder } from "discord.js";
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

      const level = getLevel(words);

      let imagePath = await getLevelBanner(interaction.user, interaction.guildId);
      if (!imagePath) {fail("Somthing happend I cant find you data!?"); return;}

      const attachment = new AttachmentBuilder(imagePath);

      await interaction.reply({ files: [attachment], flags: MessageFlags.Ephemeral});

      console.log(fs.existsSync(imagePath));

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
