import { MessageFlags } from "discord.js";
import { ensure } from "..";
import type { SelectionMenu } from "../deploy";
import { generateComponents } from "../functions/level_settings";
import { getLevelBannerSettings, setLevelBannerSettings } from "../level";

export default {
  data: "level_settings_size",
  async execute(interaction) {
    try {
      const client = interaction.client
      await interaction.deferUpdate();
      let level_settings = await getLevelBannerSettings(client, interaction.user.id, ensure(interaction.guildId));

      level_settings.is_large = interaction.values![0] == 'large'

      await setLevelBannerSettings(client, level_settings);

      const components = await generateComponents(level_settings, client);

      await interaction.editReply({ components, flags: MessageFlags.IsComponentsV2});
    } catch (_) {}
  },
} as SelectionMenu;
