import { ButtonInteraction, ContainerComponent, MessageFlags } from "discord.js";
import type { Button } from "../deploy";
import { render_page } from "../functions/render_page";

export default {
    data: "rn",
    async execute(interaction: ButtonInteraction) {
      console.log(interaction.customId);

      const match = interaction.customId.match(/^\w.(?:-)([\w]*)(?:-)([\w]*)$/)
      
      if (!match || !match[1] || !match[2]) return;

      const container = await render_page(match[1], match[2], 1500, interaction.client.db.collection("book_primitives"));

      await interaction.update({ components: container, flags: MessageFlags.IsComponentsV2 })
    },
} as Button;
