import { ButtonInteraction } from "discord.js";
import type { Button } from "../deploy";

export default {
    data: "level_share",
    async execute(interaction: ButtonInteraction) {
      if (!interaction.channel?.isSendable()) {return};
      const attachment = interaction.message.attachments.first()!;
      const url = attachment.url;

      await interaction.deferUpdate();
      await interaction.channel.send({ files: [url] })//this only runs on a bot message that will have a file
    },
} as Button;
