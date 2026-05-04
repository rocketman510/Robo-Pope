import { ButtonInteraction, MessageFlags } from "discord.js";
import type { Button } from "../deploy";
import { render_share } from "../functions/render_page";

export default {
    data: "rs",
    async execute(interaction: ButtonInteraction) {
      const match = interaction.customId.match(/^\w.(?:-+)([\w]*)(?:-+)([\w]*)$/)
      
      if (!match || !match[1] || !match[2]) return;

      const container = await render_share(match[1], match[2], 3000, interaction.client.db.collection("book_primitives"), interaction.client.db.collection("books"))

      await interaction.update({ components: container, flags: MessageFlags.IsComponentsV2 })
    },
} as Button;
