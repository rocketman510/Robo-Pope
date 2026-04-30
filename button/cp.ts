import { ButtonInteraction, MessageFlags } from "discord.js";
import type { Button } from "../deploy";
import { render } from "../functions/chapter_picker";
import { type Book } from "../commands/read";

export default {
    data: "cp",
    async execute(interaction: ButtonInteraction) {
      const match = interaction.customId.match(/^\w.(?:-+)([\w]*)(?:-+)([\w]*)$/)
      
      if (!match || !match[1] || !match[2]) return;

      const collection = interaction.client.db.collection<Book>('books');

      const book = await collection.findOne({_id: match[1]});
      if (!book) return;

      const chapter = match[2].slice(0,3)

      const start = Number(match[2].slice(3,6))

      const container = render(book, chapter, start);

      await interaction.update({ components: container, flags: MessageFlags.IsComponentsV2 })
    },
} as Button;
