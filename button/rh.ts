import { ButtonInteraction, MessageFlags } from "discord.js";
import type { Button } from "../deploy";
import { decode3BitPacked, render_highlighting } from "../functions/render_page";

export type HighlighterSetting = {
  _id: string,
  book_id: string,
  user_id: string,
  color: number,
}

export default {
  data: "rh",
  async execute(interaction: ButtonInteraction) {
    // const match = interaction.customId.match(/^\w.(?:-+)([\w]*)(?:-+)([\w]*)(?:-+)?([A-Za-z0-9+/=]*)?/)
    const match = interaction.customId.match(/^\w.(?:-+)([\w]*)(?:-+)([\w]*)(?:-+)?([\w]*)?/)

    if (!match || !match[1] || !match[2]) return;

    const collection = interaction.client.db.collection<HighlighterSetting>("highlighter_settings")

    if (!match[3] || match[3] != "000") {
      const color = interaction.client.highlight_color.get(interaction.user.id) ?? 1;

      await collection.findOneAndUpdate(
        {
          _id: match[2].slice(0, 6) + match[3],
          book_id: match[1],
          user_id: interaction.user.id,
        },
        [
          {
            $set: {
              color: {
                $cond: [
                  { $eq: ["$color", color] },
                  0,
                  color,
                ],
              },
            },
          },
        ],
        { upsert: true }
      );
    } 

    const container = await render_highlighting(match[1], match[2], interaction.client.db.collection("book_primitives"),  interaction, collection)

    await interaction.update({ components: container, flags: MessageFlags.IsComponentsV2 })
  },
} as Button;
