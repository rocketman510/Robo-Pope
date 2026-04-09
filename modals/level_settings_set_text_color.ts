import { ModalSubmitInteraction } from "discord.js";
import type { Modal } from "../deploy";
import { colorModal } from "../functions/level_settings_color_modal";

export default {
  data: "level_settings_set_primary_color_modal",
  async execute(interaction: ModalSubmitInteraction) {
    colorModal(interaction, "text");
  },
} as Modal;
