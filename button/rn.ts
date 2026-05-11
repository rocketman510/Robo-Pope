import { ButtonInteraction, MessageFlags } from "discord.js";
import type { Button } from "../deploy";
import { render_page } from "../functions/render_page";

export default {
    data: "rn",
    async execute(interaction: ButtonInteraction) {
      const match = interaction.customId.match(/^\w.(?:-+)([\w]*)(?:-+)([\w]*)$/)
      
      if (!match || !match[1] || !match[2]) return;

      const container = await render_page(match[1], match[2], 3000, interaction.client.db.collection("book_primitives"), interaction.client.db.collection("books"));

      if (interaction.message.flags.has(MessageFlags.Ephemeral)) {
        await interaction.update({ components: container, flags: MessageFlags.IsComponentsV2 })
      } else {
        await interaction.reply({ components: container, flags: [ MessageFlags.IsComponentsV2, MessageFlags.Ephemeral ] })
      }
    },
} as Button;
