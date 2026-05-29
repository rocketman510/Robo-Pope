import { ActionRow, MessageFlags, StringSelectMenuBuilder } from "discord.js";
import type { SelectionMenu } from "../deploy";

export default {
  data: "colorpicker",
  async execute(interaction) {

    const selected = interaction.values[0];

    if (!selected) return;

    interaction.message.components[0]!.components
      .find((c): c => c.type === 1)
      .components[0]
      .data
      .options = interaction.message.components[0]!.components
      .find((c): c => c.type === 1)
      .components[0]
      .data
      .options
      .map(opt => ({
        ...opt,
        default: opt.value === selected,
      }))// This is dumb but it works

    const values: Record<string, number> = {
      "yellow": 1,
      "red": 2,
      "purple": 3,
      "green": 4,
      "blue": 5
    }

    interaction.client.highlight_color.set(interaction.user.id, values[selected] ?? 1)

    await interaction.update({components: interaction.message.components, flags: MessageFlags.IsComponentsV2})
  },
} as SelectionMenu;
