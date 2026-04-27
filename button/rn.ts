import { ButtonInteraction } from "discord.js";
import type { Button } from "../deploy";

export default {
    data: "rn",
    async execute(interaction: ButtonInteraction) {
      console.log(interaction.customId);
    },
} as Button;
