import { SlashCommandBuilder, ChatInputCommandInteraction, type ApplicationCommandOptionChoiceData, MessageFlags, ContainerBuilder, ButtonBuilder, ButtonStyle } from "discord.js";
import type { Command } from "../deploy";
import { readdirSync, readFileSync } from "fs";
import path from "path";
import type { Collection } from "mongodb";

type BookPrimitive = {
  _id: string;
  book_id: string;
  absolute_index: number;
  content: string;
  type: string;
  reference_number: number;
  is_image: boolean;
  image_path: string;
  foot_note: string[];
  next: string;
  previous: string;
  reference: {
    book: string;
    chapter: string;
    primitive: string;
    section: string;
  };
};

type Book = {
  _id: string;
  title: string;
  translation: string;
  language: string;
  thumbnail: string;
  author: string;
  books: Record<string, string>;
  chapters: Record<string, string>;
  primitives: Record<string, string>;
}


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
    const db = interaction.client.db;

    const books = db.collection<Book>('books');
    const primitives = db.collection<BookPrimitive>('book_primitives');

    const container = await render_page("nrsv-ci", "gen001001", 4000, primitives);

    interaction.reply({components: container, flags: [MessageFlags.Ephemeral, MessageFlags.IsComponentsV2]})
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

    buffer.push({name: file_json.metadata.title, value: file_json.metadata._id})
  }

  return buffer
}

async function render_page(book_id: string, start_id: string, max_caharacters: number, primitives: Collection<BookPrimitive>): Promise<ContainerBuilder[]> {
  const error = new ContainerBuilder().setAccentColor(0x242429).addTextDisplayComponents(t => t.setContent("Error Could not find that part of the book"));
  let entry = await primitives.findOne({_id: start_id, book_id: book_id });

  if (entry === null) return [error];

  let text_buffer = "# " + entry.reference.book + " " + entry.reference.chapter + "\n";

  while (text_buffer.length + entry.content.length < max_caharacters) {
    text_buffer += entry.type == "title" ? "### " : ""
    text_buffer += entry.reference_number != 0 ? to_superscript(entry.reference_number) : "";
    text_buffer += entry.content + '\n';

    entry = await primitives.findOne({_id: entry.next, book_id: book_id});
    if (entry === null) return [error];
  }

  const container = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addTextDisplayComponents(t => t.setContent(text_buffer))
    .addActionRowComponents(ar => ar
      .addComponents(new ButtonBuilder().setEmoji('').setLabel("Test").setStyle(ButtonStyle.Secondary).setCustomId("test"))
    )

  return [container]
}

const superscriptMap: Record<string, string> = {
  "0": "⁰",
  "1": "¹",
  "2": "²",
  "3": "³",
  "4": "⁴",
  "5": "⁵",
  "6": "⁶",
  "7": "⁷",
  "8": "⁸",
  "9": "⁹",
};

function to_superscript(num: number): string {
  return num
    .toString()
    .split("")
    .map(d => superscriptMap[d])
    .join("");
}
