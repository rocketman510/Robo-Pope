import { ButtonInteraction, MessageFlags } from "discord.js";
import type { Button } from "../deploy";
import { render_share } from "../functions/render_page";

export default {
    data: "rs",
    async execute(interaction: ButtonInteraction) {
      const match = interaction.customId.match(/^\w.(?:-+)([\w]*)(?:-+)([\w]*)$/)
      
      if (!match || !match[1] || !match[2]) return;
    },
} as Button;
