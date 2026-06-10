import { ButtonInteraction, MessageFlags } from "discord.js";
import type { Button } from "../deploy";
import { decode } from "./rs";
import type { BookPrimitive } from "../commands/read";
import { render_primitives } from "../functions/render_page";

export default {
  data: "rv",
  async execute(interaction: ButtonInteraction) {
    const match = interaction.customId.match(/^\w.(?:-+)([\w]*)(?:-+)([\w]*)(?:-+)?([A-Za-z0-9+/=]*)?/)
    
    if (!match || !match[1] || !match[2] || !match[3]) return;
    
    const primitive_ids = decode(match[3]);
    let buffer = []

    for (const id of primitive_ids) {
      const primitive = await interaction.client.db.collection<BookPrimitive>("book_primitives").findOne({_id: match[2] + id, book_id: match[1]})
      if (primitive === null) continue;
      buffer.push(primitive);
    }

    const container = await render_primitives(buffer);
    interaction.reply({components: container, flags: [MessageFlags.IsComponentsV2, MessageFlags.Ephemeral]})
  },
} as Button;
