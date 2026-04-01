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

    const curent_color = hexNumToStr(level_settings.primary_color & 0xffffff, 0).slice(0,7);
    const curent_trans = level_settings.primary_color_trans.toFixed(1).toString().slice(0,5);

    const modal = new ModalBuilder()
      .setCustomId('level_settings_set_backgrond_modal')
      .setTitle("Backgrond Image");

    const backgrond_style = new StringSelectMenuBuilder()
      .setCustomId('level_settings_set_backgrond_modal_stlyle')
      .setRequired(true)
      .setPlaceholder('Select Backgrond Style')
      .addOptions(
        new StringSelectMenuOptionBuilder()
          .setLabel('Transparent')
          .setValue('transparent'),
        new StringSelectMenuOptionBuilder()
          .setLabel('Image')
          .setValue('image')
      );

    const backgrond_image = new FileUploadBuilder()
      .setCustomId('level_settings_set_backgrond_modal_image')
      .setRequired(false)
      .setMaxValues(1);

    const backgrond_image_lable = new LabelBuilder()
      .setLabel('Backgrond Image')
      .setFileUploadComponent(backgrond_image);
      
    
    const backgrond_style_lable = new LabelBuilder()
      .setLabel('Set Backgrond Style:')
      .setStringSelectMenuComponent(backgrond_style);

    modal
      .addLabelComponents(backgrond_style_lable)
      .addLabelComponents(backgrond_image_lable);

    interaction.showModal(modal);
},
} as Button;

