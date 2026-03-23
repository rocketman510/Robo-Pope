import { ModalSubmitInteraction, MessageFlags } from "discord.js";
import type { Modal } from "../deploy";
import { ensure } from "..";
import { generateComponents } from "../functions/level_settings";
import { getLevelBannerSettings, setLevelBannerSettings } from "../level";

export default {
    data: "level_settings_set_primery_color_modal",
    async execute(interaction: ModalSubmitInteraction) {
      try {
        const color = interaction.fields.getTextInputValue('level_settings_set_primery_color_modal_color');
        const trans = interaction.fields.getTextInputValue('level_settings_set_primery_color_modal_trans');

        const color_match = /^#([0-f]{6})$/.exec(color);
        const trans_match = /^[0-9]+(\.[0-9]+)?$/.test(trans);

        if (color_match!.length < 1) {
          interaction.reply({ content: "Improperly formatted color. Try again", flags: [ MessageFlags.Ephemeral ] });
          return
        }

        if (!trans_match) {
          interaction.reply({ content: "Improperly formatted transparency value. Try again", flags: [ MessageFlags.Ephemeral ] });
          return
        }

        const finle_color = Number("0x" + color_match![1]);
        const finle_trans = Math.min(1 ,Number(trans));

        await interaction.deferUpdate()

        console.log(finle_color, finle_trans);
        

        let level_settings = await getLevelBannerSettings(
          interaction.client,
          interaction.user.id,
          ensure(interaction.guildId)
        );

        level_settings.primary_color = finle_color;
        level_settings.primary_color_trans = finle_trans;

        await setLevelBannerSettings(interaction.client, level_settings);

        const components = await generateComponents(level_settings, interaction.client)
        
        await interaction.editReply({
          components,
          flags: [
            MessageFlags.IsComponentsV2,
          ]
        });
      } catch (err) {
        console.error(err)
      }
    },
} as Modal;
