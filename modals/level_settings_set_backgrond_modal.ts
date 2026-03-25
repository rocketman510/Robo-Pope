import { ModalSubmitInteraction, MessageFlags } from "discord.js";
import type { Modal } from "../deploy";
import { ensure } from "..";
import { generateComponents } from "../functions/level_settings";
import { getLevelBannerSettings, setLevelBannerSettings } from "../level";

export default {
    data: "level_settings_set_backgrond_modal",
    async execute(interaction: ModalSubmitInteraction) {
      try {
        const values = interaction.fields.getStringSelectValues('level_settings_set_backgrond_modal_stlyle');
        let level_settings = await getLevelBannerSettings(interaction.client, interaction.user.id, ensure(interaction.guildId));

        if (values.includes('transparent')) {
          level_settings.has_costome_background = false;
        } else if (values.includes('image')) {
          const image = interaction.fields.getUploadedFiles('level_settings_set_backgrond_modal_image')

          if (!image || image.first()!.contentType != 'image/png') {
            interaction.reply({ content: 'You must upload a Image', flags: MessageFlags.Ephemeral });
            return;
          }

          level_settings.has_costome_background = true;
          level_settings.backgrond_url = image.first()!.url;
        }
        await interaction.deferUpdate()

        await setLevelBannerSettings(interaction.client, level_settings);
        const components = await generateComponents(level_settings, interaction.client);

        interaction.editReply({
          components,
          flags: [
            MessageFlags.IsComponentsV2,
          ]
        })
      } catch (err) {
        console.error(err)
      }
    },
} as Modal;


