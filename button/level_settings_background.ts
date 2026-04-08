import { ButtonInteraction, FileUploadBuilder, LabelBuilder, MessageFlags, ModalBuilder, StringSelectMenuBuilder, StringSelectMenuOptionBuilder, TextInputBuilder, TextInputStyle, } from "discord.js";
import type { Button } from "../deploy";
import { getLevelBannerSettings, hexNumToStr } from "../level";
import { ensure } from "..";

export default {
  data: "level_settings_background",
  async execute(interaction: ButtonInteraction) {
    const level_settings = await getLevelBannerSettings(
      interaction.client,
      interaction.user.id,
      ensure(interaction.message.guildId)
    );

    const current_color = hexNumToStr(level_settings.primary_color & 0xffffff, 0).slice(0,7);
    const current_trans = level_settings.primary_color_trans.toFixed(1).toString().slice(0,5);

    const modal = new ModalBuilder()
      .setCustomId('level_settings_set_background_modal')
      .setTitle("Background Image");

    const background_style = new StringSelectMenuBuilder()
      .setCustomId('level_settings_set_background_modal_stlyle')
      .setRequired(true)
      .setPlaceholder('Select Background Style')
      .addOptions(
        new StringSelectMenuOptionBuilder()
          .setLabel('Transparent')
          .setValue('transparent'),
        new StringSelectMenuOptionBuilder()
          .setLabel('Image')
          .setValue('image')
      );

    const background_image = new FileUploadBuilder()
      .setCustomId('level_settings_set_background_modal_image')
      .setRequired(false)
      .setMaxValues(1);

    const background_image_label = new LabelBuilder()
      .setLabel('Background Image')
      .setFileUploadComponent(background_image);


    const background_style_label = new LabelBuilder()
      .setLabel('Set Background Style:')
      .setStringSelectMenuComponent(background_style);

    modal
      .addLabelComponents(background_style_label)
      .addLabelComponents(background_image_label);

    interaction.showModal(modal);
},
} as Button;
