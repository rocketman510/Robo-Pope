import { ButtonInteraction } from "discord.js";
import type { Button } from "../deploy";
import { spawn } from "child_process";

export default {
    data: "update",
    async execute(interaction: ButtonInteraction) {
      if (process.env.DEVELOPER_IDS?.includes(interaction.user.id)) {
        const GITHUB_TOKEN = process.env.GITHUB_TOKEN;
        spawn("./update.sh", [GITHUB_TOKEN], {
          detached: true,
          stdio: "ignore",
        }).unref();
      } else {
        interaction.update({content: "You have found yourself a bug. How did you get this?"})
        console.error("A User that is not a DEVELOPER has clicked the update Button. Something has gone very wrong");
      }
    },
} as Button;
