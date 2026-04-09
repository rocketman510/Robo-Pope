import { ModalSubmitInteraction, MessageFlags } from "discord.js";
import { ensure } from "..";
import { generateComponents } from "../functions/level_settings";
import { getLevelBannerSettings, setLevelBannerSettings } from "../level";

export async function colorModal(interaction: ModalSubmitInteraction, key: "primary" | "secondary" | "text") {
  const color = interaction.fields.getTextInputValue(`level_settings_set_${key}_color_modal_color`);
  const trans = interaction.fields.getTextInputValue(`level_settings_set_${key}_color_modal_trans`);

  const color_match = /^#([0-9a-f]{6})$/i.exec(color);
  const trans_match = /^[0-9]+(\.[0-9]+)?$/i.test(trans);

  if (!color_match || color_match.length < 1) {
    interaction.reply({ content: "Improperly formatted color. Try again", flags: [MessageFlags.Ephemeral] });
    return
  }

  if (!trans_match) {
    interaction.reply({ content: "Improperly formatted transparency value. Try again", flags: [MessageFlags.Ephemeral] });
    return
  }

  const final_color = Number("0x" + color_match![1]);
  const final_trans = Math.min(1, Number(trans));

  await interaction.deferUpdate()

  let level_settings = await getLevelBannerSettings(
    interaction.client,
    interaction.user.id,
    ensure(interaction.guildId)
  );

  level_settings[`${key}_color`] = final_color;
  level_settings[`${key}_color`] = final_trans;

  await setLevelBannerSettings(interaction.client, level_settings);

  const components = await generateComponents(level_settings, interaction.client)

  await interaction.editReply({
    components,
    flags: [
      MessageFlags.IsComponentsV2,
    ]
  });
}
