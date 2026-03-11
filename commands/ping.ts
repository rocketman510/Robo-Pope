import { SlashCommandBuilder, ChatInputCommandInteraction } from "discord.js";
import type { Command } from "../deploy";

export default {
    data: new SlashCommandBuilder()
        .setName('ping')
        .setDescription('ping the command.'),
    async execute(interaction: ChatInputCommandInteraction) {
        console.log(interaction.user.username);
        if (interaction.user.username == 'rocketman510') {
          interaction.reply("Pong");
        } else {
          interaction.reply("L bozo. You think I have to care what you say? No...")
        }
    },
} as Command;
