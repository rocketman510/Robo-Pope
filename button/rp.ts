import { ButtonInteraction, MessageFlags } from "discord.js";
import type { Button } from "../deploy";
import { base64ToBoolArray, boolArrayToBase64, render_share } from "../functions/render_page";

export default {
    data: "rp",
    async execute(interaction: ButtonInteraction) {
      const match = interaction.customId.match(/^\w.(?:-+)([\w]*)(?:-+)([\w]*)(?:-+)?([A-Za-z0-9+/=]*)?/)

      if (!match || !match[1] || !match[2]) return;

      const settings = base64ToBoolArray(match[3] ?? boolArrayToBase64([false]));

      console.log(match, settings);
      

      const container = await render_share(match[1], match[2], 3000, interaction.client.db.collection("book_primitives"), settings)

      await interaction.update({ components: container, flags: MessageFlags.IsComponentsV2 })
    },
} as Button;
