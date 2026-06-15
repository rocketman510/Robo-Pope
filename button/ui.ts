import { ButtonInteraction } from "discord.js";
import type { Button } from "../deploy";

export default {
  data: "ui",
  async execute(interaction: ButtonInteraction) {
    const client = interaction.client;
    const match = interaction.customId.match(/^ui-(.+?)-([A-Za-z0-9_-]+)$/);
    
    if (!match) return;
    const custom_id = match[1];
    if (!custom_id) return;
    const hash = match[2];
    if (!hash) return;
    const page = client.pages.get(custom_id);
    if (!page) return;
    const func = page.cache.get(hash);
    if (!func) return;
    const data = page.data.get(hash);

    await func(page, interaction, data);
  },
} as Button;
