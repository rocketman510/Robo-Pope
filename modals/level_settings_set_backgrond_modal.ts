import { ModalSubmitInteraction, MessageFlags, AttachmentBuilder } from "discord.js";
import type { Modal } from "../deploy";
import { ensure } from "..";
import fs from 'fs';
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
          await interaction.deferUpdate()
        } else if (values.includes('image')) {
          const image = interaction.fields.getUploadedFiles('level_settings_set_backgrond_modal_image');

          if (!image || image.first()!.contentType != 'image/png') {
            interaction.reply({ content: 'You must upload a PNG', flags: MessageFlags.Ephemeral });
            return;
          }

          await interaction.deferUpdate()

          const url = image?.first()!.url;

          const parsedUrl = new URL(url).pathname.split("/").pop();

          const fileUrl = process.env.CACHE_PATH! + parsedUrl!

          const res = await fetch(url);
          if (!res.ok) throw new Error(`Failed: ${res.statusText}`);

          const buffer = Buffer.from(await res.arrayBuffer());
          fs.writeFileSync(process.env.CACHE_PATH! + parsedUrl!, buffer);

          const cache_channel = ensure(await interaction.client.channels.fetch(process.env.CACHE_CHANNEL_ID!));

          const attachment = new AttachmentBuilder(fileUrl);

          const message = await cache_channel.send({ files: [attachment]})

          console.log(message);

          level_settings.has_costome_background = true;
          level_settings.backgrond_url = message.attachments.first()!.url;
        }

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


