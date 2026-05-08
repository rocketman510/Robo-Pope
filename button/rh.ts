import { ButtonInteraction, MessageFlags } from "discord.js";
import type { Button } from "../deploy";
import { decode3BitPacked, render_highlighting } from "../functions/render_page";

export default {
    data: "rh",
    async execute(interaction: ButtonInteraction) {
      const match = interaction.customId.match(/^\w.(?:-+)([\w]*)(?:-+)([\w]*)(?:-+)?([A-Za-z0-9+/=]*)?/)

      if (!match || !match[1] || !match[2]) return;


      const container = await render_highlighting(match[1], match[2], interaction.client.db.collection("book_primitives"), decode3BitPacked(match[3] ?? ""), interaction)

      await interaction.update({ components: container, flags: MessageFlags.IsComponentsV2 })
    },
} as Button;
