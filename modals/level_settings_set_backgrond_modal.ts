import { ModalSubmitInteraction, MessageFlags, AttachmentBuilder } from "discord.js";
import type { Modal } from "../deploy";
import { ensure } from "..";
import { generateComponents } from "../functions/level_settings";
import { getLevelBannerSettings, setLevelBannerSettings } from "../level";
import sharp from "sharp";

export default {
  data: "level_settings_set_backgrond_modal",
  async execute(interaction: ModalSubmitInteraction) {
    const values = interaction.fields.getStringSelectValues('level_settings_set_backgrond_modal_stlyle');
    let level_settings = await getLevelBannerSettings(interaction.client, interaction.user.id, ensure(interaction.guildId));

    if (values.includes('transparent')) {
      level_settings.has_costome_background = false;
      await interaction.deferUpdate()
    } else if (values.includes('image')) {
      const image = interaction.fields.getUploadedFiles('level_settings_set_backgrond_modal_image');

      if (!image || image.first()!.contentType != 'image/png') {
        interaction.reply({ content: 'You must upload a PNG', flags: MessageFlags.Ephemeral });
        return;
      }

      await interaction.deferUpdate()

      const url = image?.first()!.url;

      const parsedUrl = await (async () => {
        try {
          const res = await fetch(url);
          if (!res.ok) return url;

          const buffer = Buffer.from(await res.arrayBuffer());
          const cropped = await sharp(buffer)
            .resize(512, 125, {
              fit: "cover",
              position: "center"
            })
            .toBuffer();
          const base64 = cropped.toString("base64");
          return `data:image/png;base64,${base64}`;
        } catch {
          return url;
        }
      })();

      level_settings.has_costome_background = true;
      level_settings.backgrond_url = parsedUrl;
    }

    await setLevelBannerSettings(interaction.client, level_settings);
    const components = await generateComponents(level_settings, interaction.client);

    interaction.editReply({
      components,
      flags: [
        MessageFlags.IsComponentsV2,
      ]
    })
  },
} as Modal;


