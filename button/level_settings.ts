import { ButtonInteraction, MessageFlags, } from "discord.js";
import type { Button } from "../deploy";
import { getLevelBannerSettings } from "../level";
import { ensure } from "..";
import { generateComponents } from "../functions/level_settings";

export default {
  data: "level_settings",
  async execute(interaction: ButtonInteraction) {
    try {
      interaction.deferReply({ flags: [MessageFlags.Ephemeral] });

      const level_settings = await getLevelBannerSettings(
        interaction.client,
        interaction.user.id,
        ensure(interaction.message.guildId)
      );

      const components = await generateComponents(level_settings, interaction.client)

      await interaction.editReply({
        components,
        flags: [
          MessageFlags.IsComponentsV2,
        ]
      });
    } catch (err) {
      console.error("Failed to resond to level_settings button interaction. Error:" + err);
    }
},
} as Button;
