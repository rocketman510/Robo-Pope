import { ButtonInteraction, LabelBuilder, MessageFlags, ModalBuilder, TextInputBuilder, TextInputStyle, } from "discord.js";
import type { Button } from "../deploy";
import { getLevelBannerSettings, hexNumToStr } from "../level";
import { ensure } from "..";

export default {
  data: "level_settings_set_primery_color",
  async execute(interaction: ButtonInteraction) {
    const level_settings = await getLevelBannerSettings(
      interaction.client,
      interaction.user.id,
      ensure(interaction.message.guildId)
    );

    const curent_color = hexNumToStr(level_settings.primary_color & 0xffffff, 0).slice(0,7);
    const curent_trans = level_settings.primary_color_trans.toFixed(1).toString().slice(0,5);

    const modal = new ModalBuilder()
      .setCustomId('level_settings_set_primery_color_modal')
      .setTitle("Primery Color");

    const color = new TextInputBuilder()
      .setCustomId('level_settings_set_primery_color_modal_color')
      .setStyle(TextInputStyle.Short)
      .setPlaceholder('#RRGGBB')
      .setValue(curent_color)
      .setRequired(true)
      .setMaxLength(7)
      .setMinLength(7);

    const color_label = new LabelBuilder()
      .setLabel("Set The Color")
      .setDescription("Set the color as a Hexadecimal color. Formated formatted as #RRGGBB")
      .setTextInputComponent(color);

    const trans = new TextInputBuilder()
      .setCustomId('level_settings_set_primery_color_modal_trans')
      .setStyle(TextInputStyle.Short)
      .setPlaceholder('1.0 for fully opaque')
      .setValue(curent_trans)
      .setRequired(true)
      .setMaxLength(5)
      .setMinLength(3);
      
    const trans_label = new LabelBuilder()
      .setLabel("Set The Transparency")
      .setDescription("Set the Transparency as a number from 1.0 - 0.0")
      .setTextInputComponent(trans);

    modal
      .addLabelComponents(color_label)
      .addLabelComponents(trans_label);

    interaction.showModal(modal);
},
} as Button;
