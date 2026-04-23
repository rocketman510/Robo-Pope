import { SlashCommandBuilder, ChatInputCommandInteraction, type ApplicationCommandOptionChoiceData } from "discord.js";
import type { Command } from "../deploy";
import { readdirSync, readFileSync } from "fs";
import path from "path";

export default {
  data: new SlashCommandBuilder()
    .setName('read')
    .setDescription('Read form a selection of books.')
    .addStringOption(
      option => option
        .setName('book')
        .setDescription('The book to read')
        .addChoices(...get_choices('./books/'))
        .setRequired(true)
    ),
  async execute(interaction: ChatInputCommandInteraction) {
  },
} as Command;

function get_choices(dir_path: string): ApplicationCommandOptionChoiceData<string>[] {
  let buffer = []

  const dir = readdirSync(dir_path);

  for (const file of dir) {
    const file_name = path.parse(file).name;
    const file_ext = path.parse(file).ext;

    if (file_name.startsWith('.')) continue;
    if (file_ext != ".json") continue;

    const file_data = readFileSync(dir_path + file, "utf-8");
    const file_json = JSON.parse(file_data);

    buffer.push({name: file_json.name, value: file_json.id})
  }

  console.log(buffer);
  

  return buffer
}
